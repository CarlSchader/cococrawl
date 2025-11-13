use std::path::PathBuf;
use std::fs;
use clap::Parser;
use serde_json;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// coco JSON file path
    #[clap(required = true)]
    coco_file: PathBuf,
}

fn main() {
    let args = Args::parse();

    let coco_json = fs::read_to_string(&args.coco_file).expect("Could not read COCO JSON file");
    let coco_json_file_name = args.coco_file.file_name().unwrap().to_string_lossy();
    let coco_file: cococrawl::CocoFile = serde_json::from_str(&coco_json).expect("Could not parse COCO JSON");

    // Iterate over images and copy them to the output directory
    let images_count = coco_file.images.len() as u64;
    let annotations_count = coco_file.annotations.len() as u64;

    println!("Coco File: {}", coco_json_file_name);
    println!("Images: {}", images_count);
    println!("Annotations: {}", annotations_count);
}
