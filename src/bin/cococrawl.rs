use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use indicatif::ProgressIterator;
use clap::Parser;
use image::ImageReader;
use chrono::{Datelike, Utc, DateTime};
use serde_json;
// use rayon::prelude::*;

use cococrawl::{CocoFile, CocoImage, CocoInfo};

const IMAGE_EXTENSIONS: [&str; 8] = ["png", "jpg", "jpeg", "gif", "bmp", "tiff", "svg", "webp"];

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Directories to crawl positional arguments
    #[clap(required = true)]
    directories: Vec<String>,

    /// JSON output path
    #[clap(short, long, default_value = "coco.json")]
    output: String,

    #[clap(short, long, default_value = "1.0.0")]
    version_string: String,
}

fn main() {
    let args = Args::parse();

    let output_file = File::create(&args.output).expect("Could not create output file");

    let extension_set: HashSet<&str> = IMAGE_EXTENSIONS.iter().cloned().collect();

    let found_files: Vec<PathBuf> = args.directories.iter().flat_map(|dir| {
        walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext_str| extension_set.contains(&ext_str.to_lowercase().as_str()))
            })
            .map(|entry| entry.path().to_path_buf())
    }).collect();

    let images: Vec<CocoImage> = found_files.iter().progress().enumerate().map(|(id, file_path)| {
        let metadata = fs::metadata(file_path).unwrap();
        let date_created = metadata.created().unwrap_or_else(|_| std::time::SystemTime::now());

        let (width, height) = ImageReader::open(file_path).unwrap()
            .with_guessed_format().unwrap()
            .into_dimensions().unwrap_or((0, 0));

        CocoImage {
            id: id as u64,
            width,
            height,
            file_name: file_path.to_string_lossy().to_string(),
            license: 0,
            flickr_url: String::new(),
            coco_url: String::new(),
            date_captured: DateTime::<Utc>::from(date_created),
        }
    }).collect();

    let coco_info = CocoInfo {
        year: Utc::now().year(),
        version: args.version_string,
        description: "".to_string(),
        contributor: "".to_string(),
        url: "".to_string(),
        date_created: Utc::now(),
    };

    let coco_file = CocoFile {
        info: coco_info,
        images,
        annotations: Vec::new(),
    };

    let writer = BufWriter::new(output_file);

    serde_json::to_writer_pretty(writer, &coco_file).expect("Could not write JSON to output file");
}
