use clap::Parser;
use cococrawl::{CocoAnnotation, CocoCategory};
use serde_json;
use std::fs;
use std::path::PathBuf;

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
    let coco_file: cococrawl::CocoFile =
        serde_json::from_str(&coco_json).expect("Could not parse COCO JSON");

    // Iterate over images and copy them to the output directory
    let images_count = coco_file.images.len() as u64;

    let annotations_count = coco_file.annotations.len() as u64;
    let annotation_counts: &mut [u64] = &mut [0; 5];
    coco_file
        .annotations
        .iter()
        .for_each(|annotation: &CocoAnnotation| match *annotation {
            CocoAnnotation::ObjectDetection(_) => annotation_counts[0] += 1,
            CocoAnnotation::KeypointDetection(_) => annotation_counts[1] += 1,
            CocoAnnotation::PanopticSegmentation(_) => annotation_counts[2] += 1,
            CocoAnnotation::ImageCaptioning(_) => annotation_counts[3] += 1,
            CocoAnnotation::DensePose(_) => annotation_counts[4] += 1,
        });

    let categories_count: &mut [u64] = &mut [0; 3];
    let category_count = coco_file.categories.clone().unwrap_or_default().len() as u64;
    coco_file
        .categories
        .unwrap_or_default()
        .iter()
        .for_each(|category| match category {
            CocoCategory::ObjectDetection(_) => categories_count[0] += 1,
            CocoCategory::PanopticSegmentation(_) => categories_count[0] += 1,
            CocoCategory::KeypointDetection(_) => categories_count[0] += 1,
        });

    println!("Coco File: {}", coco_json_file_name);
    println!("Images: {}", images_count);
    println!("Annotations: {}", annotations_count);

    println!("  Object Detection Annotations: {}", annotation_counts[0]);
    println!("  Keypoint Detection Annotations: {}", annotation_counts[1]);
    println!(
        "  Panoptic Segmentation Annotations: {}",
        annotation_counts[2]
    );
    println!("  Image Captioning Annotations: {}", annotation_counts[3]);
    println!("  DensePose Annotations: {}", annotation_counts[4]);

    println!("Categories: {}", category_count);
    println!("  Object Detection Categories: {}", categories_count[0]);
    println!(
        "  Panoptic Segmentation Categories: {}",
        categories_count[1]
    );
    println!("  Keypoint Detection Categories: {}", categories_count[2]);
}
