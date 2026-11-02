use std::fs;
use std::process::Command;

use serde_json::{Value, json};
use tempfile::TempDir;

fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let mut images = Vec::new();
    let mut annotations = Vec::new();
    // Deliberately reverse input order so no-shuffle tests require sorting by ID.
    for id in (1..=40).rev() {
        let file_name = format!("image-{id}.png");
        image::RgbImage::new(1, 1)
            .save(dir.path().join(&file_name))
            .unwrap();
        images.push(json!({
            "id": id, "width": 1, "height": 1, "file_name": file_name
        }));
        if id % 2 == 0 {
            annotations.push(json!({
                "id": id, "image_id": id, "category_id": 1,
                "bbox": [0, 0, 1, 1], "area": 1, "iscrowd": 0,
                "segmentation": [[0, 0, 1, 0, 1, 1]]
            }));
        }
    }
    let dataset = json!({
        "images": images, "annotations": annotations,
        "categories": [{"id": 1, "name": "object", "supercategory": "object"}],
        "licenses": []
    });
    fs::write(
        dir.path().join("input.json"),
        serde_json::to_vec(&dataset).unwrap(),
    )
    .unwrap();
    let blacklist = json!({
        "images": dataset["images"].as_array().unwrap().iter()
            .filter(|image| image["id"].as_i64().unwrap() <= 10)
            .cloned().collect::<Vec<_>>(),
        "annotations": [], "categories": [], "licenses": []
    });
    fs::write(
        dir.path().join("blacklist.json"),
        serde_json::to_vec(&blacklist).unwrap(),
    )
    .unwrap();
    dir
}

fn split(dir: &TempDir, args: &[&str]) -> Vec<i64> {
    let result = Command::new(env!("CARGO_BIN_EXE_cocosplit"))
        .current_dir(dir.path())
        .args(["input.json", "--output", "output.json"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "cocosplit failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output: Value =
        serde_json::from_slice(&fs::read(dir.path().join("output.json")).unwrap()).unwrap();
    output["images"]
        .as_array()
        .unwrap()
        .iter()
        .map(|image| image["id"].as_i64().unwrap())
        .collect()
}

#[test]
fn seeded_splits_are_reproducible_across_processes_with_filters() {
    let dir = fixture();
    for blacklist in [false, true] {
        for annotated_only in [false, true] {
            let mut args = vec!["--shuffle", "42", "--count", "5"];
            if blacklist {
                args.extend(["--blacklist-file", "blacklist.json"]);
            }
            if annotated_only {
                args.push("--annotated-only");
            }
            let expected = split(&dir, &args);
            assert_eq!(expected.len(), 5);
            assert!(expected.iter().all(|id| !blacklist || *id > 10));
            assert!(expected.iter().all(|id| !annotated_only || id % 2 == 0));
            for _ in 0..4 {
                assert_eq!(split(&dir, &args), expected);
            }
        }
    }
}

#[test]
fn different_seeds_can_change_selected_images() {
    let dir = fixture();
    let mut first = split(&dir, &["--shuffle", "42", "--count", "5"]);
    let mut second = split(&dir, &["--shuffle", "43", "--count", "5"]);
    // Compare membership, not merely permutation of the same selection.
    first.sort_unstable();
    second.sort_unstable();
    assert_ne!(first, second);
}

#[test]
fn no_shuffle_sorts_ids_and_applies_offset_after_filtering() {
    let dir = fixture();
    assert_eq!(split(&dir, &["--count", "5"]), vec![1, 2, 3, 4, 5]);
    assert_eq!(
        split(&dir, &["--offset", "3", "--count", "5"]),
        vec![4, 5, 6, 7, 8]
    );
    let filtered = ["--blacklist-file", "blacklist.json", "--annotated-only"];
    let mut args = filtered.to_vec();
    args.extend(["--offset", "3", "--count", "5"]);
    assert_eq!(split(&dir, &args), vec![18, 20, 22, 24, 26]);
    let mut args = filtered.to_vec();
    args.extend(["--offset", "12"]);
    assert_eq!(split(&dir, &args), vec![36, 38, 40]);
    assert!(split(&dir, &["--offset", "50"]).is_empty());
}
