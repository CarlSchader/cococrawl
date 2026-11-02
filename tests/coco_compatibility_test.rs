use std::fs;
use std::process::Command;

use chrono::{DateTime, Utc};
use cococrawl::{CocoAnnotation, CocoFile, CocoImage, CocoInfo, CocoRLECounts, CocoSegmentation};
use serde_json::{Value, json};
use tempfile::TempDir;

fn info(date: Value) -> Value {
    json!({
        "year": 2017,
        "version": "1.0",
        "description": "Compatibility fixture",
        "contributor": "Test",
        "url": "https://example.com",
        "date_created": date
    })
}

fn image() -> Value {
    json!({"id": 1, "width": 2, "height": 2, "file_name": "image.png"})
}

fn annotations(counts: Value) -> Vec<Value> {
    let object = json!({
        "id": 1, "image_id": 1, "category_id": 1,
        "segmentation": {"size": [2, 2], "counts": counts},
        "area": 4.0, "bbox": [0.0, 0.0, 2.0, 2.0], "iscrowd": 1
    });
    let mut keypoint = object.clone();
    keypoint["id"] = json!(2);
    keypoint["keypoints"] = json!([1.0, 1.0, 2.0]);
    keypoint["num_keypoints"] = json!(1);
    let densepose = json!({
        "id": 3, "image_id": 1, "category_id": 1, "iscrowd": 1,
        "area": 4, "bbox": [0.0, 0.0, 2.0, 2.0],
        "dp_I": [], "dp_U": [], "dp_V": [], "dp_x": [], "dp_y": [],
        "dp_masks": [{"size": [2, 2], "counts": counts}]
    });
    vec![object, keypoint, densepose]
}

#[test]
fn conventional_and_rfc3339_dates_roundtrip_as_utc_rfc3339() {
    for (input, expected) in [
        ("2013-11-14 17:02:52", "2013-11-14T17:02:52Z"),
        ("2017/09/01", "2017-09-01T00:00:00Z"),
        ("2017-09-01", "2017-09-01T00:00:00Z"),
        ("2020-02-29", "2020-02-29T00:00:00Z"),
        ("2013-11-14T17:02:52Z", "2013-11-14T17:02:52Z"),
        (
            "2013-11-14T17:02:52.123456789+02:30",
            "2013-11-14T14:32:52.123456789Z",
        ),
    ] {
        let expected_date: DateTime<Utc> = expected.parse().unwrap();
        let parsed_info: CocoInfo = serde_json::from_value(info(json!(input))).unwrap();
        let mut image_json = image();
        image_json["date_captured"] = json!(input);
        let parsed_image: CocoImage = serde_json::from_value(image_json).unwrap();
        assert_eq!(parsed_info.date_created, expected_date, "{input}");
        assert_eq!(parsed_image.date_captured, Some(expected_date), "{input}");

        let serialized_info = serde_json::to_value(&parsed_info).unwrap();
        let serialized_image = serde_json::to_value(&parsed_image).unwrap();
        assert_eq!(serialized_info["date_created"], expected);
        assert_eq!(serialized_image["date_captured"], expected);
        let reparsed_info: CocoInfo = serde_json::from_value(serialized_info).unwrap();
        let reparsed_image: CocoImage = serde_json::from_value(serialized_image).unwrap();
        assert_eq!(reparsed_info.date_created, expected_date);
        assert_eq!(reparsed_image.date_captured, Some(expected_date));
    }
}

#[test]
fn optional_image_dates_accept_missing_and_null_and_serialize_as_absent() {
    let missing = image();
    let mut null = image();
    null["date_captured"] = Value::Null;
    for value in [missing, null] {
        let parsed: CocoImage = serde_json::from_value(value).unwrap();
        assert!(parsed.date_captured.is_none());
        let serialized = serde_json::to_value(&parsed).unwrap();
        assert!(serialized.get("date_captured").is_none());
        let reparsed: CocoImage = serde_json::from_value(serialized).unwrap();
        assert!(reparsed.date_captured.is_none());
    }

    let mut missing_info_date = info(Value::Null);
    assert!(serde_json::from_value::<CocoInfo>(missing_info_date.clone()).is_err());
    missing_info_date
        .as_object_mut()
        .unwrap()
        .remove("date_created");
    assert!(serde_json::from_value::<CocoInfo>(missing_info_date).is_err());
}

#[test]
fn malformed_dates_are_rejected_in_both_fields() {
    for value in [
        json!(""),
        json!("not a date"),
        json!("2017-02-29"),
        json!("2017/13/01"),
        json!("2013-11-14 25:02:52"),
        json!("2017-09-01 trailing text"),
        json!("2013-11-14T17:02:52+25:00"),
        json!(123),
        json!(false),
        json!([]),
        json!({}),
    ] {
        assert!(
            serde_json::from_value::<CocoInfo>(info(value.clone())).is_err(),
            "{value}"
        );
        let mut image_json = image();
        image_json["date_captured"] = value.clone();
        assert!(
            serde_json::from_value::<CocoImage>(image_json).is_err(),
            "{value}"
        );
    }
}

