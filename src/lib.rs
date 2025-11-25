use chrono::{DateTime, Utc};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct CocoFile {
    pub images: Vec<CocoImage>,
    pub annotations: Vec<CocoAnnotation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<CocoInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<CocoCategory>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub licenses: Option<Vec<CocoLicense>>,
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flickr_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub coco_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_captured: Option<DateTime<Utc>>,
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
    /// image id
    pub id: i64,
    pub image: &'a CocoImage,
    pub annotations: Vec<&'a CocoAnnotation>,
}

impl CocoFile {
    pub fn make_image_id_map(&self) -> std::collections::HashMap<i64, IDMapEntry<'_>> {
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
        assert_eq!(image.license, Some(1));
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
        assert_eq!(coco_file.info.unwrap().year, 2020);
        assert_eq!(coco_file.images.len(), 1);
        assert_eq!(coco_file.annotations.len(), 1);
        assert_eq!(coco_file.licenses.unwrap().len(), 0);
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
            info: Some(CocoInfo {
                year: 2020,
                version: "1.0".to_string(),
                description: "Test".to_string(),
                contributor: "Test".to_string(),
                url: "http://test.com".to_string(),
                date_created: Utc::now(),
            }),
            licenses: Some(vec![]),
            images: vec![
                CocoImage {
                    id: 1,
                    width: 640,
                    height: 480,
                    file_name: "test1.jpg".to_string(),
                    license: Some(0),
                    flickr_url: Some("".to_string()),
                    coco_url: Some("".to_string()),
                    date_captured: Some(Utc::now()),
                },
                CocoImage {
                    id: 2,
                    width: 800,
                    height: 600,
                    file_name: "test2.jpg".to_string(),
                    license: Some(0),
                    flickr_url: Some("".to_string()),
                    coco_url: Some("".to_string()),
                    date_captured: Some(Utc::now()),
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

        let id_map = coco_file.make_image_id_map();

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

    // ========== DENSEPOSE ANNOTATION TESTS ==========

    #[test]
    fn test_densepose_annotation_serde() {
        let json = r#"{
            "id": 1,
            "image_id": 42,
            "category_id": 1,
            "iscrowd": 0,
            "area": 1500,
            "bbox": [100.0, 100.0, 50.0, 30.0],
            "dp_I": [1.0, 2.0, 3.0],
            "dp_U": [0.5, 0.6, 0.7],
            "dp_V": [0.8, 0.9, 1.0],
            "dp_x": [10.0, 20.0, 30.0],
            "dp_y": [15.0, 25.0, 35.0],
            "dp_masks": [
                {
                    "counts": [100, 50],
                    "size": [640, 480]
                }
            ]
        }"#;

        let annotation: CocoAnnotation = serde_json::from_str(json).unwrap();
        match annotation {
            CocoAnnotation::DensePose(ann) => {
                assert_eq!(ann.id, 1);
                assert_eq!(ann.image_id, 42);
                assert_eq!(ann.category_id, 1);
                assert_eq!(ann.iscrowd, false);
                assert_eq!(ann.area, 1500);
                assert_eq!(ann.bbox, [100.0, 100.0, 50.0, 30.0]);
                assert_eq!(ann.dp_i, vec![1.0, 2.0, 3.0]);
                assert_eq!(ann.dp_u, vec![0.5, 0.6, 0.7]);
                assert_eq!(ann.dp_v, vec![0.8, 0.9, 1.0]);
                assert_eq!(ann.dp_x, vec![10.0, 20.0, 30.0]);
                assert_eq!(ann.dp_y, vec![15.0, 25.0, 35.0]);
                assert_eq!(ann.dp_masks.len(), 1);
                assert_eq!(ann.dp_masks[0].counts, vec![100, 50]);
            }
            _ => panic!("Expected DensePose annotation"),
        }
    }

    #[test]
    fn test_densepose_annotation_roundtrip() {
        let ann = CocoDensePoseAnnotation {
            id: 123,
            image_id: 456,
            category_id: 7,
            iscrowd: true,
            area: 2000,
            bbox: [10.0, 20.0, 30.0, 40.0],
            dp_i: vec![1.0, 2.0],
            dp_u: vec![0.1, 0.2],
            dp_v: vec![0.3, 0.4],
            dp_x: vec![5.0, 6.0],
            dp_y: vec![7.0, 8.0],
            dp_masks: vec![CocoRLE {
                counts: vec![10, 20, 30],
                size: (100, 200),
            }],
        };

        let serialized = serde_json::to_string(&ann).unwrap();
        let deserialized: CocoDensePoseAnnotation = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, 123);
        assert_eq!(deserialized.image_id, 456);
        assert_eq!(deserialized.category_id, 7);
        assert_eq!(deserialized.iscrowd, true);
        assert_eq!(deserialized.dp_i, vec![1.0, 2.0]);
        assert_eq!(deserialized.dp_masks[0].counts, vec![10, 20, 30]);
    }

