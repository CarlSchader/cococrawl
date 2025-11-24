use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde_json;
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
    output_dir_path: String,
}

fn main() {
    let args = Args::parse();

    let coco_json = fs::read_to_string(&args.coco_file).expect("Could not read COCO JSON file");
    let coco_json_file_name = args.coco_file.file_name().unwrap().to_string_lossy();

    // Make directory for output if it doesn't exist
    fs::create_dir_all(&args.output_dir_path).expect("Could not create output directory");

    let images_output_path = PathBuf::from(&args.output_dir_path).join("images");
    fs::create_dir_all(&images_output_path).expect("Could not create images output directory");

    let mut coco_file: cococrawl::CocoFile =
        serde_json::from_str(&coco_json).expect("Could not parse COCO JSON");

    // Iterate over images and copy them to the output directory
    let images_count = coco_file.images.len() as u64;
    let digits = ((images_count as f64).log10().floor() as usize) + 1;
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
                let file_extension = src_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");
                // output file name should be padded id with leading zeros + original extension
                let file_name = format!("{:0width$}.{}", image.id, file_extension, width = digits);
                let dest_path = images_output_path.join(file_name);
                let file_name = dest_path.file_name().unwrap();
                fs::copy(&src_path, &dest_path).expect("Could not copy image file");
                // change the file_name in the image struct to the new relative path
                image.file_name = format!("images/{}", file_name.to_string_lossy());
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
}
