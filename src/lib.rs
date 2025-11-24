use chrono::{DateTime, Utc};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoFile {
    pub info: CocoInfo,
    pub images: Vec<CocoImage>,
    pub annotations: Vec<CocoAnnotation>,

    #[serde(skip_serializing_if = "Option::is_none")]
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

impl HasID<i64> for CocoImage {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_id(&mut self, new_id: i64) {
        self.id = new_id;
    }
}

#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct CocoLicense {
    pub id: i32,
    pub name: String,
    pub url: String,
}

impl PartialEq for CocoLicense {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.url == other.url
    }
}

impl Eq for CocoLicense {}

impl HasID<i32> for CocoLicense {
    fn id(&self) -> i32 {
        self.id
    }

    fn set_id(&mut self, new_id: i32) {
        self.id = new_id;
    }
}

// annotation types ///////////////////////////////////

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CocoAnnotation {
    KeypointDetection(CocoKeypointDetectionAnnotation),
    PanopticSegmentation(CocoPanopticSegmentationAnnotation),
    ImageCaptioning(CocoImageCaptioningAnnotation),
    ObjectDetection(CocoObjectDetectionAnnotation),
    DensePose(CocoDensePoseAnnotation),
}

impl CocoAnnotation {
    pub fn image_id(&self) -> i64 {
        match self {
            CocoAnnotation::ObjectDetection(ann) => ann.image_id,
            CocoAnnotation::KeypointDetection(ann) => ann.image_id,
            CocoAnnotation::PanopticSegmentation(ann) => ann.image_id,
            CocoAnnotation::ImageCaptioning(ann) => ann.image_id,
            CocoAnnotation::DensePose(ann) => ann.image_id,
        }
    }