    #[test]
    fn test_densepose_has_id() {
        let mut ann = CocoDensePoseAnnotation {
            id: 100,
            image_id: 200,
            category_id: 1,
            iscrowd: false,
            area: 500,
            bbox: [0.0, 0.0, 10.0, 10.0],
            dp_i: vec![],
            dp_u: vec![],
            dp_v: vec![],
            dp_x: vec![],
            dp_y: vec![],
            dp_masks: vec![],
        };

        assert_eq!(ann.id(), 100);
        ann.set_id(999);
        assert_eq!(ann.id(), 999);
    }

    #[test]
    fn test_densepose_has_category_id() {
        let mut ann = CocoDensePoseAnnotation {
            id: 1,
            image_id: 2,
            category_id: 5,
            iscrowd: false,
            area: 100,
            bbox: [0.0, 0.0, 1.0, 1.0],
            dp_i: vec![],
            dp_u: vec![],
            dp_v: vec![],
            dp_x: vec![],
            dp_y: vec![],
            dp_masks: vec![],
        };

        assert_eq!(ann.category_id(), 5);
        ann.set_category_id(10);
        assert_eq!(ann.category_id(), 10);
    }

    #[test]
    fn test_densepose_annotation_image_id() {
        let mut ann = CocoAnnotation::DensePose(CocoDensePoseAnnotation {
            id: 1,
            image_id: 50,
            category_id: 1,
            iscrowd: false,
            area: 100,
            bbox: [0.0, 0.0, 1.0, 1.0],
            dp_i: vec![],
            dp_u: vec![],
            dp_v: vec![],
            dp_x: vec![],
            dp_y: vec![],
            dp_masks: vec![],
        });

        assert_eq!(ann.image_id(), 50);
        ann.set_image_id(75);
        assert_eq!(ann.image_id(), 75);
    }

    // ========== TRAIT SETTER TESTS ==========

    #[test]
    fn test_coco_image_set_id() {
        let mut image = CocoImage {
            id: 10,
            width: 100,
            height: 200,
            file_name: "test.jpg".to_string(),
            license: None,
            flickr_url: None,
            coco_url: None,
            date_captured: None,
        };

        assert_eq!(image.id(), 10);
        image.set_id(999);
        assert_eq!(image.id(), 999);
        assert_eq!(image.id, 999);
    }

    #[test]
    fn test_coco_license_set_id() {
        let mut license = CocoLicense {
            id: 1,
            name: "MIT".to_string(),
            url: "http://mit.edu".to_string(),
        };

        assert_eq!(license.id(), 1);
        license.set_id(50);
        assert_eq!(license.id(), 50);
        assert_eq!(license.id, 50);
    }

    #[test]
    fn test_object_detection_annotation_setters() {
        let mut ann = CocoObjectDetectionAnnotation {
            id: 1,
            image_id: 10,
            category_id: 5,
            segmentation: CocoSegmentation::Polygon(vec![vec![0.0, 0.0, 1.0, 1.0]]),
            area: 100.0,
            bbox: [0.0, 0.0, 10.0, 10.0],
            iscrowd: false,
        };

        assert_eq!(ann.id(), 1);
        ann.set_id(100);
        assert_eq!(ann.id(), 100);

        assert_eq!(ann.category_id(), 5);
        ann.set_category_id(20);
        assert_eq!(ann.category_id(), 20);
    }

