use std::collections::HashMap;
use chrono::{DateTime, Utc};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoFile {
    pub info: CocoInfo,
    pub images: Vec<CocoImage>,
    pub annotations: Vec<CocoAnnotation>,
    pub categories: Option<Vec<CocoCategory>>,
    pub licenses: Vec<CocoLicense>,
}

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
    pub id: i64,
    pub width: u32,
    pub height: u32,
    pub file_name: String,
    pub license: i32,
    pub flickr_url: String,
    pub coco_url: String,
    pub date_captured: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoLicense {
    pub id: i32,
    pub name: String,
    pub url: String,
}

// annotation types ///////////////////////////////////

#[derive(Serialize, Deserialize, Clone)]
pub enum CocoAnnotation {
    ObjectDetection(CocoObjectDetectionAnnotation),
    KeypointDetection(CocoKeypointDetectionAnnotation),
    PanopticSegmentation(CocoPanopticSegmentationAnnotation),
    ImageCaptioning(CocoImageCaptioningAnnotation),
}

impl CocoAnnotation {
    pub fn image_id(&self) -> i64 {
        match self {
            CocoAnnotation::ObjectDetection(ann) => ann.image_id,
            CocoAnnotation::KeypointDetection(ann) => ann.image_id,
            CocoAnnotation::PanopticSegmentation(ann) => ann.image_id,
            CocoAnnotation::ImageCaptioning(ann) => ann.image_id,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoObjectDetectionAnnotation {
    pub id: i64,
    pub image_id: i64,
    pub category_id: i32,
    pub segmentation: CocoSegmentation,
    pub area: f32,
    pub bbox: [f32; 4],

    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub iscrowd: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoKeypointDetectionAnnotation {
    pub id: i64,
    pub image_id: i64,
    pub category_id: i32,
    pub segmentation: CocoSegmentation,
    pub area: f32,
    pub bbox: [f32; 4],

    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub iscrowd: bool,
    pub keypoints: Vec<f32>, // [x1, y1, v1, x2, y2, v2, ..., xn, yn, vn]
    pub num_keypoints: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoPanopticSegmentationAnnotation {
    pub image_id: i64,
    pub file_name: String,
    pub segments_info: Vec<CocoPanopticSegmentInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoImageCaptioningAnnotation {
    pub id: i64,
    pub image_id: i64,
    pub caption: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoDensePoseAnnotation {
    pub id: i64,
    pub image_id: i64,
    pub category_id: i32,

    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub iscrowd: bool,

    pub area: u32,
    pub bbox: [f32; 4],

    #[serde(rename = "dp_I")]
    pub dp_i: Vec<f32>,

    #[serde(rename = "dp_U")]
    pub dp_u: Vec<f32>,

    #[serde(rename = "dp_V")]
    pub dp_v: Vec<f32>,


    pub dp_x: Vec<f32>,
    pub dp_y: Vec<f32>,

    pub dp_masks: Vec<CocoRLE>,
}

// category types ///////////////////////////////////

#[derive(Serialize, Deserialize, Clone)]
pub enum CocoCategory {
    ObjectDetection(CocoObjectDetectionCategory),
    KeypointDetection(CocoKeypointDetectionCategory),
    PanopticSegmentation(CocoPanopticSegmentationCategory),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoObjectDetectionCategory { // also used for dense pose
    pub id: i32,
    pub name: String,
    pub supercategory: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoKeypointDetectionCategory {
    pub id: i32,
    pub name: String,
    pub supercategory: String,
    pub keypoints: Vec<String>,
    pub skeleton: Vec<[u32; 2]>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoPanopticSegmentationCategory {
    pub id: i32,
    pub name: String,
    pub supercategory: String,
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub isthing: bool,
    pub color: [u8; 3],
}


// special types ///////////////////////////////////

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoPanopticSegmentInfo {
    pub id: i64,
    pub category_id: i32,
    pub area: u32,
    pub bbox: [f32; 4],
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub iscrowd: bool,
}


#[derive(Serialize, Deserialize, Clone)]
pub enum CocoSegmentation {
    Polygon(Vec<CocoPolygon>),
    RLE(CocoRLE),
}

// Each polygon is a vector of [x1, y1, x2, y2, ..., xn, yn]
type CocoPolygon = Vec<f32>; 
 
// Run-length encoding for masks
#[derive(Serialize, Deserialize, Clone)]
pub struct CocoRLE {
    pub counts: Vec<u32>,
    pub size: (u32, u32),
}




// Methods for CocoFile ///////////////////////////////////


pub struct IDMapEntry<'a> {
    pub id: i64,
    pub image: &'a CocoImage,
    pub annotations: Vec<&'a CocoAnnotation>,
}

impl CocoFile {
    pub fn make_id_map(&self) -> std::collections::HashMap<i64, IDMapEntry<'_>> {
        let image_map: HashMap<i64, &CocoImage> = self
            .images
            .par_iter()
            .progress()
            .map(|im| (im.id, im))
            .collect();
        let annotation_map: HashMap<i64, Vec<&CocoAnnotation>> = self
            .annotations
            .par_iter()
            .progress()
            .fold(
                || HashMap::new(),
                |mut acc, ann| {
                    acc.entry(ann.image_id()).or_insert_with(Vec::new).push(ann);
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




fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let v = i64::deserialize(deserializer)?;
    match v {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(serde::de::Error::custom(format!("invalid bool value: {}", v))),
    }
}

fn bool_to_int<S>(b: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(if *b { 1 } else { 0 })
}
