use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use cococrawl::output::{validate_output_destination, write_json_atomic};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use same_file::is_same_file;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// coco JSON file path
    #[clap(required = true)]
    coco_file: PathBuf,

    /// JSON output path
    #[clap(short, long, default_value = "coco-dataset")]
    output_dir_path: PathBuf,

    /// Force absolute paths for copied image file names. By default, relative paths are used.
    #[clap(short, long)]
    absolute_paths: bool,
}

// Resolve aliases without creating missing output directories or files. In
// particular, a dangling symlink may target a not-yet-copied image.
fn resolve_planned_path(path: &Path, remaining_links: u32) -> Result<PathBuf> {
    match path.canonicalize() {
        Ok(resolved) => return Ok(resolved),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).with_context(|| format!("Could not resolve {:?}", path)),
    }

    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_symlink() => {
            if remaining_links == 0 {
                bail!("Too many symlinks while resolving {:?}", path);
            }
            let target = fs::read_link(path)?;
            let target = path.parent().context("Symlink has no parent")?.join(target);
            return resolve_planned_path(&target, remaining_links - 1);
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).with_context(|| format!("Could not inspect {:?}", path)),
    }

    let parent = path.parent().context("Output path has no existing ancestor")?;
    let mut resolved = resolve_planned_path(parent, remaining_links)?;
    match path.components().next_back() {
        Some(Component::Normal(name)) => resolved.push(name),
        Some(Component::ParentDir) => {
            resolved.pop();
        }
        Some(Component::CurDir) => {}
        _ => bail!("Could not resolve output path {:?}", path),
    }
    Ok(resolved)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let coco_json = fs::read_to_string(&args.coco_file).expect("Could not read COCO JSON file");
    let coco_json_file_name = args.coco_file.file_name().unwrap().to_string_lossy();

    let mut coco_file: cococrawl::CocoFile =
        serde_json::from_str(&coco_json).expect("Could not parse COCO JSON");

    // Validate every mapping before creating directories or copying any bytes.
    // A destination can be shared only by references to the very same source file.
    let images_output_path = args.output_dir_path.join("images");
    // Reserve the eventual manifest destination too: an aliased images directory
    // must not let the final manifest write replace an image or its source.
    let manifest_path = args.output_dir_path.join(coco_json_file_name.as_ref());
    // same_file opens paths for reading, which would block on a FIFO manifest.
    // Reject unsupported output types without opening them, before identity checks.
    validate_output_destination(&manifest_path)
        .with_context(|| format!("Manifest destination {:?} is not supported", manifest_path))?;
    let resolved_manifest = resolve_planned_path(&std::path::absolute(&manifest_path)?, 40)?;
    let mut destinations: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut copies = Vec::new();
    let mut image_file_names = Vec::with_capacity(coco_file.images.len());
    for image in &coco_file.images {
        let src_path = if image.file_name.is_absolute() {
            image.file_name.clone()
        } else {
            args.coco_file.parent().unwrap().join(&image.file_name)
        };
        if !src_path.exists() || !src_path.is_file() {
            eprintln!(
                "Warning: Source image file does not exist or is not a file: {:?}",
                src_path
            );
            image_file_names.push(None);
            continue;
        }

        let file_name = src_path
            .file_name()
            .with_context(|| format!("Could not get source image file name: {:?}", src_path))?
            .to_owned();
        let dest_path = images_output_path.join(&file_name);
        for image_path in [&src_path, &dest_path] {
            let resolved_image = resolve_planned_path(&std::path::absolute(image_path)?, 40)?;
            let overlaps_manifest = if resolved_image == resolved_manifest {
                true
            } else {
                match is_same_file(&resolved_manifest, &resolved_image) {
                    Ok(same) => same,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => false,
                    Err(error) => {
                        return Err(error).with_context(|| {
                            format!("Could not compare manifest and image paths {:?}", image_path)
                        });
                    }
                }
            };
            if overlaps_manifest {
                bail!(
                    "Manifest destination {:?} overlaps image source or destination {:?}",
                    manifest_path,
                    image_path
                );
            }
        }
        if let Some(previous_source) = destinations.get(&dest_path) {
            if !is_same_file(previous_source, &src_path).with_context(|| {
                format!(
                    "Could not compare sources {:?} and {:?}",
                    previous_source, src_path
                )
            })? {
                bail!(
                    "Destination collision at {:?}: distinct source files {:?} and {:?}",
                    dest_path,
                    previous_source,
                    src_path
                );
            }
        } else {
            // symlink_metadata also detects dangling links: never replace or follow
            // an unrelated destination, even if its target does not exist yet.
            match fs::symlink_metadata(&dest_path) {
                Ok(_) => {
                    if !is_same_file(&src_path, &dest_path).with_context(|| {
                        format!("Could not verify existing destination {:?}", dest_path)
                    })? {
                        bail!(
                            "Refusing to overwrite existing destination {:?} with source {:?}",
                            dest_path,
                            src_path
                        );
                    }
                    // Same-path, symlink and hard-link copies are successful no-ops.
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    copies.push((src_path.clone(), dest_path.clone()));
                }
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("Could not inspect destination {:?}", dest_path));
                }
            }
            destinations.insert(dest_path, src_path);
        }
        image_file_names.push(Some(file_name));
    }

    fs::create_dir_all(&images_output_path).context("Could not create images output directory")?;
    let output_dir_path = args.output_dir_path.canonicalize()?;
    let images_output_path = images_output_path.canonicalize()?;

    // Each destination has at most one writer. Exclusive creation also prevents
    // overwriting a file or following a symlink introduced after preflight.
    copies
        .par_iter()
        .progress_count(copies.len() as u64)
        .try_for_each(|(src_path, dest_path)| -> Result<()> {
            let mut source = File::open(src_path)
                .with_context(|| format!("Could not open source image {:?}", src_path))?;
            let mut destination = File::options()
                .write(true)
                .create_new(true)
                .open(dest_path)
                .with_context(|| format!("Could not create destination image {:?}", dest_path))?;
            io::copy(&mut source, &mut destination).with_context(|| {
                format!("Could not copy image from {:?} to {:?}", src_path, dest_path)
            })?;
            destination.set_permissions(source.metadata()?.permissions())?;
            Ok(())
        })?;

    for (image, file_name) in coco_file.images.iter_mut().zip(image_file_names) {
        if let Some(file_name) = file_name {
            let dest_path = images_output_path.join(file_name);
            image.file_name = if args.absolute_paths {
                dest_path
            } else {
                dest_path.strip_prefix(&output_dir_path)?.to_path_buf()
            };
        }
    }

    // Write updated COCO JSON to output directory
    let output_coco_path =
        PathBuf::from(&args.output_dir_path).join(coco_json_file_name.to_string());
    write_json_atomic(&output_coco_path, &coco_file)?;

    Ok(())
}
