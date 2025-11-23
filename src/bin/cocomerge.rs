use clap::Parser;
use cococrawl::{CocoCategory, CocoFile, CocoImage, HasID, SetID};
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

    // Categories don't hash on id but instead they hash on the everything else in the struct.
    // This allows us to use this as a ground truth for making sure all categories have the same id
    // across files.
    let mut category_set: HashSet<CocoCategory> = HashSet::new(); 
    let mut category_seen_ids: HashSet<i64> = HashSet::new();
    let mut next_unseen_category_id: i64 = 0; // this can technically start at any number but we start at 0 for simplicity

    let mut images: Vec<CocoImage> = Vec::new();

    coco_files.iter().for_each(|coco_file| {
        // remap category ids that clash
        let mut category_id_remap: HashMap<i64, i64> = HashMap::new();
        coco_file.categories.map(|categories| categories.iter().for_each(|category| {
            if let Some(entry) = category_set.get(category) {                                                               
                // Category id exists so we use the existing id
                category_id_remap.insert(category.id(), entry.id());
            } else { 
                if category_seen_ids.contains(&category.id()) {
                    // category hasn't been seen yet and it's id clashes with an existing category
                    let mut new_category = category.clone();
                    new_category.set_id(next_unseen_category_id);
                    next_unseen_category_id += 1;
                    category_id_remap.insert(category.id(), new_category.id());
                } else {
                    // category hasn't been seen yet and it's id doesn't clash
                    category_seen_ids.insert(category.id());
                    if category.id() >= next_unseen_category_id {
                        next_unseen_category_id = category.id() + 1;
                    }
                    category_id_remap.insert(category.id(), category.id());
                }
            }


        }));
    });

    let output_coco_path =
        PathBuf::from(&args.output_dir_path).join(coco_json_file_name.to_string());
    let output_file =
        File::create(&output_coco_path).expect("Could not create output COCO JSON file");
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &coco_file)
        .expect("Could not write COCO JSON to output file");
}
