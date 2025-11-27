use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde_json;
use anyhow::Result;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

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

fn main() -> Result<()> {
    let args = Args::parse();

    let coco_json = fs::read_to_string(&args.coco_file).expect("Could not read COCO JSON file");
    let coco_json_file_name = args.coco_file.file_name().unwrap().to_string_lossy();
    
    // Make directory for output if it doesn't exist
    fs::create_dir_all(&args.output_dir_path).expect("Could not create output directory");
    let output_dir_path = args.output_dir_path.canonicalize()?;

    let images_output_path = output_dir_path.join("images");
    fs::create_dir_all(&images_output_path).expect("Could not create images output directory");
    let images_output_path = images_output_path.canonicalize()?;

    let mut coco_file: cococrawl::CocoFile =
        serde_json::from_str(&coco_json).expect("Could not parse COCO JSON");

    // Iterate over images and copy them to the output directory
    let images_count = coco_file.images.len() as u64;
    coco_file
        .images
        .par_iter_mut()
        .progress_count(images_count)
        .for_each(|image| {
            // src_path is relative to the input coco json file location
            // unless it's an absolute path
            let src_path = if PathBuf::from(&image.file_name).is_absolute() {
                PathBuf::from(&image.file_name)
            } else {
                args.coco_file.parent().unwrap().join(&image.file_name)
            };
            if src_path.exists() && src_path.is_file() {
                // output file name is the original basename prefixed with the path to the
                // images_output_path
                let file_name = src_path
                    .file_name()
                    .expect(format!(
                        "Could not get file name for source image path {:?}",
                        src_path
                    ).as_str());

                let dest_path = images_output_path
                    .join(file_name);

                fs::copy(&src_path, &dest_path)
                    .expect(format!(
                        "Could not copy image from {:?} to {:?}",
                        src_path, dest_path
                    ).as_str());

                // written path is relative to the output coco json file location
                // unless absolute_paths is set
                let written_path = if args.absolute_paths {
                    dest_path
                } else {
                    dest_path
                        .strip_prefix(output_dir_path.clone())
                        .expect(format!(
                            "Could not strip prefix {:?} from destination path {:?}",
                            output_dir_path, dest_path
                        ).as_str())
                        .to_path_buf()
                };

                image.file_name = written_path;

            } else {
                eprintln!(
                    "Warning: Source image file does not exist or is not a file: {:?}",
                    src_path
                );
            }
        });

    // Write updated COCO JSON to output directory
    let output_coco_path =
        PathBuf::from(&args.output_dir_path).join(coco_json_file_name.to_string());
    let output_file =
        File::create(&output_coco_path).expect("Could not create output COCO JSON file");
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &coco_file)
        .expect("Could not write COCO JSON to output file");

    Ok(())
}
