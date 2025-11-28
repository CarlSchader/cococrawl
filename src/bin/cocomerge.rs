use chrono::{Datelike, Utc};
use clap::Parser;
use cococrawl::{
    CocoAnnotation, CocoCategory, CocoFile, CocoImage, CocoInfo, CocoLicense, HasCategoryID, HasID,
};
// use indicatif::ParallelProgressIterator;
// use rayon::prelude::*;
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
    coco_files: Vec<PathBuf>,

    /// JSON output path
    #[clap(short, long, default_value = "merged.json")]
    output_path: PathBuf,

    /// If files contain clashing image ids, reassign ids to new unique ids
    /// If not set then clashing ids will be ignored and the image id from the first file will be
    /// used
    #[clap(short, long)]
    reassign_clashing_ids: bool,

    /// Version string for the COCO info section
    #[clap(short, long, default_value = "1.0.0")]
    version_string: String,

    /// Force absolute paths for image file names in the merged output file.
    #[clap(short, long)]
    absolute_paths: bool,
}

fn main() {
    let args = Args::parse();

    let coco_files: Vec<CocoFile> = args
        .coco_files
        .iter()
        .map(|path| {
            let coco_json = fs::read_to_string(path).expect("Could not read COCO JSON file");
            serde_json::from_str(&coco_json).expect("Could not parse COCO JSON")
        })
        .collect();

    // Categories don't hash on id but instead they hash on the everything else in the struct.
    // This allows us to use this as a ground truth for making sure all categories have the same id
    // across files.
    let mut category_set: HashSet<CocoCategory> = HashSet::new();
    let mut category_seen_ids: HashSet<i32> = HashSet::new();
    let mut next_unseen_category_id: i32 = 0; // this can technically start at any number but we start at 0 for simplicity

    // Licenses work the same way as categories
    let mut license_set: HashSet<CocoLicense> = HashSet::new();
    let mut license_seen_ids: HashSet<i32> = HashSet::new();
    let mut next_unseen_license_id: i32 = 0;

    let mut images: Vec<CocoImage> = Vec::new();
    let mut seen_image_ids: HashSet<i64> = HashSet::new();
    let mut next_unseen_image_id: i64 = 0;

    let mut annotations: Vec<CocoAnnotation> = Vec::new();
    let mut seen_annotation_ids: HashSet<i64> = HashSet::new();
    let mut next_unseen_annotation_id: i64 = 0;

    coco_files.iter().enumerate().for_each(|(file_index, coco_file)| {
        let coco_file_path = &args.coco_files[file_index];

        // categories logic
        let mut category_id_remap: HashMap<i32, i32> = HashMap::new();
        coco_file.categories.as_ref().map(|categories| categories.iter().for_each(|category| {
            if let Some(entry) = category_set.get(category) {
                // category id exists so we use the existing id
                category_id_remap.insert(category.id(), entry.id());
            } else {
                if category_seen_ids.contains(&category.id()) {
                    // category hasn't been seen yet and it's id clashes with an existing category
                    let mut new_category = category.clone();
                    new_category.set_id(next_unseen_category_id);
                    next_unseen_category_id += 1;
                    category_id_remap.insert(category.id(), new_category.id());
                    category_set.insert(new_category);
                } else {
                    // category hasn't been seen yet and it's id doesn't clash
                    category_seen_ids.insert(category.id());
                    if category.id() >= next_unseen_category_id {
                        next_unseen_category_id = category.id() + 1;
                    }
                    category_id_remap.insert(category.id(), category.id());
                    category_set.insert(category.clone());
                }
            }
        }));

        // licenses logic
        let mut license_id_remap: HashMap<i32, i32> = HashMap::new();
        coco_file.licenses.as_ref().map(|licenses| licenses.iter().for_each(|license| {
            if let Some(entry) = license_set.get(license) {
                // license id exists so we use the existing id
                license_id_remap.insert(license.id(), entry.id());
            } else {
                if license_seen_ids.contains(&license.id()) {
                    // license hasn't been seen yet and it's id clashes with an existing license
                    let mut new_license = license.clone();
                    new_license.set_id(next_unseen_license_id);
                    next_unseen_license_id += 1;
                    license_id_remap.insert(license.id(), new_license.id());
                    license_set.insert(new_license);
                } else {
                    // license hasn't been seen yet and it's id doesn't clash
                    license_seen_ids.insert(license.id());
                    if license.id() >= next_unseen_license_id {
                        next_unseen_license_id = license.id() + 1;
                    }
                    license_id_remap.insert(license.id(), license.id());
                    license_set.insert(license.clone());
                }
            }
        }));

        // images logic
        let mut image_id_remap: HashMap<i64, i64> = HashMap::new();
        coco_file.images.iter().for_each(|image| {
            let mut new_image = image.clone();

            // hanlde image path
            new_image.file_name = if image.file_name.is_absolute() {
                image.file_name.clone()
            } else {
                coco_file_path
                    .parent()
                    .unwrap()
                    .join(&image.file_name)
            };

            // handle license
            if let Some(new_license_id) = new_image.license {
                new_image.license = Some(license_id_remap.get(&new_license_id)
                    .expect(format!(
                        "License id {} not found in remap for image id {} in file {}",
                        new_license_id,
                        new_image.id(),
                        coco_file_path.to_string_lossy(),
                    ).as_str())
                    .clone());
            }

            if seen_image_ids.contains(&image.id()) {
                if args.reassign_clashing_ids {
                    new_image.set_id(next_unseen_image_id);
                    next_unseen_image_id += 1;
                    seen_image_ids.insert(new_image.id());
                    image_id_remap.insert(image.id(), new_image.id());
                    images.push(new_image);
                } else {
                    // ignore clashing image
                    eprintln!(
                        "Warning: Image id {} in file {} clashes with an existing image id. Ignoring this image.",
                        image.id(),
                        coco_file_path.to_string_lossy(),
                    );
                }
            } else {
                if new_image.id() >= next_unseen_image_id {
                    next_unseen_image_id = new_image.id() + 1;
                }
                seen_image_ids.insert(new_image.id());
                image_id_remap.insert(image.id(), new_image.id());
                images.push(new_image);
            }
        });

        // annotations logic
        coco_file.annotations.iter().for_each(|annotation| {
            // only add annotation if its image id was added
            if let Some(new_annotation_id) = image_id_remap.get(&annotation.image_id()) {
                let mut new_annotation = annotation.clone();
                new_annotation.set_image_id(*new_annotation_id);

                // handle category id remappings and annotation id remapping
                match new_annotation {
                    CocoAnnotation::KeypointDetection(ref mut ann) => {
                        let new_category_id = *category_id_remap.get(&ann.category_id()).expect(
                            format!(
                                "Category id {} not found in remap for annotation id {} in file {}",
                                ann.category_id(),
                                ann.id(),
                                coco_file_path.to_string_lossy(),
                            ).as_str()
                        );
                        ann.set_category_id(new_category_id);

                        if seen_annotation_ids.contains(&ann.id()) {
                            ann.set_id(next_unseen_annotation_id);
                            next_unseen_annotation_id += 1;
                            seen_annotation_ids.insert(ann.id());
                        } else {
                            if ann.id() >= next_unseen_annotation_id {
                                next_unseen_annotation_id = ann.id() + 1;
                            }
                            seen_annotation_ids.insert(ann.id());
                        }
                    },
                    CocoAnnotation::PanopticSegmentation(ref mut ann) => {
                        ann.segments_info.iter_mut().for_each(|segment| {
                            let new_category_id = *category_id_remap.get(&segment.category_id).expect(
                                format!(
                                    "Category id {} not found in remap for segment info id {} in file {}",
                                    segment.category_id,
                                    segment.id(),
                                    coco_file_path.to_string_lossy(),
                                ).as_str()
                            );
                            segment.category_id = new_category_id;

                            // special case. We want unique segment ids across the whole dataset
                            // including other annotation types
                            if seen_annotation_ids.contains(&segment.id()) {
                                segment.set_id(next_unseen_annotation_id);
                                next_unseen_annotation_id += 1;
                                seen_annotation_ids.insert(segment.id());
                            } else {
                                if segment.id() >= next_unseen_annotation_id {
                                    next_unseen_annotation_id = segment.id() + 1;
                                }
                                seen_annotation_ids.insert(segment.id());
                            }
                        });
                    },
                    CocoAnnotation::ImageCaptioning(ref mut ann) => {
                        if seen_annotation_ids.contains(&ann.id()) {
                            ann.set_id(next_unseen_annotation_id);
                            next_unseen_annotation_id += 1;
                            seen_annotation_ids.insert(ann.id());
                        } else {
                            if ann.id() >= next_unseen_annotation_id {
                                next_unseen_annotation_id = ann.id() + 1;
                            }
                            seen_annotation_ids.insert(ann.id());
                        }
                    },
                    CocoAnnotation::ObjectDetection(ref mut ann) => {
                        let new_category_id = *category_id_remap.get(&ann.category_id()).expect(
                            format!(
                                "Category id {} not found in remap for annotation id {} in file {}", 
                                ann.category_id(),
                                ann.id(),
                                coco_file_path.to_string_lossy(),
                            ).as_str()
                        );
                        ann.set_category_id(new_category_id);

                        if seen_annotation_ids.contains(&ann.id()) {
                            ann.set_id(next_unseen_annotation_id);
                            next_unseen_annotation_id += 1;
                            seen_annotation_ids.insert(ann.id());
                        } else {
                            if ann.id() >= next_unseen_annotation_id {
                                next_unseen_annotation_id = ann.id() + 1;
                            }
                            seen_annotation_ids.insert(ann.id());
                        }
                    },
                    CocoAnnotation::DensePose(ref mut ann) => {
                        let new_category_id = *category_id_remap.get(&ann.category_id()).expect(
                            format!(
                                "Category id {} not found in remap for annotation id {} in file {}", 
                                ann.category_id(),
                                ann.id(),
                                coco_file_path.to_string_lossy(),
                            ).as_str()
                        );
                        ann.set_category_id(new_category_id);

                        if seen_annotation_ids.contains(&ann.id()) {
                            ann.set_id(next_unseen_annotation_id);
                            next_unseen_annotation_id += 1;
                            seen_annotation_ids.insert(ann.id());
                        } else {
                            if ann.id() >= next_unseen_annotation_id {
                                next_unseen_annotation_id = ann.id() + 1;
                            }
                            seen_annotation_ids.insert(ann.id());
                        }
                    },
                }

                annotations.push(new_annotation);
            }
        });
    });

    let merged_file = CocoFile {
        info: Some(CocoInfo {
            year: Utc::now().year(),
            version: args.version_string,
            description: "".to_string(),
            contributor: "".to_string(),
            url: "".to_string(),
            date_created: Utc::now(),
        }),
        licenses: Some(license_set.into_iter().collect()),
        images,
        annotations,
        categories: Some(category_set.into_iter().collect()),
    };

    let merged_path = PathBuf::from(&args.output_path);
    let output_file = File::create(&merged_path).expect("Could not create output COCO JSON file");
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &merged_file)
        .expect("Could not write COCO JSON to output file");
}