#[test]
fn both_rle_representations_roundtrip_in_all_mask_annotation_types() {
    // Compressed strings remain opaque: no decompression or normalization is attempted.
    for counts in [
        json!([0, 4]),
        json!("04"),
        json!("opaque\\counts"),
        json!([]),
        json!(""),
    ] {
        for value in annotations(counts.clone()) {
            let annotation: CocoAnnotation = serde_json::from_value(value).unwrap();
            let rle = match &annotation {
                CocoAnnotation::ObjectDetection(ann) => match &ann.segmentation {
                    CocoSegmentation::RLE(rle) => rle,
                    _ => panic!("expected object RLE"),
                },
                CocoAnnotation::KeypointDetection(ann) => match &ann.segmentation {
                    CocoSegmentation::RLE(rle) => rle,
                    _ => panic!("expected keypoint RLE"),
                },
                CocoAnnotation::DensePose(ann) => &ann.dp_masks[0],
                _ => panic!("expected mask annotation"),
            };
            match &rle.counts {
                CocoRLECounts::Uncompressed(runs) => assert_eq!(json!(runs), counts),
                CocoRLECounts::Compressed(encoded) => assert_eq!(json!(encoded), counts),
            }
            assert_eq!(rle.size, (2, 2));
            let serialized = serde_json::to_value(&annotation).unwrap();
            let mask = serialized
                .get("segmentation")
                .unwrap_or(&serialized["dp_masks"][0]);
            assert_eq!(mask, &json!({"size": [2, 2], "counts": counts}));
            let reparsed: CocoAnnotation = serde_json::from_value(serialized.clone()).unwrap();
            assert_eq!(serde_json::to_value(&reparsed).unwrap(), serialized);
        }
    }
}

#[test]
fn malformed_rle_counts_are_rejected_inside_annotations() {
    for counts in [
        Value::Null,
        json!(4),
        json!(true),
        json!({}),
        json!([-1]),
        json!([4294967296_u64]),
        json!([1.5]),
        json!(["4"]),
        json!([0, null]),
    ] {
        for value in annotations(counts.clone()) {
            assert!(
                serde_json::from_value::<CocoAnnotation>(value).is_err(),
                "{counts}"
            );
        }
    }
    for mut value in annotations(json!([0, 4])) {
        let mask = if value.get("segmentation").is_some() {
            &mut value["segmentation"]
        } else {
            &mut value["dp_masks"][0]
        };
        mask.as_object_mut().unwrap().remove("counts");
        assert!(serde_json::from_value::<CocoAnnotation>(value).is_err());
    }
}

#[test]
fn cli_outputs_preserve_both_rle_representations_and_optional_dates() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("input.json");
    let mut images = vec![];
    for id in 1..=3 {
        let name = format!("image{id}.png");
        image::RgbImage::new(2, 2)
            .save(temp.path().join(&name))
            .unwrap();
        let mut value = image();
        value["id"] = json!(id);
        value["file_name"] = json!(name);
        if id == 1 {
            value["date_captured"] = json!("2013-11-14 17:02:52");
        } else if id == 2 {
            value["date_captured"] = Value::Null;
        }
        images.push(value);
    }
    let mut anns = annotations(json!([0, 4]));
    for mut ann in annotations(json!("04")) {
        ann["id"] = json!(ann["id"].as_i64().unwrap() + 3);
        anns.push(ann);
    }
    let fixture = json!({
        "info": info(json!("2017/09/01")),
        "images": images, "annotations": anns,
        "categories": [{"id": 1, "name": "person", "supercategory": "human"}]
    });
    fs::write(&input, serde_json::to_vec(&fixture).unwrap()).unwrap();

    for (binary, output_name, is_copy) in [
        (env!("CARGO_BIN_EXE_cocosplit"), "split.json", false),
        (env!("CARGO_BIN_EXE_cococp"), "copied", true),
        (env!("CARGO_BIN_EXE_cocomerge"), "merged.json", false),
    ] {
        let destination = temp.path().join(output_name);
        let result = Command::new(binary)
            .arg(&input)
            .arg("-o")
            .arg(&destination)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{binary}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let manifest = if is_copy {
            destination.join("input.json")
        } else {
            destination
        };
        let output: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
        let _: CocoFile = serde_json::from_value(output.clone()).unwrap();
        assert_eq!(output["annotations"].as_array().unwrap().len(), 6);
        for annotation in output["annotations"].as_array().unwrap() {
            let id = annotation["id"].as_u64().unwrap();
            let expected = if id <= 3 { json!([0, 4]) } else { json!("04") };
            let mask = annotation
                .get("segmentation")
                .unwrap_or(&annotation["dp_masks"][0]);
            assert_eq!(mask, &json!({"size": [2, 2], "counts": expected}));
        }
        assert_eq!(output["images"].as_array().unwrap().len(), 3);
        for image in output["images"].as_array().unwrap() {
            if image["id"] == 1 {
                assert_eq!(image["date_captured"], "2013-11-14T17:02:52Z");
            } else {
                assert!(image.get("date_captured").is_none());
            }
        }
        // Merge intentionally creates fresh dataset info; split and copy retain it.
        if output_name != "merged.json" {
            assert_eq!(output["info"]["date_created"], "2017-09-01T00:00:00Z");
        }
        let counted = Command::new(env!("CARGO_BIN_EXE_cococount"))
            .arg(&manifest)
            .output()
            .unwrap();
        assert!(counted.status.success());
        assert!(String::from_utf8_lossy(&counted.stdout).contains("Annotations: 6"));
    }
}