    pub fn set_image_id(&mut self, new_image_id: i64) {
        match self {
            CocoAnnotation::ObjectDetection(ann) => ann.image_id = new_image_id,
            CocoAnnotation::KeypointDetection(ann) => ann.image_id = new_image_id,
            CocoAnnotation::PanopticSegmentation(ann) => ann.image_id = new_image_id,
            CocoAnnotation::ImageCaptioning(ann) => ann.image_id = new_image_id,
            CocoAnnotation::DensePose(ann) => ann.image_id = new_image_id,
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

    #[serde(deserialize_with = "bool_from_int", serialize_with = "bool_to_int")]
    pub iscrowd: bool,
}

impl HasID<i64> for CocoObjectDetectionAnnotation {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_id(&mut self, new_id: i64) {
        self.id = new_id;
    }
}

impl HasCategoryID for CocoObjectDetectionAnnotation {
    fn category_id(&self) -> i32 {
        self.category_id
    }

    fn set_category_id(&mut self, new_category_id: i32) {
        self.category_id = new_category_id;
    }
} 

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoKeypointDetectionAnnotation {
    pub id: i64,
    pub image_id: i64,
    pub category_id: i32,
    pub segmentation: CocoSegmentation,
    pub area: f32,
    pub bbox: [f32; 4],

    #[serde(deserialize_with = "bool_from_int", serialize_with = "bool_to_int")]
    pub iscrowd: bool,
    pub keypoints: Vec<f32>, // [x1, y1, v1, x2, y2, v2, ..., xn, yn, vn]
    pub num_keypoints: u32,
}

impl HasID<i64> for CocoKeypointDetectionAnnotation {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_id(&mut self, new_id: i64) {
        self.id = new_id;
    }
}

impl HasCategoryID for CocoKeypointDetectionAnnotation {
    fn category_id(&self) -> i32 {
        self.category_id
    }

    fn set_category_id(&mut self, new_category_id: i32) {
        self.category_id = new_category_id;
    }
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

impl HasID<i64> for CocoImageCaptioningAnnotation {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_id(&mut self, new_id: i64) {
        self.id = new_id;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoDensePoseAnnotation {
    pub id: i64,
    pub image_id: i64,
    
    /// uses object detection categories
    pub category_id: i32, 

    #[serde(deserialize_with = "bool_from_int", serialize_with = "bool_to_int")]
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

impl HasID<i64> for CocoDensePoseAnnotation {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_id(&mut self, new_id: i64) {
        self.id = new_id;
    }
}

impl HasCategoryID for CocoDensePoseAnnotation {
    fn category_id(&self) -> i32 {
        self.category_id
    }

    fn set_category_id(&mut self, new_category_id: i32) {
        self.category_id = new_category_id;
    }
}

// category types ///////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum CocoCategory {
    KeypointDetection(CocoKeypointDetectionCategory),
    PanopticSegmentation(CocoPanopticSegmentationCategory),
    ObjectDetection(CocoObjectDetectionCategory),
}

impl HasID<i32> for CocoCategory {
    fn id(&self) -> i32 {
        match self {
            CocoCategory::ObjectDetection(cat) => cat.id(),
            CocoCategory::KeypointDetection(cat) => cat.id(),
            CocoCategory::PanopticSegmentation(cat) => cat.id(),
        }
    }

    fn set_id(&mut self, new_id: i32) {
        match self {
            CocoCategory::ObjectDetection(cat) => cat.set_id(new_id),
            CocoCategory::KeypointDetection(cat) => cat.set_id(new_id),
            CocoCategory::PanopticSegmentation(cat) => cat.set_id(new_id),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct CocoObjectDetectionCategory {
    // also used for dense pose
    pub id: i32,
    pub name: String,
    pub supercategory: String,
}

impl PartialEq for CocoObjectDetectionCategory {
    fn eq(&self, other: &Self) -> bool {
        self.supercategory == other.supercategory && self.name == other.name
    }
}

impl Eq for CocoObjectDetectionCategory {}

impl HasID<i32> for CocoObjectDetectionCategory {
    fn id(&self) -> i32 {
        self.id
    }

    fn set_id(&mut self, new_id: i32) {
        self.id = new_id;
    }
}

#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct CocoKeypointDetectionCategory {
    pub id: i32,
    pub name: String,
    pub supercategory: String,
    pub keypoints: Vec<String>,
    pub skeleton: Vec<[u32; 2]>,
}

impl PartialEq for CocoKeypointDetectionCategory {
    fn eq(&self, other: &Self) -> bool {
        let names_match = self.supercategory == other.supercategory && self.name == other.name;
        let keypoints_match = self.keypoints == other.keypoints;
        let skeleton_match = self.skeleton == other.skeleton;
        names_match && keypoints_match && skeleton_match
    }
}

impl Eq for CocoKeypointDetectionCategory {}

impl HasID<i32> for CocoKeypointDetectionCategory {
    fn id(&self) -> i32 {
        self.id
    }

    fn set_id(&mut self, new_id: i32) {
        self.id = new_id;
    }
}

#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct CocoPanopticSegmentationCategory {
    pub id: i32,
    pub name: String,
    pub supercategory: String,
    #[serde(deserialize_with = "bool_from_int", serialize_with = "bool_to_int")]
    pub isthing: bool,
    pub color: [u8; 3],
}

impl PartialEq for CocoPanopticSegmentationCategory {
    fn eq(&self, other: &Self) -> bool {
        let names_match = self.supercategory == other.supercategory && self.name == other.name;
        let isthing_match = self.isthing == other.isthing;
        let color_match = self.color == other.color;
        names_match && isthing_match && color_match
    }
}

impl Eq for CocoPanopticSegmentationCategory {}

impl HasID<i32> for CocoPanopticSegmentationCategory {
    fn id(&self) -> i32 {
        self.id
    }

    fn set_id(&mut self, new_id: i32) {
        self.id = new_id;
    }
}

// special types ///////////////////////////////////

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoPanopticSegmentInfo {
    pub id: i64,
    pub category_id: i32,
    pub area: u32,
    pub bbox: [f32; 4],
    #[serde(deserialize_with = "bool_from_int", serialize_with = "bool_to_int")]
    pub iscrowd: bool,
}

impl HasID<i64> for CocoPanopticSegmentInfo {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_id(&mut self, new_id: i64) {
        self.id = new_id;
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CocoSegmentation {
    RLE(CocoRLE),
    Polygon(Vec<CocoPolygon>),
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
        _ => Err(serde::de::Error::custom(format!(
            "invalid bool value: {}",
            v
        ))),
    }
}

fn bool_to_int<S>(b: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(if *b { 1 } else { 0 })
}

// Traits ///////////////////////////////////

pub trait HasID<T> {
    fn id(&self) -> T;
    fn set_id(&mut self, new_id: T);
}

pub trait HasCategoryID {
    fn category_id(&self) -> i32;
    fn set_category_id(&mut self, new_category_id: i32);
}


// Tests ///////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_coco_info_serde() {
        let json = r#"{
            "year": 2020,
            "version": "1.0",
            "description": "COCO 2020 Dataset",
            "contributor": "COCO Consortium",
            "url": "http://cocodataset.org",
            "date_created": "2020-01-01T00:00:00Z"
        }"#;

        let info: CocoInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.year, 2020);
        assert_eq!(info.version, "1.0");
        assert_eq!(info.description, "COCO 2020 Dataset");

        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: CocoInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.year, info.year);
    }

    #[test]
    fn test_coco_license_serde() {
        let json = r#"{
            "id": 1,
            "name": "Attribution License",
            "url": "http://creativecommons.org/licenses/by/2.0/"
        }"#;

        let license: CocoLicense = serde_json::from_str(json).unwrap();
        assert_eq!(license.id, 1);
        assert_eq!(license.name, "Attribution License");
        assert_eq!(license.url, "http://creativecommons.org/licenses/by/2.0/");
    }

    #[test]
    fn test_coco_image_serde() {
        let json = r#"{
            "id": 42,
            "width": 640,
            "height": 480,
            "file_name": "test.jpg",
            "license": 1,
            "flickr_url": "http://flickr.com/photo/123",
            "coco_url": "http://images.cocodataset.org/test.jpg",
            "date_captured": "2020-01-15T12:30:00Z"
        }"#;

        let image: CocoImage = serde_json::from_str(json).unwrap();
        assert_eq!(image.id, 42);
        assert_eq!(image.width, 640);
        assert_eq!(image.height, 480);
        assert_eq!(image.file_name, "test.jpg");
        assert_eq!(image.license, 1);
    }

