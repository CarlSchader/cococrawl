// Integration tests for cococrawl binary tools
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_binary_path(name: &str) -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test executable name
    path.pop(); // Remove 'deps'
    path.push(name);
    path
}

fn create_dummy_image(path: &PathBuf, width: u32, height: u32) {
    use image::{ImageBuffer, Rgb};
    let img = ImageBuffer::from_fn(width, height, |_x, _y| Rgb([255u8, 0u8, 0u8]));
    img.save(path).unwrap();
}

fn create_test_coco_file(temp_dir: &TempDir, name: &str) -> PathBuf {
    // Create actual image files
    create_dummy_image(&temp_dir.path().join("test1.jpg"), 100, 100);
    create_dummy_image(&temp_dir.path().join("test2.jpg"), 200, 200);

    let coco_json = r#"{
        "info": {
            "year": 2020,
            "version": "1.0",
            "description": "Test Dataset",
            "contributor": "Test",
            "url": "http://test.com",
            "date_created": "2020-01-01T00:00:00Z"
        },
        "images": [
            {
                "id": 1,
                "width": 100,
                "height": 100,
                "file_name": "test1.jpg",
                "license": 0,
                "date_captured": "2020-01-01T00:00:00Z"
            },
            {
                "id": 2,
                "width": 200,
                "height": 200,
                "file_name": "test2.jpg",
                "license": 0,
                "date_captured": "2020-01-01T00:00:00Z"
            }
        ],
        "annotations": [
            {
                "id": 1,
                "image_id": 1,
                "category_id": 1,
                "segmentation": [[10.0, 10.0, 20.0, 10.0, 20.0, 20.0]],
                "area": 50.0,
                "bbox": [10.0, 10.0, 10.0, 10.0],
                "iscrowd": 0
            },
            {
                "id": 2,
                "image_id": 2,
                "category_id": 1,
                "segmentation": [[30.0, 30.0, 40.0, 30.0, 40.0, 40.0]],
                "area": 50.0,
                "bbox": [30.0, 30.0, 10.0, 10.0],
                "iscrowd": 0
            }
        ],
        "categories": [
            {
                "id": 1,
                "name": "person",
                "supercategory": "human"
            }
        ],
        "licenses": [
            {
                "id": 0,
                "name": "Test License",
                "url": "http://test.com/license"
            }
        ]
    }"#;

    let coco_path = temp_dir.path().join(name);
    fs::write(&coco_path, coco_json).unwrap();
    coco_path
}

// ========== COCOCOUNT TESTS ==========

#[test]
fn test_cococount_basic() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_file(&temp_dir, "test.json");

    let output = Command::new(get_binary_path("cococount"))
        .arg(&coco_path)
        .output()
        .expect("Failed to execute cococount");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Images: 2"));
    assert!(stdout.contains("Annotations: 2"));
    assert!(stdout.contains("Categories: 1"));
}

#[test]
fn test_cococount_empty_dataset() {
    let temp_dir = TempDir::new().unwrap();
    let coco_json = r#"{
        "images": [],
        "annotations": []
    }"#;
    let coco_path = temp_dir.path().join("empty.json");
    fs::write(&coco_path, coco_json).unwrap();

    let output = Command::new(get_binary_path("cococount"))
        .arg(&coco_path)
        .output()
        .expect("Failed to execute cococount");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Images: 0"));
    assert!(stdout.contains("Annotations: 0"));
}

#[test]
fn test_cococount_mixed_annotations() {
    let temp_dir = TempDir::new().unwrap();
    let coco_json = r#"{
        "images": [{"id": 1, "width": 100, "height": 100, "file_name": "test.jpg"}],
        "annotations": [
            {
                "id": 1,
                "image_id": 1,
                "category_id": 1,
                "segmentation": [[10.0, 10.0]],
                "area": 50.0,
                "bbox": [10.0, 10.0, 10.0, 10.0],
                "iscrowd": 0
            },
            {
                "id": 2,
                "image_id": 1,
                "caption": "A test image"
            }
        ]
    }"#;
    let coco_path = temp_dir.path().join("mixed.json");
    fs::write(&coco_path, coco_json).unwrap();

    let output = Command::new(get_binary_path("cococount"))
        .arg(&coco_path)
        .output()
        .expect("Failed to execute cococount");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Object Detection Annotations: 1"));
    assert!(stdout.contains("Image Captioning Annotations: 1"));
}

// ========== COCOSPLIT TESTS ==========

