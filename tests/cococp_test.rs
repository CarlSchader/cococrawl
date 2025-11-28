// Integration tests for cococp binary
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_binary_path(name: &str) -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.push(name);
    path
}

fn create_dummy_image(path: &PathBuf, width: u32, height: u32) {
    use image::{ImageBuffer, Rgb};
    let img = ImageBuffer::from_fn(width, height, |_x, _y| Rgb([255u8, 0u8, 0u8]));
    img.save(path).unwrap();
}

fn create_test_coco_with_images(temp_dir: &TempDir) -> PathBuf {
    let images_dir = temp_dir.path().join("source_images");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("img1.jpg"), 100, 100);
    create_dummy_image(&images_dir.join("img2.png"), 200, 200);

    let coco_json = r#"{
        "images": [
            {
                "id": 0,
                "width": 100,
                "height": 100,
                "file_name": "source_images/img1.jpg"
            },
            {
                "id": 1,
                "width": 200,
                "height": 200,
                "file_name": "source_images/img2.png"
            }
        ],
        "annotations": []
    }"#;

    let coco_path = temp_dir.path().join("test.json");
    fs::write(&coco_path, coco_json).unwrap();
    coco_path
}

#[test]
fn test_cococp_basic() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_with_images(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    let output = Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to execute cococp");

    assert!(output.status.success(), "cococp failed: {:?}", output);
    assert!(output_dir.exists());
    assert!(output_dir.join("images").exists());
    assert!(output_dir.join("test.json").exists());
}

#[test]
fn test_cococp_copies_images() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_with_images(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to execute cococp");

    let images_dir = output_dir.join("images");
    // Original filenames should be preserved
    assert!(images_dir.join("img1.jpg").exists());
    assert!(images_dir.join("img2.png").exists());
}

#[test]
fn test_cococp_updates_paths() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_with_images(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to execute cococp");

    let output_coco_path = output_dir.join("test.json");
    let coco_json = fs::read_to_string(&output_coco_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    // Original filenames should be preserved
    assert_eq!(
        coco["images"][0]["file_name"].as_str().unwrap(),
        "images/img1.jpg"
    );
    assert_eq!(
        coco["images"][1]["file_name"].as_str().unwrap(),
        "images/img2.png"
    );
}

#[test]
fn test_cococp_preserves_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("source");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("img.jpg"), 100, 100);

    let coco_json = r#"{
        "info": {
            "year": 2020,
            "version": "1.0",
            "description": "Test",
            "contributor": "Tester",
            "url": "http://test.com",
            "date_created": "2020-01-01T00:00:00Z"
        },
        "images": [
            {
                "id": 5,
                "width": 100,
                "height": 100,
                "file_name": "source/img.jpg"
            }
        ],
        "annotations": [
            {
                "id": 1,
                "image_id": 5,
                "category_id": 1,
                "segmentation": [[]],
                "area": 50.0,
                "bbox": [0.0, 0.0, 10.0, 10.0],
                "iscrowd": 0
            }
        ],
        "categories": [
            {
                "id": 1,
                "name": "person",
                "supercategory": "human"
            }
        ]
    }"#;

    let coco_path = temp_dir.path().join("test.json");
    fs::write(&coco_path, coco_json).unwrap();

    let output_dir = temp_dir.path().join("output");

    Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to execute cococp");

    let output_coco_path = output_dir.join("test.json");
    let coco_json = fs::read_to_string(&output_coco_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    assert_eq!(coco["info"]["version"].as_str().unwrap(), "1.0");
    assert_eq!(coco["annotations"].as_array().unwrap().len(), 1);
    assert_eq!(coco["categories"].as_array().unwrap().len(), 1);
}

