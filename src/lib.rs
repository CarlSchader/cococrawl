use std::collections::HashMap;

use chrono::{DateTime, Utc};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoInfo {
    pub year: i32,
    pub version: String,
    pub description: String,
    pub contributor: String,
    pub url: String,
    pub date_created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoImage {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub file_name: String,
    pub license: u32,
    pub flickr_url: String,
    pub coco_url: String,
    pub date_captured: DateTime<Utc>,
}

// dummy for now
// will need to be virtual with sub classes if I decide to actually use annotations
#[derive(Serialize, Deserialize, Clone)]
pub struct CocoAnnotation {
    pub id: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoFile {
    pub info: CocoInfo,
    pub images: Vec<CocoImage>,
    pub annotations: Vec<CocoAnnotation>,
}

pub struct IDMapEntry<'a> {
    pub id: u64,
    pub image: &'a CocoImage,
    pub annotations: Vec<&'a CocoAnnotation>,
}

impl CocoFile {
    pub fn make_id_map(&self) -> std::collections::HashMap<u64, IDMapEntry<'_>> {
        let image_map: HashMap<u64, &CocoImage> = self
            .images
            .par_iter()
            .progress()
            .map(|im| (im.id, im))
            .collect();
        let annotation_map: HashMap<u64, Vec<&CocoAnnotation>> = self
            .annotations
            .par_iter()
            .progress()
            .fold(
                || HashMap::new(),
                |mut acc, ann| {
                    acc.entry(ann.id).or_insert_with(Vec::new).push(ann);
                    acc
                },
            )
            .reduce(
                || HashMap::new(),
                |mut acc, map| {
                    map.into_iter().for_each(|(k, v)| {
                        acc.entry(k).or_insert_with(Vec::new).extend(v);
                    });
                    acc
                },
            );

        image_map
            .par_iter()
            .progress_count(image_map.len() as u64)
            .map(|(&id, &image)| {
                (
                    id,
                    IDMapEntry {
                        id,
                        image,
                        annotations: annotation_map.get(&id).cloned().unwrap_or_default(),
                    },
                )
            })
            .collect()
    }
}