    #[test]
    fn test_keypoint_detection_annotation_setters() {
        let mut ann = CocoKeypointDetectionAnnotation {
            id: 2,
            image_id: 20,
            category_id: 3,
            segmentation: CocoSegmentation::Polygon(vec![]),
            area: 50.0,
            bbox: [0.0, 0.0, 5.0, 5.0],
            iscrowd: false,
            keypoints: vec![1.0, 2.0, 3.0],
            num_keypoints: 1,
        };

        assert_eq!(ann.id(), 2);
        ann.set_id(200);
        assert_eq!(ann.id(), 200);

        assert_eq!(ann.category_id(), 3);
        ann.set_category_id(30);
        assert_eq!(ann.category_id(), 30);
    }

    #[test]
    fn test_image_captioning_annotation_set_id() {
        let mut ann = CocoImageCaptioningAnnotation {
            id: 5,
            image_id: 50,
            caption: "Test caption".to_string(),
        };

        assert_eq!(ann.id(), 5);
        ann.set_id(500);
        assert_eq!(ann.id(), 500);
    }

    #[test]
    fn test_panoptic_segment_info_set_id() {
        let mut segment = CocoPanopticSegmentInfo {
            id: 10,
            category_id: 2,
            area: 100,
            bbox: [0.0, 0.0, 10.0, 10.0],
            iscrowd: false,
        };

        assert_eq!(segment.id(), 10);
        segment.set_id(1000);
        assert_eq!(segment.id(), 1000);
    }

    #[test]
    fn test_category_enum_set_id() {
        let mut cat = CocoCategory::ObjectDetection(CocoObjectDetectionCategory {
            id: 1,
            name: "person".to_string(),
            supercategory: "human".to_string(),
        });

        assert_eq!(cat.id(), 1);
        cat.set_id(99);
        assert_eq!(cat.id(), 99);
    }

    #[test]
    fn test_object_detection_category_set_id() {
        let mut cat = CocoObjectDetectionCategory {
            id: 1,
            name: "car".to_string(),
            supercategory: "vehicle".to_string(),
        };

        assert_eq!(cat.id(), 1);
        cat.set_id(42);
        assert_eq!(cat.id(), 42);
    }

    #[test]
    fn test_keypoint_detection_category_set_id() {
        let mut cat = CocoKeypointDetectionCategory {
            id: 2,
            name: "person".to_string(),
            supercategory: "human".to_string(),
            keypoints: vec!["nose".to_string()],
            skeleton: vec![[0, 1]],
        };

        assert_eq!(cat.id(), 2);
        cat.set_id(43);
        assert_eq!(cat.id(), 43);
    }

    #[test]
    fn test_panoptic_segmentation_category_set_id() {
        let mut cat = CocoPanopticSegmentationCategory {
            id: 3,
            name: "sky".to_string(),
            supercategory: "background".to_string(),
            isthing: false,
            color: [135, 206, 235],
        };

        assert_eq!(cat.id(), 3);
        cat.set_id(44);
        assert_eq!(cat.id(), 44);
    }

    // ========== SET_IMAGE_ID TESTS ==========

    #[test]
    fn test_annotation_set_image_id_all_types() {
        let mut obj_det = CocoAnnotation::ObjectDetection(CocoObjectDetectionAnnotation {
            id: 1,
            image_id: 10,
            category_id: 1,
            segmentation: CocoSegmentation::Polygon(vec![]),
            area: 1.0,
            bbox: [0.0, 0.0, 1.0, 1.0],
            iscrowd: false,
        });
        obj_det.set_image_id(100);
        assert_eq!(obj_det.image_id(), 100);

        let mut kp_det = CocoAnnotation::KeypointDetection(CocoKeypointDetectionAnnotation {
            id: 2,
            image_id: 20,
            category_id: 1,
            segmentation: CocoSegmentation::Polygon(vec![]),
            area: 1.0,
            bbox: [0.0, 0.0, 1.0, 1.0],
            iscrowd: false,
            keypoints: vec![],
            num_keypoints: 0,
        });
        kp_det.set_image_id(200);
        assert_eq!(kp_det.image_id(), 200);

        let mut panoptic = CocoAnnotation::PanopticSegmentation(
            CocoPanopticSegmentationAnnotation {
                image_id: 30,
                file_name: "test.png".to_string(),
                segments_info: vec![],
            },
        );
        panoptic.set_image_id(300);
        assert_eq!(panoptic.image_id(), 300);

        let mut caption = CocoAnnotation::ImageCaptioning(CocoImageCaptioningAnnotation {
            id: 4,
            image_id: 40,
            caption: "test".to_string(),
        });
        caption.set_image_id(400);
        assert_eq!(caption.image_id(), 400);

        let mut densepose = CocoAnnotation::DensePose(CocoDensePoseAnnotation {
            id: 5,
            image_id: 50,
            category_id: 1,
            iscrowd: false,
            area: 100,
            bbox: [0.0, 0.0, 1.0, 1.0],
            dp_i: vec![],
            dp_u: vec![],
            dp_v: vec![],
            dp_x: vec![],
            dp_y: vec![],
            dp_masks: vec![],
        });
        densepose.set_image_id(500);
        assert_eq!(densepose.image_id(), 500);
    }