#[test]
fn test_cocosplit_with_count() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_file(&temp_dir, "test.json");
    let output_path = temp_dir.path().join("split.json");

    let output = Command::new(get_binary_path("cocosplit"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-c")
        .arg("1")
        .output()
        .expect("Failed to execute cocosplit");

    assert!(output.status.success());
    assert!(output_path.exists());

    let split_json = fs::read_to_string(&output_path).unwrap();
    let split_coco: serde_json::Value = serde_json::from_str(&split_json).unwrap();

    assert_eq!(split_coco["images"].as_array().unwrap().len(), 1);
}

#[test]
fn test_cocosplit_all_images() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_file(&temp_dir, "test.json");
    let output_path = temp_dir.path().join("all.json");

    let output = Command::new(get_binary_path("cocosplit"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cocosplit");

    assert!(output.status.success());

    let split_json = fs::read_to_string(&output_path).unwrap();
    let split_coco: serde_json::Value = serde_json::from_str(&split_json).unwrap();

    assert_eq!(split_coco["images"].as_array().unwrap().len(), 2);
    assert_eq!(split_coco["annotations"].as_array().unwrap().len(), 2);
}

#[test]
fn test_cocosplit_with_blacklist() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_file(&temp_dir, "full.json");

    // Create first split
    let split1_path = temp_dir.path().join("split1.json");
    Command::new(get_binary_path("cocosplit"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&split1_path)
        .arg("-c")
        .arg("1")
        .output()
        .expect("Failed to execute cocosplit");

    // Create second split excluding first
    let split2_path = temp_dir.path().join("split2.json");
    let output = Command::new(get_binary_path("cocosplit"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&split2_path)
        .arg("-b")
        .arg(&split1_path)
        .output()
        .expect("Failed to execute cocosplit");

    assert!(output.status.success());

    let split2_json = fs::read_to_string(&split2_path).unwrap();
    let split2_coco: serde_json::Value = serde_json::from_str(&split2_json).unwrap();

    // Should have 1 image (2 total - 1 blacklisted)
    assert_eq!(split2_coco["images"].as_array().unwrap().len(), 1);
}

#[test]
fn test_cocosplit_preserves_annotations() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_file(&temp_dir, "test.json");
    let output_path = temp_dir.path().join("split.json");

    Command::new(get_binary_path("cocosplit"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-c")
        .arg("1")
        .output()
        .expect("Failed to execute cocosplit");

    let split_json = fs::read_to_string(&output_path).unwrap();
    let split_coco: serde_json::Value = serde_json::from_str(&split_json).unwrap();

    // Should have 1 image and 1 annotation (preserving the relationship)
    assert_eq!(split_coco["images"].as_array().unwrap().len(), 1);
    assert_eq!(split_coco["annotations"].as_array().unwrap().len(), 1);
}

#[test]
fn test_cocosplit_annotated_only() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create actual image files
    create_dummy_image(&temp_dir.path().join("test1.jpg"), 100, 100);
    create_dummy_image(&temp_dir.path().join("test2.jpg"), 100, 100);
    create_dummy_image(&temp_dir.path().join("test3.jpg"), 100, 100);
    
    let coco_json = r#"{
        "images": [
            {"id": 1, "width": 100, "height": 100, "file_name": "test1.jpg"},
            {"id": 2, "width": 100, "height": 100, "file_name": "test2.jpg"},
            {"id": 3, "width": 100, "height": 100, "file_name": "test3.jpg"}
        ],
        "annotations": [
            {
                "id": 1,
                "image_id": 1,
                "category_id": 1,
                "segmentation": [[]],
                "area": 1.0,
                "bbox": [0.0, 0.0, 1.0, 1.0],
                "iscrowd": 0
            }
        ]
    }"#;
    let coco_path = temp_dir.path().join("test.json");
    fs::write(&coco_path, coco_json).unwrap();

    let output_path = temp_dir.path().join("annotated.json");
    let output = Command::new(get_binary_path("cocosplit"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--annotated-only")
        .output()
        .expect("Failed to execute cocosplit");

    assert!(output.status.success());

    let split_json = fs::read_to_string(&output_path).unwrap();
    let split_coco: serde_json::Value = serde_json::from_str(&split_json).unwrap();

    // Should only have 1 image (the one with annotations)
    assert_eq!(split_coco["images"].as_array().unwrap().len(), 1);
    assert_eq!(
        split_coco["images"].as_array().unwrap()[0]["id"]
            .as_i64()
            .unwrap(),
        1
    );
}

// ========== COCOMERGE TESTS ==========

#[test]
fn test_cocomerge_basic() {
    let temp_dir = TempDir::new().unwrap();
    let coco1_path = create_test_coco_file(&temp_dir, "coco1.json");
    let coco2_path = create_test_coco_file(&temp_dir, "coco2.json");
    let output_path = temp_dir.path().join("merged.json");

    let output = Command::new(get_binary_path("cocomerge"))
        .arg(&coco1_path)
        .arg(&coco2_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cocomerge");

    assert!(output.status.success());
    assert!(output_path.exists());

    let merged_json = fs::read_to_string(&output_path).unwrap();
    let merged_coco: serde_json::Value = serde_json::from_str(&merged_json).unwrap();

    // Without reassign flag, duplicate IDs are ignored, so we get 2 images
    // (the first file's images are kept, the second file's duplicates are dropped)
    assert_eq!(merged_coco["images"].as_array().unwrap().len(), 2);
    assert_eq!(merged_coco["annotations"].as_array().unwrap().len(), 2);
}

#[test]
fn test_cocomerge_with_reassign_ids() {
    let temp_dir = TempDir::new().unwrap();
    let coco1_path = create_test_coco_file(&temp_dir, "coco1.json");
    let coco2_path = create_test_coco_file(&temp_dir, "coco2.json");
    let output_path = temp_dir.path().join("merged.json");

    let output = Command::new(get_binary_path("cocomerge"))
        .arg(&coco1_path)
        .arg(&coco2_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-r")
        .output()
        .expect("Failed to execute cocomerge");

    assert!(output.status.success());

    let merged_json = fs::read_to_string(&output_path).unwrap();
    let merged_coco: serde_json::Value = serde_json::from_str(&merged_json).unwrap();

    // With reassign, should have 4 images
    assert_eq!(merged_coco["images"].as_array().unwrap().len(), 4);
}

#[test]
fn test_cocomerge_deduplicates_categories() {
    let temp_dir = TempDir::new().unwrap();
    let coco1_path = create_test_coco_file(&temp_dir, "coco1.json");
    let coco2_path = create_test_coco_file(&temp_dir, "coco2.json");
    let output_path = temp_dir.path().join("merged.json");

    Command::new(get_binary_path("cocomerge"))
        .arg(&coco1_path)
        .arg(&coco2_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cocomerge");

    let merged_json = fs::read_to_string(&output_path).unwrap();
    let merged_coco: serde_json::Value = serde_json::from_str(&merged_json).unwrap();

    // Should deduplicate categories (both files have same "person" category)
    assert_eq!(merged_coco["categories"].as_array().unwrap().len(), 1);
}

#[test]
fn test_cocomerge_deduplicates_licenses() {
    let temp_dir = TempDir::new().unwrap();
    let coco1_path = create_test_coco_file(&temp_dir, "coco1.json");
    let coco2_path = create_test_coco_file(&temp_dir, "coco2.json");
    let output_path = temp_dir.path().join("merged.json");

    Command::new(get_binary_path("cocomerge"))
        .arg(&coco1_path)
        .arg(&coco2_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cocomerge");

    let merged_json = fs::read_to_string(&output_path).unwrap();
    let merged_coco: serde_json::Value = serde_json::from_str(&merged_json).unwrap();

    // Should deduplicate licenses (both files have same license)
    assert_eq!(merged_coco["licenses"].as_array().unwrap().len(), 1);
}

#[test]
fn test_cocomerge_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_file(&temp_dir, "coco.json");
    let output_path = temp_dir.path().join("merged.json");

    let output = Command::new(get_binary_path("cocomerge"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cocomerge");

    assert!(output.status.success());

    let merged_json = fs::read_to_string(&output_path).unwrap();
    let merged_coco: serde_json::Value = serde_json::from_str(&merged_json).unwrap();

    // Should be identical to input
    assert_eq!(merged_coco["images"].as_array().unwrap().len(), 2);
    assert_eq!(merged_coco["annotations"].as_array().unwrap().len(), 2);
}

// ========== ERROR HANDLING TESTS ==========

#[test]
fn test_cococount_missing_file() {
    let output = Command::new(get_binary_path("cococount"))
        .arg("nonexistent.json")
        .output()
        .expect("Failed to execute cococount");

    assert!(!output.status.success());
}

#[test]
fn test_cocosplit_missing_file() {
    let output = Command::new(get_binary_path("cocosplit"))
        .arg("nonexistent.json")
        .output()
        .expect("Failed to execute cocosplit");

    assert!(!output.status.success());
}

#[test]
fn test_cocomerge_missing_file() {
    let output = Command::new(get_binary_path("cocomerge"))
        .arg("nonexistent.json")
        .output()
        .expect("Failed to execute cocomerge");

    assert!(!output.status.success());
}