#[test]
fn test_cococp_preserves_many_filenames() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("source");
    fs::create_dir(&images_dir).unwrap();

    // Create 100 images to test that original filenames are preserved
    for i in 0..100 {
        create_dummy_image(&images_dir.join(format!("img{}.jpg", i)), 10, 10);
    }

    let mut images_json = Vec::new();
    for i in 0..100 {
        images_json.push(format!(
            r#"{{"id": {}, "width": 10, "height": 10, "file_name": "source/img{}.jpg"}}"#,
            i, i
        ));
    }

    let coco_json = format!(
        r#"{{"images": [{}], "annotations": []}}"#,
        images_json.join(",")
    );

    let coco_path = temp_dir.path().join("test.json");
    fs::write(&coco_path, coco_json).unwrap();

    let output_dir = temp_dir.path().join("output");

    Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to execute cococp");

    let images_output = output_dir.join("images");
    // Original filenames should be preserved (not renamed to zero-padded IDs)
    assert!(images_output.join("img0.jpg").exists());
    assert!(images_output.join("img99.jpg").exists());
    assert!(images_output.join("img50.jpg").exists());
}

#[test]
fn test_cococp_absolute_paths() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("source");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("img.jpg"), 100, 100);

    let absolute_path = images_dir.join("img.jpg").canonicalize().unwrap();

    let coco_json = format!(
        r#"{{
        "images": [
            {{
                "id": 0,
                "width": 100,
                "height": 100,
                "file_name": "{}"
            }}
        ],
        "annotations": []
    }}"#,
        absolute_path.to_string_lossy().replace("\\", "\\\\")
    );

    let coco_path = temp_dir.path().join("test.json");
    fs::write(&coco_path, coco_json).unwrap();

    let output_dir = temp_dir.path().join("output");

    let output = Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to execute cococp");

    assert!(output.status.success());
    assert!(output_dir.join("images").join("img.jpg").exists());
}

#[test]
fn test_cococp_missing_source_image() {
    let temp_dir = TempDir::new().unwrap();

    let coco_json = r#"{
        "images": [
            {
                "id": 0,
                "width": 100,
                "height": 100,
                "file_name": "nonexistent.jpg"
            }
        ],
        "annotations": []
    }"#;

    let coco_path = temp_dir.path().join("test.json");
    fs::write(&coco_path, coco_json).unwrap();

    let output_dir = temp_dir.path().join("output");

    let output = Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to execute cococp");

    // Should complete but warn about missing file
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Warning"));
}

#[test]
fn test_cococp_custom_output_dir() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_with_images(&temp_dir);
    let custom_dir = temp_dir.path().join("my-custom-output");

    Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&custom_dir)
        .output()
        .expect("Failed to execute cococp");

    assert!(custom_dir.exists());
    assert!(custom_dir.join("images").exists());
    assert!(custom_dir.join("test.json").exists());
}

#[test]
fn test_cococp_default_output_dir() {
    let temp_dir = TempDir::new().unwrap();
    let coco_path = create_test_coco_with_images(&temp_dir);

    // Change to temp directory and run without -o flag
    let output = Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute cococp");

    assert!(output.status.success());

    let default_dir = temp_dir.path().join("coco-dataset");
    assert!(default_dir.exists());
    assert!(default_dir.join("images").exists());
}

#[test]
fn test_cococp_preserves_extension() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("source");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("img1.jpg"), 100, 100);
    create_dummy_image(&images_dir.join("img2.png"), 100, 100);
    create_dummy_image(&images_dir.join("img3.bmp"), 100, 100);

    let coco_json = r#"{
        "images": [
            {"id": 0, "width": 100, "height": 100, "file_name": "source/img1.jpg"},
            {"id": 1, "width": 100, "height": 100, "file_name": "source/img2.png"},
            {"id": 2, "width": 100, "height": 100, "file_name": "source/img3.bmp"}
        ],
        "annotations": []
    }"#;

    let coco_path = temp_dir.path().join("test.json");
    fs::write(&coco_path, coco_json).unwrap();

    let output_dir = temp_dir.path().join("output");

    Command::new(get_binary_path("cococp"))
        .arg(&coco_path)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to execute cococp");

    let images_output = output_dir.join("images");
    assert!(images_output.join("img1.jpg").exists());
    assert!(images_output.join("img2.png").exists());
    assert!(images_output.join("img3.bmp").exists());
}