    // ========== PARTIAL EQ TESTS ==========

    #[test]
    fn test_coco_license_equality() {
        let license1 = CocoLicense {
            id: 1,
            name: "MIT".to_string(),
            url: "http://mit.edu".to_string(),
        };
        let license2 = CocoLicense {
            id: 999, // Different ID
            name: "MIT".to_string(),
            url: "http://mit.edu".to_string(),
        };
        let license3 = CocoLicense {
            id: 1,
            name: "Apache".to_string(),
            url: "http://apache.org".to_string(),
        };

        // Same name and URL, different ID -> should be equal
        assert!(license1 == license2);
        // Different name -> should not be equal
        assert!(license1 != license3);
    }

    #[test]
    fn test_object_detection_category_equality() {
        let cat1 = CocoObjectDetectionCategory {
            id: 1,
            name: "person".to_string(),
            supercategory: "human".to_string(),
        };
        let cat2 = CocoObjectDetectionCategory {
            id: 999, // Different ID
            name: "person".to_string(),
            supercategory: "human".to_string(),
        };
        let cat3 = CocoObjectDetectionCategory {
            id: 1,
            name: "car".to_string(),
            supercategory: "vehicle".to_string(),
        };

        assert!(cat1 == cat2);
        assert!(cat1 != cat3);
    }

    #[test]
    fn test_keypoint_detection_category_equality() {
        let cat1 = CocoKeypointDetectionCategory {
            id: 1,
            name: "person".to_string(),
            supercategory: "human".to_string(),
            keypoints: vec!["nose".to_string(), "eye".to_string()],
            skeleton: vec![[0, 1], [1, 2]],
        };
        let cat2 = CocoKeypointDetectionCategory {
            id: 999,
            name: "person".to_string(),
            supercategory: "human".to_string(),
            keypoints: vec!["nose".to_string(), "eye".to_string()],
            skeleton: vec![[0, 1], [1, 2]],
        };
        let cat3 = CocoKeypointDetectionCategory {
            id: 1,
            name: "person".to_string(),
            supercategory: "human".to_string(),
            keypoints: vec!["nose".to_string()], // Different keypoints
            skeleton: vec![[0, 1], [1, 2]],
        };

        assert!(cat1 == cat2);
        assert!(cat1 != cat3);
    }

    #[test]
    fn test_panoptic_segmentation_category_equality() {
        let cat1 = CocoPanopticSegmentationCategory {
            id: 1,
            name: "sky".to_string(),
            supercategory: "background".to_string(),
            isthing: false,
            color: [135, 206, 235],
        };
        let cat2 = CocoPanopticSegmentationCategory {
            id: 999,
            name: "sky".to_string(),
            supercategory: "background".to_string(),
            isthing: false,
            color: [135, 206, 235],
        };
        let cat3 = CocoPanopticSegmentationCategory {
            id: 1,
            name: "sky".to_string(),
            supercategory: "background".to_string(),
            isthing: true, // Different isthing
            color: [135, 206, 235],
        };

        assert!(cat1 == cat2);
        assert!(cat1 != cat3);
    }

    // ========== OPTIONAL FIELD SERIALIZATION TESTS ==========

