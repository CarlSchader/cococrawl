use clap::Parser;
use cococrawl::{CocoCategory, CocoFile, CocoImage};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// coco JSON file paths to merge
    #[clap(required = true)]
    coco_file: Vec<PathBuf>,

    /// JSON output path
    #[clap(short, long, default_value = "merged.json")]
    output_path: PathBuf,

    /// If files contain clashing image ids, reassign ids to new unique ids
    /// If not set then clashing ids will be ignored and the image id from the first file will be
    /// used 
    #[clap(short, long)]
    reassign_clashing_ids: bool,
}

fn main() {
    let args = Args::parse();

    let coco_files: Vec<CocoFile> = args.coco_file.iter().map(|path| {
        let coco_json = fs::read_to_string(path).expect("Could not read COCO JSON file");
        serde_json::from_str(&coco_json).expect("Could not parse COCO JSON")
    }).collect(); 

    let mut seen_image_ids: HashSet<i64> = HashSet::new();
    let mut categories: HashSet<CocoCategory> = HashSet::new();
    let mut images: Vec<CocoImage> = Vec::new();

    coco_files.iter().for_each(|coco_file| {
        let mut category_id_reemap: HashMap<>
        coco_file.categories.iter().for_each(|category| {
            categories.insert(category.clone());
        });
    });

    let output_coco_path =
        PathBuf::from(&args.output_dir_path).join(coco_json_file_name.to_string());
    let output_file =
        File::create(&output_coco_path).expect("Could not create output COCO JSON file");
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &coco_file)
        .expect("Could not write COCO JSON to output file");
}
