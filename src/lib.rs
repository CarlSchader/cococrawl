use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CocoInfo {
    pub year: i32,
    pub version: String,
    pub description: String,
    pub contributor: String,
    pub url: String,
    pub date_created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub struct CocoAnnotation {}

#[derive(Serialize, Deserialize)]
pub struct CocoFile {
    pub info: CocoInfo,
    pub images: Vec<CocoImage>,
    pub annotations: Vec<CocoAnnotation>,
}