    #[test]
    fn test_coco_file_optional_fields_none() {
        let coco_file = CocoFile {
            info: None,
            licenses: None,
            images: vec![],
            annotations: vec![],
            categories: None,
        };

        let serialized = serde_json::to_string(&coco_file).unwrap();
        assert!(!serialized.contains("\"info\""));
        assert!(!serialized.contains("\"licenses\""));
        assert!(!serialized.contains("\"categories\""));
    }

    #[test]
    fn test_coco_image_optional_fields_none() {
        let image = CocoImage {
            id: 1,
            width: 100,
            height: 200,
            file_name: "test.jpg".to_string(),
            license: None,
            flickr_url: None,
            coco_url: None,
            date_captured: None,
        };

        let serialized = serde_json::to_string(&image).unwrap();
        assert!(!serialized.contains("\"license\""));
        assert!(!serialized.contains("\"flickr_url\""));
        assert!(!serialized.contains("\"coco_url\""));
        assert!(!serialized.contains("\"date_captured\""));
    }

    #[test]
    fn test_coco_image_optional_fields_present() {
        let image = CocoImage {
            id: 1,
            width: 100,
            height: 200,
            file_name: "test.jpg".to_string(),
            license: Some(1),
            flickr_url: Some("http://flickr.com".to_string()),
            coco_url: Some("http://coco.com".to_string()),
            date_captured: Some(Utc::now()),
        };

        let serialized = serde_json::to_string(&image).unwrap();
        assert!(serialized.contains("\"license\""));
        assert!(serialized.contains("\"flickr_url\""));
        assert!(serialized.contains("\"coco_url\""));
        assert!(serialized.contains("\"date_captured\""));
    }

    // ========== ROUNDTRIP TESTS ==========

    #[test]
    fn test_coco_file_roundtrip() {
        let original = CocoFile {
            info: Some(CocoInfo {
                year: 2020,
                version: "1.0".to_string(),
                description: "Test".to_string(),
                contributor: "Tester".to_string(),
                url: "http://test.com".to_string(),
                date_created: Utc::now(),
            }),
            licenses: Some(vec![CocoLicense {
                id: 1,
                name: "MIT".to_string(),
                url: "http://mit.edu".to_string(),
            }]),
            images: vec![CocoImage {
                id: 1,
                width: 640,
                height: 480,
                file_name: "test.jpg".to_string(),
                license: Some(1),
                flickr_url: None,
                coco_url: None,
                date_captured: None,
            }],
            annotations: vec![],
            categories: Some(vec![CocoCategory::ObjectDetection(
                CocoObjectDetectionCategory {
                    id: 1,
                    name: "person".to_string(),
                    supercategory: "human".to_string(),
                },
            )]),
        };

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: CocoFile = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.images.len(), 1);
        assert_eq!(deserialized.images[0].id, 1);
        assert_eq!(deserialized.licenses.as_ref().unwrap().len(), 1);
        assert_eq!(deserialized.categories.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_object_detection_annotation_roundtrip() {
        let original = CocoObjectDetectionAnnotation {
            id: 1,
            image_id: 10,
            category_id: 5,
            segmentation: CocoSegmentation::Polygon(vec![vec![0.0, 0.0, 10.0, 0.0, 10.0, 10.0]]),
            area: 50.0,
            bbox: [0.0, 0.0, 10.0, 10.0],
            iscrowd: true,
        };

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: CocoObjectDetectionAnnotation =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.image_id, 10);
        assert_eq!(deserialized.category_id, 5);
        assert_eq!(deserialized.iscrowd, true);
    }