    #[test]
    fn test_object_detection_category() {
        let json = r#"{
            "id": 1,
            "name": "person",
            "supercategory": "human"
        }"#;

        let category: CocoCategory = serde_json::from_str(json).unwrap();
        match category {
            CocoCategory::ObjectDetection(cat) => {
                assert_eq!(cat.id, 1);
                assert_eq!(cat.name, "person");
                assert_eq!(cat.supercategory, "human");
            }
            _ => panic!("Expected ObjectDetection category"),
        }
    }

    #[test]
    fn test_keypoint_detection_category() {
        let json = r#"{
            "id": 1,
            "name": "person",
            "supercategory": "human",
            "keypoints": ["nose", "left_eye", "right_eye"],
            "skeleton": [[0, 1], [0, 2]]
        }"#;

        let category: CocoCategory = serde_json::from_str(json).unwrap();
        match category {
            CocoCategory::KeypointDetection(cat) => {
                assert_eq!(cat.id, 1);
                assert_eq!(cat.name, "person");
                assert_eq!(cat.keypoints.len(), 3);
                assert_eq!(cat.skeleton.len(), 2);
                assert_eq!(cat.keypoints[0], "nose");
            }
            _ => panic!("Expected KeypointDetection category"),
        }
    }

    #[test]
    fn test_panoptic_segmentation_category() {
        let json = r#"{
            "id": 1,
            "name": "person",
            "supercategory": "human",
            "isthing": 1,
            "color": [255, 128, 0]
        }"#;

        let category: CocoCategory = serde_json::from_str(json).unwrap();
        match category {
            CocoCategory::PanopticSegmentation(cat) => {
                assert_eq!(cat.id, 1);
                assert_eq!(cat.name, "person");
                assert_eq!(cat.isthing, true);
                assert_eq!(cat.color, [255, 128, 0]);
            }
            _ => panic!("Expected PanopticSegmentation category"),
        }
    }

    #[test]
    fn test_object_detection_annotation() {
        let json = r#"{
            "id": 1,
            "image_id": 42,
            "category_id": 1,
            "segmentation": [[10.0, 10.0, 20.0, 10.0, 20.0, 20.0, 10.0, 20.0]],
            "area": 100.0,
            "bbox": [10.0, 10.0, 10.0, 10.0],
            "iscrowd": 0
        }"#;

        let annotation: CocoAnnotation = serde_json::from_str(json).unwrap();
        match annotation {
            CocoAnnotation::ObjectDetection(ann) => {
                assert_eq!(ann.id, 1);
                assert_eq!(ann.image_id, 42);
                assert_eq!(ann.category_id, 1);
                assert_eq!(ann.area, 100.0);
                assert_eq!(ann.bbox, [10.0, 10.0, 10.0, 10.0]);
                assert_eq!(ann.iscrowd, false);
            }
            _ => panic!("Expected ObjectDetection annotation"),
        }
    }

    #[test]
    fn test_keypoint_detection_annotation() {
        let json = r#"{
            "id": 1,
            "image_id": 42,
            "category_id": 1,
            "segmentation": [[10.0, 10.0, 20.0, 10.0, 20.0, 20.0]],
            "area": 100.0,
            "bbox": [10.0, 10.0, 10.0, 10.0],
            "iscrowd": 0,
            "keypoints": [15.0, 15.0, 2.0, 18.0, 12.0, 2.0],
            "num_keypoints": 2
        }"#;

        let annotation: CocoAnnotation = serde_json::from_str(json).unwrap();
        match annotation {
            CocoAnnotation::KeypointDetection(ann) => {
                assert_eq!(ann.id, 1);
                assert_eq!(ann.image_id, 42);
                assert_eq!(ann.num_keypoints, 2);
                assert_eq!(ann.keypoints.len(), 6);
                assert_eq!(ann.keypoints[0], 15.0);
            }
            _ => panic!("Expected KeypointDetection annotation"),
        }
    }

    #[test]
    fn test_panoptic_segmentation_annotation() {
        let json = r#"{
            "image_id": 42,
            "file_name": "segmentation_42.png",
            "segments_info": [
                {
                    "id": 1,
                    "category_id": 1,
                    "area": 100,
                    "bbox": [10.0, 10.0, 10.0, 10.0],
                    "iscrowd": 0
                }
            ]
        }"#;

        let annotation: CocoAnnotation = serde_json::from_str(json).unwrap();
        match annotation {
            CocoAnnotation::PanopticSegmentation(ann) => {
                assert_eq!(ann.image_id, 42);
                assert_eq!(ann.file_name, "segmentation_42.png");
                assert_eq!(ann.segments_info.len(), 1);
                assert_eq!(ann.segments_info[0].id, 1);
            }
            _ => panic!("Expected PanopticSegmentation annotation"),
        }
    }

    #[test]
    fn test_image_captioning_annotation() {
        let json = r#"{
            "id": 1,
            "image_id": 42,
            "caption": "A person riding a bicycle"
        }"#;

        let annotation: CocoAnnotation = serde_json::from_str(json).unwrap();
        match annotation {
            CocoAnnotation::ImageCaptioning(ann) => {
                assert_eq!(ann.id, 1);
                assert_eq!(ann.image_id, 42);
                assert_eq!(ann.caption, "A person riding a bicycle");
            }
            _ => panic!("Expected ImageCaptioning annotation"),
        }
    }

    #[test]
    fn test_polygon_segmentation() {
        let json = r#"[[10.0, 10.0, 20.0, 10.0, 20.0, 20.0, 10.0, 20.0]]"#;
        let segmentation: CocoSegmentation = serde_json::from_str(json).unwrap();

        match segmentation {
            CocoSegmentation::Polygon(polygons) => {
                assert_eq!(polygons.len(), 1);
                assert_eq!(polygons[0].len(), 8);
                assert_eq!(polygons[0][0], 10.0);
                assert_eq!(polygons[0][1], 10.0);
            }
            _ => panic!("Expected Polygon segmentation"),
        }
    }

    #[test]
    fn test_rle_segmentation() {
        let json = r#"{
            "counts": [100, 50, 100, 50],
            "size": [640, 480]
        }"#;
        let segmentation: CocoSegmentation = serde_json::from_str(json).unwrap();

        match segmentation {
            CocoSegmentation::RLE(rle) => {
                assert_eq!(rle.counts.len(), 4);
                assert_eq!(rle.counts[0], 100);
                assert_eq!(rle.size, (640, 480));
            }
            _ => panic!("Expected RLE segmentation"),
        }
    }

    #[test]
    fn test_coco_rle() {
        let json = r#"{
            "counts": [100, 50, 100],
            "size": [640, 480]
        }"#;

        let rle: CocoRLE = serde_json::from_str(json).unwrap();
        assert_eq!(rle.counts, vec![100, 50, 100]);
        assert_eq!(rle.size, (640, 480));
    }

    #[test]
    fn test_panoptic_segment_info() {
        let json = r#"{
            "id": 1,
            "category_id": 5,
            "area": 1500,
            "bbox": [100.0, 100.0, 50.0, 30.0],
            "iscrowd": 1
        }"#;

        let segment: CocoPanopticSegmentInfo = serde_json::from_str(json).unwrap();
        assert_eq!(segment.id, 1);
        assert_eq!(segment.category_id, 5);
        assert_eq!(segment.area, 1500);
        assert_eq!(segment.iscrowd, true);
    }

    #[test]
    fn test_bool_from_int_zero() {
        let json = r#"{"iscrowd": 0}"#;
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "bool_from_int")]
            #[allow(dead_code)]
            iscrowd: bool,
        }
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.iscrowd, false);
    }

    #[test]
    fn test_bool_from_int_one() {
        let json = r#"{"iscrowd": 1}"#;
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "bool_from_int")]
            #[allow(dead_code)]
            iscrowd: bool,
        }
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.iscrowd, true);
    }

    #[test]
    fn test_bool_from_int_invalid() {
        let json = r#"{"iscrowd": 2}"#;
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "bool_from_int")]
            #[allow(dead_code)]
            iscrowd: bool,
        }
        let result: Result<Test, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_bool_to_int() {
        #[derive(Serialize)]
        struct Test {
            #[serde(serialize_with = "bool_to_int")]
            iscrowd: bool,
        }

        let test_false = Test { iscrowd: false };
        let json = serde_json::to_string(&test_false).unwrap();
        assert!(json.contains("\"iscrowd\":0"));

        let test_true = Test { iscrowd: true };
        let json = serde_json::to_string(&test_true).unwrap();
        assert!(json.contains("\"iscrowd\":1"));
    }

    #[test]
    fn test_minimal_coco_file() {
        let json = r#"{
            "info": {
                "year": 2020,
                "version": "1.0",
                "description": "Test Dataset",
                "contributor": "Test",
                "url": "http://test.com",
                "date_created": "2020-01-01T00:00:00Z"
            },
            "licenses": [],
            "images": [
                {
                    "id": 1,
                    "width": 640,
                    "height": 480,
                    "file_name": "test.jpg",
                    "license": 0,
                    "flickr_url": "",
                    "coco_url": "",
                    "date_captured": "2020-01-01T00:00:00Z"
                }
            ],
            "annotations": [
                {
                    "id": 1,
                    "image_id": 1,
                    "caption": "A test image"
                }
            ],
            "categories": [
                {
                    "id": 1,
                    "name": "test",
                    "supercategory": ""
                }
            ]
        }"#;

        let coco_file: CocoFile = serde_json::from_str(json).unwrap();
        assert_eq!(coco_file.info.year, 2020);
        assert_eq!(coco_file.images.len(), 1);
        assert_eq!(coco_file.annotations.len(), 1);
        assert_eq!(coco_file.licenses.len(), 0);
        assert!(coco_file.categories.is_some());
        assert_eq!(coco_file.categories.unwrap().len(), 1);
    }

    #[test]
    fn test_annotation_image_id() {
        let obj_det = CocoAnnotation::ObjectDetection(CocoObjectDetectionAnnotation {
            id: 1,
            image_id: 42,
            category_id: 1,
            segmentation: CocoSegmentation::Polygon(vec![vec![0.0, 0.0, 1.0, 1.0]]),
            area: 1.0,
            bbox: [0.0, 0.0, 1.0, 1.0],
            iscrowd: false,
        });
        assert_eq!(obj_det.image_id(), 42);

        let captioning = CocoAnnotation::ImageCaptioning(CocoImageCaptioningAnnotation {
            id: 2,
            image_id: 99,
            caption: "test".to_string(),
        });
        assert_eq!(captioning.image_id(), 99);
    }

    #[test]
    fn test_make_id_map() {
        let coco_file = CocoFile {
            info: CocoInfo {
                year: 2020,
                version: "1.0".to_string(),
                description: "Test".to_string(),
                contributor: "Test".to_string(),
                url: "http://test.com".to_string(),
                date_created: Utc::now(),
            },
            licenses: vec![],
            images: vec![
                CocoImage {
                    id: 1,
                    width: 640,
                    height: 480,
                    file_name: "test1.jpg".to_string(),
                    license: 0,
                    flickr_url: "".to_string(),
                    coco_url: "".to_string(),
                    date_captured: Utc::now(),
                },
                CocoImage {
                    id: 2,
                    width: 800,
                    height: 600,
                    file_name: "test2.jpg".to_string(),
                    license: 0,
                    flickr_url: "".to_string(),
                    coco_url: "".to_string(),
                    date_captured: Utc::now(),
                },
            ],
            annotations: vec![
                CocoAnnotation::ImageCaptioning(CocoImageCaptioningAnnotation {
                    id: 1,
                    image_id: 1,
                    caption: "First image".to_string(),
                }),
                CocoAnnotation::ImageCaptioning(CocoImageCaptioningAnnotation {
                    id: 2,
                    image_id: 1,
                    caption: "First image alt".to_string(),
                }),
                CocoAnnotation::ImageCaptioning(CocoImageCaptioningAnnotation {
                    id: 3,
                    image_id: 2,
                    caption: "Second image".to_string(),
                }),
            ],
            categories: None,
        };

        let id_map = coco_file.make_id_map();

        assert_eq!(id_map.len(), 2);

        let entry1 = id_map.get(&1).unwrap();
        assert_eq!(entry1.id, 1);
        assert_eq!(entry1.image.file_name, "test1.jpg");
        assert_eq!(entry1.annotations.len(), 2);

        let entry2 = id_map.get(&2).unwrap();
        assert_eq!(entry2.id, 2);
        assert_eq!(entry2.image.file_name, "test2.jpg");
        assert_eq!(entry2.annotations.len(), 1);
    }

    #[test]
    fn test_coco_file_without_categories() {
        let json = r#"{
            "info": {
                "year": 2020,
                "version": "1.0",
                "description": "Test",
                "contributor": "Test",
                "url": "http://test.com",
                "date_created": "2020-01-01T00:00:00Z"
            },
            "licenses": [],
            "images": [],
            "annotations": []
        }"#;

        let coco_file: CocoFile = serde_json::from_str(json).unwrap();
        assert!(coco_file.categories.is_none());
    }
}
