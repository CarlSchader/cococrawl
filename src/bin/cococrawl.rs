use chrono::{DateTime, Datelike, Utc};
use clap::Parser;
use image::ImageReader;
use anyhow::{Context, Result};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use cococrawl::{CocoFile, CocoImage, CocoInfo, path_utils::create_coco_image_path};
use cococrawl::output::write_json_atomic;

const IMAGE_EXTENSIONS: [&str; 8] = ["png", "jpg", "jpeg", "gif", "bmp", "tiff", "svg", "webp"];

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Directories to crawl positional arguments
    #[clap(required = true)]
    directories: Vec<String>,

    /// JSON output path
    #[clap(short, long, default_value = "coco.json")]
    output: PathBuf,

    /// Version string for the COCO info section
    #[clap(short, long, default_value = "1.0.0")]
    version_string: String,

    /// Force absolute paths for image file names. By default, relative paths are used if the image
    /// is located within the same directory tree as the output JSON file. Otherwise, absolute paths are used.
    #[clap(short, long)]
    absolute_paths: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let extension_set: HashSet<&str> = IMAGE_EXTENSIONS.iter().cloned().collect();

    let mut entries = Vec::new();
    for dir in &args.directories {
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry.with_context(|| format!("Could not traverse input {dir}"))?;
            if entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| {
                        extension_set.contains(&ext.to_lowercase().as_str())
                    })
            {
                entries.push(entry);
            }
        }
    }

    let images: Vec<CocoImage> = entries
        .par_iter()
        .progress_count(entries.len() as u64)
        .enumerate()
        .map(|(id, entry)| -> Result<CocoImage> {
            let written_path = create_coco_image_path(
                &args.output,
                entry.path(),
                args.absolute_paths,
            )
            .with_context(|| {
                format!(
                    "Could not create COCO image path for {} relative to {}",
                    entry.path().display(),
                    args.output.display()
                )
            })?;
            let metadata = fs::metadata(entry.path())
                .with_context(|| format!("Could not read metadata for {}", entry.path().display()))?;
            let date_created = metadata.created().ok();
            let date_created = date_created.map(|dt| DateTime::<Utc>::from(dt));

            let (width, height) = ImageReader::open(&entry.path())
                .with_context(|| format!("Could not open image {}", entry.path().display()))?
                .with_guessed_format()
                .with_context(|| format!("Could not read image {}", entry.path().display()))?
                .into_dimensions()
                .unwrap_or((0, 0));

            Ok(CocoImage {
                id: id as i64,
                width,
                height,
                file_name: written_path,
                license: None,
                flickr_url: None,
                coco_url: None,
                date_captured: date_created,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let coco_info = CocoInfo {
        year: Utc::now().year(),
        version: args.version_string,
        description: "".to_string(),
        contributor: "".to_string(),
        url: "".to_string(),
        date_created: Utc::now(),
    };

    let coco_file = CocoFile {
        info: Some(coco_info),
        images,
        annotations: Vec::new(),
        categories: None,
        licenses: None,
    };

    write_json_atomic(&args.output, &coco_file)?;

    Ok(())
}