    #[test]
    fn test_keypoint_detection_annotation_roundtrip() {
        let original = CocoKeypointDetectionAnnotation {
            id: 2,
            image_id: 20,
            category_id: 3,
            segmentation: CocoSegmentation::Polygon(vec![]),
            area: 100.0,
            bbox: [5.0, 5.0, 15.0, 15.0],
            iscrowd: false,
            keypoints: vec![10.0, 10.0, 2.0, 15.0, 15.0, 2.0],
            num_keypoints: 2,
        };

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: CocoKeypointDetectionAnnotation =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.num_keypoints, 2);
        assert_eq!(deserialized.keypoints.len(), 6);
    }

    // ========== MAKE_ID_MAP EDGE CASE TESTS ==========

    #[test]
    fn test_make_id_map_with_no_annotations() {
        let coco_file = CocoFile {
            info: None,
            licenses: None,
            images: vec![
                CocoImage {
                    id: 1,
                    width: 100,
                    height: 100,
                    file_name: "img1.jpg".to_string(),
                    license: None,
                    flickr_url: None,
                    coco_url: None,
                    date_captured: None,
                },
                CocoImage {
                    id: 2,
                    width: 200,
                    height: 200,
                    file_name: "img2.jpg".to_string(),
                    license: None,
                    flickr_url: None,
                    coco_url: None,
                    date_captured: None,
                },
            ],
            annotations: vec![],
            categories: None,
        };

        let id_map = coco_file.make_image_id_map();

        assert_eq!(id_map.len(), 2);
        assert_eq!(id_map.get(&1).unwrap().annotations.len(), 0);
        assert_eq!(id_map.get(&2).unwrap().annotations.len(), 0);
    }

    #[test]
    fn test_make_id_map_with_mixed_annotation_types() {
        let coco_file = CocoFile {
            info: None,
            licenses: None,
            images: vec![CocoImage {
                id: 1,
                width: 100,
                height: 100,
                file_name: "img1.jpg".to_string(),
                license: None,
                flickr_url: None,
                coco_url: None,
                date_captured: None,
            }],
            annotations: vec![
                CocoAnnotation::ObjectDetection(CocoObjectDetectionAnnotation {
                    id: 1,
                    image_id: 1,
                    category_id: 1,
                    segmentation: CocoSegmentation::Polygon(vec![]),
                    area: 1.0,
                    bbox: [0.0, 0.0, 1.0, 1.0],
                    iscrowd: false,
                }),
                CocoAnnotation::ImageCaptioning(CocoImageCaptioningAnnotation {
                    id: 2,
                    image_id: 1,
                    caption: "test".to_string(),
                }),
                CocoAnnotation::DensePose(CocoDensePoseAnnotation {
                    id: 3,
                    image_id: 1,
                    category_id: 1,
                    iscrowd: false,
                    area: 100,
                    bbox: [0.0, 0.0, 10.0, 10.0],
                    dp_i: vec![],
                    dp_u: vec![],
                    dp_v: vec![],
                    dp_x: vec![],
                    dp_y: vec![],
                    dp_masks: vec![],
                }),
            ],
            categories: None,
        };

        let id_map = coco_file.make_image_id_map();
        let entry = id_map.get(&1).unwrap();

        assert_eq!(entry.annotations.len(), 3);
    }

    #[test]
    fn test_make_id_map_empty_dataset() {
        let coco_file = CocoFile {
            info: None,
            licenses: None,
            images: vec![],
            annotations: vec![],
            categories: None,
        };

        let id_map = coco_file.make_image_id_map();
        assert_eq!(id_map.len(), 0);
    }

    // ========== EDGE CASE TESTS ==========

    #[test]
    fn test_empty_polygon_segmentation() {
        let json = r#"[]"#;
        let segmentation: CocoSegmentation = serde_json::from_str(json).unwrap();

        match segmentation {
            CocoSegmentation::Polygon(polygons) => {
                assert_eq!(polygons.len(), 0);
            }
            _ => panic!("Expected empty Polygon segmentation"),
        }
    }

    #[test]
    fn test_multiple_polygons_segmentation() {
        let json = r#"[[10.0, 10.0, 20.0, 10.0], [30.0, 30.0, 40.0, 40.0]]"#;
        let segmentation: CocoSegmentation = serde_json::from_str(json).unwrap();

        match segmentation {
            CocoSegmentation::Polygon(polygons) => {
                assert_eq!(polygons.len(), 2);
                assert_eq!(polygons[0].len(), 4);
                assert_eq!(polygons[1].len(), 4);
            }
            _ => panic!("Expected Polygon segmentation with 2 polygons"),
        }
    }

    #[test]
    fn test_empty_rle_counts() {
        let rle = CocoRLE {
            counts: vec![],
            size: (100, 100),
        };

        let serialized = serde_json::to_string(&rle).unwrap();
        let deserialized: CocoRLE = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.counts.len(), 0);
        assert_eq!(deserialized.size, (100, 100));
    }

    #[test]
    fn test_large_image_dimensions() {
        let image = CocoImage {
            id: 1,
            width: 10000,
            height: 8000,
            file_name: "huge.jpg".to_string(),
            license: None,
            flickr_url: None,
            coco_url: None,
            date_captured: None,
        };

        let serialized = serde_json::to_string(&image).unwrap();
        let deserialized: CocoImage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.width, 10000);
        assert_eq!(deserialized.height, 8000);
    }

    #[test]
    fn test_bbox_with_zeros() {
        let ann = CocoObjectDetectionAnnotation {
            id: 1,
            image_id: 1,
            category_id: 1,
            segmentation: CocoSegmentation::Polygon(vec![]),
            area: 0.0,
            bbox: [0.0, 0.0, 0.0, 0.0],
            iscrowd: false,
        };

        let serialized = serde_json::to_string(&ann).unwrap();
        let deserialized: CocoObjectDetectionAnnotation =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.bbox, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(deserialized.area, 0.0);
    }

    #[test]
    fn test_negative_bbox_values() {
        let ann = CocoObjectDetectionAnnotation {
            id: 1,
            image_id: 1,
            category_id: 1,
            segmentation: CocoSegmentation::Polygon(vec![]),
            area: 100.0,
            bbox: [-10.0, -20.0, 30.0, 40.0],
            iscrowd: false,
        };

        let serialized = serde_json::to_string(&ann).unwrap();
        let deserialized: CocoObjectDetectionAnnotation =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.bbox[0], -10.0);
        assert_eq!(deserialized.bbox[1], -20.0);
    }

    #[test]
    fn test_empty_keypoints() {
        let ann = CocoKeypointDetectionAnnotation {
            id: 1,
            image_id: 1,
            category_id: 1,
            segmentation: CocoSegmentation::Polygon(vec![]),
            area: 1.0,
            bbox: [0.0, 0.0, 1.0, 1.0],
            iscrowd: false,
            keypoints: vec![],
            num_keypoints: 0,
        };

        let serialized = serde_json::to_string(&ann).unwrap();
        let deserialized: CocoKeypointDetectionAnnotation =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.keypoints.len(), 0);
        assert_eq!(deserialized.num_keypoints, 0);
    }

    #[test]
    fn test_empty_category_strings() {
        let cat = CocoObjectDetectionCategory {
            id: 1,
            name: "".to_string(),
            supercategory: "".to_string(),
        };

        let serialized = serde_json::to_string(&cat).unwrap();
        let deserialized: CocoObjectDetectionCategory =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, "");
        assert_eq!(deserialized.supercategory, "");
    }

    #[test]
    fn test_empty_caption() {
        let ann = CocoImageCaptioningAnnotation {
            id: 1,
            image_id: 1,
            caption: "".to_string(),
        };

        let serialized = serde_json::to_string(&ann).unwrap();
        let deserialized: CocoImageCaptioningAnnotation =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.caption, "");
    }

    #[test]
    fn test_panoptic_segmentation_empty_segments_info() {
        let ann = CocoPanopticSegmentationAnnotation {
            image_id: 1,
            file_name: "seg_1.png".to_string(),
            segments_info: vec![],
        };

        let serialized = serde_json::to_string(&ann).unwrap();
        let deserialized: CocoPanopticSegmentationAnnotation =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.segments_info.len(), 0);
    }

    #[test]
    fn test_category_enum_all_variants_id_methods() {
        let obj_det = CocoCategory::ObjectDetection(CocoObjectDetectionCategory {
            id: 1,
            name: "test".to_string(),
            supercategory: "test".to_string(),
        });
        assert_eq!(obj_det.id(), 1);

        let kp_det = CocoCategory::KeypointDetection(CocoKeypointDetectionCategory {
            id: 2,
            name: "test".to_string(),
            supercategory: "test".to_string(),
            keypoints: vec![],
            skeleton: vec![],
        });
        assert_eq!(kp_det.id(), 2);

        let panoptic = CocoCategory::PanopticSegmentation(CocoPanopticSegmentationCategory {
            id: 3,
            name: "test".to_string(),
            supercategory: "test".to_string(),
            isthing: true,
            color: [0, 0, 0],
        });
        assert_eq!(panoptic.id(), 3);
    }
}
