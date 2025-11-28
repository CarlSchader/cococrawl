use clap::Parser;
use cococrawl::path_utils::create_coco_image_path;
use cococrawl::{CocoFile, IDMapEntry};
use indicatif::ParallelProgressIterator;
use rand::{SeedableRng, rng, rngs::StdRng, seq::SliceRandom};
use rayon::prelude::*;
use serde_json;
use std::collections::HashSet;
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
    #[clap(short, long, default_value = "split.json")]
    output: PathBuf,

    /// number of images to create the split with
    /// if not provided, all images not in the blacklisted sets will be used
    #[clap(short, long)]
    count: Option<usize>,

    /// blacklist dataset JSON file paths
    /// cocosplit dataset.json -o val-set.json -c 10000
    /// cocosplit dataset.json -o test-set.json -c 20000 -b val-set.json
    /// cocosplit dataset.json -o train-set.json -b test-set.json -b val-set.json
    #[clap(short, long)]
    blacklist_file: Vec<PathBuf>,

    /// shuffle the images before splitting (with optional seed for reproducibility)
    #[clap(long)]
    shuffle: Option<Option<u64>>,

    /// offset index to start at when splitting (only valid without shuffle)
    /// if offset is 0, starts from the lowest image_id
    #[clap(long, conflicts_with = "shuffle")]
    offset: Option<usize>,

    /// annotated images only
    /// if set, only images with at least one annotation will be included in the split
    #[clap(long)]
    annotated_only: bool,

    /// Force absolute paths for image file names in the split output file.
    #[clap(short, long)]
    absolute_paths: bool,
}

fn main() {
    let args = Args::parse();

    let coco_json = fs::read_to_string(&args.coco_file).expect("Could not read COCO JSON file");
    let coco_file: cococrawl::CocoFile =
        serde_json::from_str(&coco_json).expect("Could not parse COCO JSON");


    // create output file upfront so canonicalize works
    let output_file = File::create(&args.output).expect("Could not create output file");

    let blacklisted_image_ids: HashSet<i64> = args
        .blacklist_file
        .iter()
        .flat_map(|path| {
            let json_str =
                fs::read_to_string(path).expect("Could not read blacklist COCO JSON file");
            let blacklist_coco: cococrawl::CocoFile =
                serde_json::from_str(&json_str).expect("Could not parse blacklist COCO JSON");
            blacklist_coco
                .images
                .into_par_iter()
                .progress()
                .map(|img| img.id)
                .collect::<HashSet<i64>>()
        })
        .collect();

    let id_map = coco_file.make_image_id_map();
    let mut id_map_entries: Vec<(&i64, &IDMapEntry<'_>)> = id_map
        .par_iter()
        .progress_count(id_map.len() as u64)
        .filter(|(id, _)| !blacklisted_image_ids.contains(id))
        .collect();


    if args.shuffle.is_some() {
        match args.shuffle.unwrap() {
            Some(seed) => {
                let mut rng = StdRng::seed_from_u64(seed);
                id_map_entries.shuffle(&mut rng);
            }
            None => {
                let mut rng = rng();
                id_map_entries.shuffle(&mut rng);
            }
        }
    } else {
        id_map_entries.sort_by_key(|(id, _)| *id);
    }

    // filter annotated only
    let id_map_entries: Vec<_> = if args.annotated_only {
        eprintln!("Filtering to annotated images only...");
        id_map_entries
            .into_par_iter()
            .progress()
            .filter(|(_, entry)| !entry.annotations.is_empty())
            .collect()
    } else {
        id_map_entries
    };

    let offset = args.offset.unwrap_or(0);
    let output_count = args
        .count
        .unwrap_or(id_map_entries.len().saturating_sub(offset));

    let id_map_entries: Vec<(&i64, &IDMapEntry<'_>)> = id_map_entries
        .into_iter()
        .skip(offset)
        .take(output_count)
        .collect();

    // Write updated COCO JSON to output directory
    let output_coco_file = CocoFile {
        info: coco_file.info.clone(),
        images: id_map_entries
            .par_iter()
            .progress()
            .map(|(_, entry)| {
                let mut new_image = entry.image.clone();
                new_image.file_name = create_coco_image_path(
                    args.output.as_path(),
                    new_image.get_absolute_path(&args.coco_file).expect("Could not get absolute image path").as_path(),
                    args.absolute_paths,
                ).expect(format!(
                    "Could not create COCO image path for image id {}",
                    new_image.id
                ).as_str());
                new_image
            })
            .collect(),
        annotations: id_map_entries
            .par_iter()
            .progress()
            .flat_map(|(_, entry)| {
                entry
                    .annotations
                    .clone()
                    .into_par_iter()
                    .map(|ann| ann.clone())
            })
            .collect(),
        categories: coco_file.categories.clone(),
        licenses: coco_file.licenses.clone(),
    };

    let writer = BufWriter::new(output_file);

    serde_json::to_writer_pretty(writer, &output_coco_file)
        .expect("Could not write JSON to output file");
}
