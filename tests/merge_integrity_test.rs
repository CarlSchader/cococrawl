use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use cococrawl::{
    CocoCategory, CocoKeypointDetectionCategory, CocoLicense, CocoObjectDetectionCategory,
    CocoPanopticSegmentationCategory, HasID,
};
use image::{Rgb, RgbImage};
use serde_json::{Value, json};
use tempfile::TempDir;

fn hash(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn assert_id_independent_hash<T: Clone + Eq + Hash + HasID<i32>>(value: T) {
    let mut other = value.clone();
    other.set_id(value.id() + 100);
    assert!(value == other);
    assert_eq!(hash(&value), hash(&other));
    assert_eq!(HashSet::from([value, other]).len(), 1);
}

#[test]
fn license_hash_uses_only_equality_fields() {
    let license = CocoLicense {
        id: 1,
        name: "License".into(),
        url: "https://example.com/license".into(),
    };
    assert_eq!(hash(&license), hash(&(&license.name, &license.url)));
    assert_id_independent_hash(license);
}

#[test]
fn object_category_hash_uses_only_equality_fields() {
    let category = CocoObjectDetectionCategory {
        id: 1,
        name: "person".into(),
        supercategory: "human".into(),
    };
    assert_eq!(
        hash(&category),
        hash(&(&category.supercategory, &category.name))
    );
    assert_id_independent_hash(category.clone());
    assert_id_independent_hash(CocoCategory::ObjectDetection(category));
}

#[test]
fn keypoint_category_hash_uses_only_equality_fields() {
    let category = CocoKeypointDetectionCategory {
        id: 1,
        name: "person".into(),
        supercategory: "human".into(),
        keypoints: vec!["left".into(), "right".into()],
        skeleton: vec![[1, 2]],
    };
    assert_eq!(
        hash(&category),
        hash(&(
            &category.supercategory,
            &category.name,
            &category.keypoints,
            &category.skeleton,
        ))
    );
    assert_id_independent_hash(category.clone());
    assert_id_independent_hash(CocoCategory::KeypointDetection(category));
}

#[test]
fn panoptic_category_hash_uses_only_equality_fields() {
    let category = CocoPanopticSegmentationCategory {
        id: 1,
        name: "person".into(),
        supercategory: "human".into(),
        isthing: true,
        color: [30, 20, 10],
    };
    assert_eq!(
        hash(&category),
        hash(&(
            &category.supercategory,
            &category.name,
            category.isthing,
            category.color,
        ))
    );
    assert_id_independent_hash(category.clone());
    assert_id_independent_hash(CocoCategory::PanopticSegmentation(category));
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

fn merge(inputs: &[PathBuf], output: &Path) -> Value {
    let result = Command::new(env!("CARGO_BIN_EXE_cocomerge"))
        .args(inputs)
        .arg("--reassign-clashing-ids")
        .arg("--output-path")
        .arg(output)
        .output()
        .expect("Failed to execute cocomerge");
    assert!(
        result.status.success(),
        "cocomerge failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&fs::read(output).unwrap()).unwrap()
}

fn category(kind: &str, id: i32, name: &str) -> Value {
    let mut category = json!({"id": id, "name": name, "supercategory": "test"});
    match kind {
        "keypoint" => {
            category["keypoints"] = json!(["left", "right"]);
            category["skeleton"] = json!([[1, 2]]);
        }
        "panoptic" => {
            category["isthing"] = json!(1);
            category["color"] = json!([10, 20, 30]);
        }
        "object" => {}
        _ => panic!("Unknown category kind"),
    }
    category
}

fn assert_merge_collisions_and_dedup(kind: &str) {
    let temp = TempDir::new().unwrap();
    RgbImage::new(1, 1)
        .save(temp.path().join("image.png"))
        .unwrap();
    if kind == "panoptic" {
        RgbImage::from_pixel(1, 1, Rgb([1, 0, 0]))
            .save(temp.path().join("mask.png"))
            .unwrap();
    }
    let datasets = [
        vec![(1, "A")],
        vec![(1, "B"), (2, "C"), (3, "D")],
        vec![(1, "C"), (2, "A"), (3, "E"), (4, "B"), (5, "D")],
    ];
    let mut inputs = Vec::new();
    let mut expected = HashMap::new();
    let mut image_id = 0;
    for (index, entries) in datasets.iter().enumerate() {
        let mut categories = Vec::new();
        let mut licenses = Vec::new();
        let mut images = Vec::new();
        let mut annotations = Vec::new();
        for &(id, name) in entries {
            image_id += 1;
            expected.insert(image_id, name);
            categories.push(category(kind, id, name));
            licenses.push(json!({"id": id, "name": name, "url": format!("https://{name}")}));
            images.push(json!({
                "id": image_id, "width": 1, "height": 1,
                "file_name": "image.png", "license": id
            }));
            let mut annotation = json!({
                "id": image_id, "image_id": image_id, "category_id": id,
                "segmentation": [], "area": 1, "bbox": [0, 0, 1, 1], "iscrowd": 0
            });
            match kind {
                "keypoint" => {
                    annotation["keypoints"] = json!([0, 0, 2, 0, 0, 2]);
                    annotation["num_keypoints"] = json!(2);
                }
                "panoptic" => {
                    annotation = json!({
                        "image_id": image_id, "file_name": "mask.png",
                        "segments_info": [{
                            "id": 1, "category_id": id, "area": 1,
                            "bbox": [0, 0, 1, 1], "iscrowd": 0
                        }]
                    });
                }
                _ => {}
            }
            annotations.push(annotation);
        }
        let input = temp.path().join(format!("input-{index}.json"));
        write_json(
            &input,
            &json!({
                "categories": categories, "licenses": licenses,
                "images": images, "annotations": annotations
            }),
        );
        inputs.push(input);
    }

    let output = merge(&inputs, &temp.path().join("merged.json"));
    for field in ["categories", "licenses"] {
        let entries = output[field].as_array().unwrap();
        assert_eq!(entries.len(), 5, "{field} must deduplicate equal values");
        let ids: HashSet<_> = entries
            .iter()
            .map(|entry| entry["id"].as_i64().unwrap())
            .collect();
        assert_eq!(ids.len(), entries.len(), "{field} IDs must be unique");
        assert_eq!(
            entries
                .iter()
                .find(|entry| entry["name"] == "A")
                .unwrap()["id"],
            1,
            "the first semantic match keeps its original ID"
        );
    }
    let names_by_id = |field: &str| -> HashMap<i64, &str> {
        output[field]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                (entry["id"].as_i64().unwrap(), entry["name"].as_str().unwrap())
            })
            .collect()
    };
    let licenses = names_by_id("licenses");
    let categories = names_by_id("categories");
    assert_eq!(output["images"].as_array().unwrap().len(), expected.len());
    assert_eq!(
        output["annotations"].as_array().unwrap().len(),
        expected.len()
    );
    for image in output["images"].as_array().unwrap() {
        assert_eq!(
            licenses[&image["license"].as_i64().unwrap()],
            expected[&image["id"].as_i64().unwrap()]
        );
    }
    for annotation in output["annotations"].as_array().unwrap() {
        let category_id = if kind == "panoptic" {
            &annotation["segments_info"][0]["category_id"]
        } else {
            &annotation["category_id"]
        };
        assert_eq!(
            categories[&category_id.as_i64().unwrap()],
            expected[&annotation["image_id"].as_i64().unwrap()]
        );
    }
}

#[test]
fn merge_object_categories_and_licenses_preserves_references() {
    assert_merge_collisions_and_dedup("object");
}

#[test]
fn merge_keypoint_categories_and_licenses_preserves_references() {
    assert_merge_collisions_and_dedup("keypoint");
}

#[test]
fn merge_panoptic_categories_and_licenses_preserves_references() {
    assert_merge_collisions_and_dedup("panoptic");
}

#[test]
fn merge_preserves_per_image_segment_ids_and_mask_pixels() {
    let temp = TempDir::new().unwrap();
    let colors = [[1, 0, 0], [1, 2, 3]];
    let segment_ids = [1, 1 + 2 * 256 + 3 * 256 * 256];
    let mut inputs = Vec::new();
    let mut original_masks = HashMap::new();
    for name in ["A", "B"] {
        let image_name = format!("image-{name}.png");
        RgbImage::new(2, 1)
            .save(temp.path().join(&image_name))
            .unwrap();
        let mask_name = format!("mask-{name}.png");
        let mask_path = temp.path().join(&mask_name);
        let mask = RgbImage::from_fn(2, 1, |x, _| Rgb(colors[x as usize]));
        mask.save(&mask_path).unwrap();
        original_masks.insert(mask_name.clone(), fs::read(&mask_path).unwrap());
        let segments: Vec<_> = segment_ids
            .iter()
            .enumerate()
            .map(|(x, id)| {
                json!({
                    "id": id, "category_id": 1, "area": 1,
                    "bbox": [x, 0, 1, 1], "iscrowd": 0
                })
            })
            .collect();
        let input = temp.path().join(format!("input-{name}.json"));
        write_json(
            &input,
            &json!({
                "images": [{"id": 1, "width": 2, "height": 1, "file_name": image_name}],
                "categories": [category("panoptic", 1, name)],
                "annotations": [
                    {"id": 1, "image_id": 1, "caption": name},
                    {"image_id": 1, "file_name": mask_name, "segments_info": segments}
                ]
            }),
        );
        inputs.push(input);
    }
    let output = merge(&inputs, &temp.path().join("merged.json"));
    let images = output["images"].as_array().unwrap();
    assert_eq!(images.len(), 2);
    assert_ne!(images[0]["id"], images[1]["id"]);
    let annotations = output["annotations"].as_array().unwrap();
    assert_eq!(annotations.len(), 4);
    let captions: Vec<_> = annotations
        .iter()
        .filter(|ann| ann.get("caption").is_some())
        .collect();
    assert_eq!(captions.len(), 2);
    assert_eq!(captions[0]["id"], 1);
    assert_eq!(captions[1]["id"], 2, "mask IDs must not reserve annotation IDs");
    for annotation in annotations
        .iter()
        .filter(|ann| ann.get("segments_info").is_some())
    {
        let mask_name = annotation["file_name"].as_str().unwrap();
        let mask_path = temp.path().join(mask_name);
        assert_eq!(fs::read(&mask_path).unwrap(), original_masks[mask_name]);
        let mask = image::open(&mask_path).unwrap().to_rgb8();
        let image = images
            .iter()
            .find(|im| im["id"] == annotation["image_id"])
            .unwrap();
        let segments = annotation["segments_info"].as_array().unwrap();
        assert_eq!(segments.len(), 2);
        for (x, segment) in segments.iter().enumerate() {
            let [r, g, b] = mask.get_pixel(x as u32, 0).0;
            let pixel_id = i64::from(r) + 256 * i64::from(g) + 65536 * i64::from(b);
            assert_eq!(segment["id"], pixel_id);
            assert_eq!(segment["id"], segment_ids[x]);
            let category = output["categories"]
                .as_array()
                .unwrap()
                .iter()
                .find(|cat| cat["id"] == segment["category_id"])
                .unwrap();
            let name = category["name"].as_str().unwrap();
            assert_eq!(image["file_name"], format!("image-{name}.png"));
            assert_eq!(mask_name, format!("mask-{name}.png"));
        }
    }
}
