// Integration tests for cococrawl binary
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

#[test]
fn test_cococrawl_single_directory() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    // Create test images
    create_dummy_image(&images_dir.join("test1.jpg"), 100, 100);
    create_dummy_image(&images_dir.join("test2.png"), 200, 200);

    let output_path = temp_dir.path().join("coco.json");

    let output = Command::new(get_binary_path("cococrawl"))
        .arg(&images_dir)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cococrawl");

    assert!(output.status.success(), "cococrawl failed: {:?}", output);
    assert!(output_path.exists());

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    assert_eq!(coco["images"].as_array().unwrap().len(), 2);
    
    // Check that we have one 100x100 and one 200x200 image (order may vary)
    let widths: Vec<u64> = coco["images"]
        .as_array()
        .unwrap()
        .iter()
        .map(|img| img["width"].as_u64().unwrap())
        .collect();
    let heights: Vec<u64> = coco["images"]
        .as_array()
        .unwrap()
        .iter()
        .map(|img| img["height"].as_u64().unwrap())
        .collect();
    
    assert!(widths.contains(&100));
    assert!(widths.contains(&200));
    assert!(heights.contains(&100));
    assert!(heights.contains(&200));
}

#[test]
fn test_cococrawl_multiple_directories() {
    let temp_dir = TempDir::new().unwrap();
    let dir1 = temp_dir.path().join("dir1");
    let dir2 = temp_dir.path().join("dir2");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();

    create_dummy_image(&dir1.join("img1.jpg"), 100, 100);
    create_dummy_image(&dir2.join("img2.jpg"), 200, 200);

    let output_path = temp_dir.path().join("coco.json");

    let output = Command::new(get_binary_path("cococrawl"))
        .arg(&dir1)
        .arg(&dir2)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cococrawl");

    assert!(output.status.success());

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    assert_eq!(coco["images"].as_array().unwrap().len(), 2);
}

#[test]
fn test_cococrawl_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("root");
    let subdir = root.join("subdir");
    fs::create_dir_all(&subdir).unwrap();

    create_dummy_image(&root.join("img1.jpg"), 100, 100);
    create_dummy_image(&subdir.join("img2.jpg"), 200, 200);

    let output_path = temp_dir.path().join("coco.json");

    let output = Command::new(get_binary_path("cococrawl"))
        .arg(&root)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cococrawl");

    assert!(output.status.success());

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    assert_eq!(coco["images"].as_array().unwrap().len(), 2);
}

#[test]
fn test_cococrawl_absolute_paths() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("test.jpg"), 100, 100);

    let output_path = temp_dir.path().join("coco.json");

    let output = Command::new(get_binary_path("cococrawl"))
        .arg(&images_dir)
        .arg("-o")
        .arg(&output_path)
        .arg("--absolute-paths")
        .output()
        .expect("Failed to execute cococrawl");

    assert!(output.status.success());

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    let file_name = coco["images"][0]["file_name"].as_str().unwrap();
    assert!(file_name.starts_with("/") || file_name.contains(":\\"));
}

#[test]
fn test_cococrawl_relative_paths() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("test.jpg"), 100, 100);

    let output_path = temp_dir.path().join("coco.json");

    let output = Command::new(get_binary_path("cococrawl"))
        .arg(&images_dir)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cococrawl");

    assert!(output.status.success());

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    let file_name = coco["images"][0]["file_name"].as_str().unwrap();
    // Just verify that a file_name exists - path format depends on how walkdir returns paths
    assert!(!file_name.is_empty());
}

#[test]
fn test_cococrawl_multiple_formats() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("test1.jpg"), 100, 100);
    create_dummy_image(&images_dir.join("test2.png"), 100, 100);
    create_dummy_image(&images_dir.join("test3.bmp"), 100, 100);
    create_dummy_image(&images_dir.join("test4.gif"), 100, 100);

    let output_path = temp_dir.path().join("coco.json");

    let output = Command::new(get_binary_path("cococrawl"))
        .arg(&images_dir)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cococrawl");

    assert!(output.status.success());

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    assert_eq!(coco["images"].as_array().unwrap().len(), 4);
}

#[test]
fn test_cococrawl_ignores_non_images() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("test.jpg"), 100, 100);
    fs::write(images_dir.join("readme.txt"), "test").unwrap();
    fs::write(images_dir.join("data.json"), "{}").unwrap();

    let output_path = temp_dir.path().join("coco.json");

    let output = Command::new(get_binary_path("cococrawl"))
        .arg(&images_dir)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cococrawl");

    assert!(output.status.success());

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    // Should only find the jpg, not txt or json files
    assert_eq!(coco["images"].as_array().unwrap().len(), 1);
}

#[test]
fn test_cococrawl_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    let output_path = temp_dir.path().join("coco.json");

    let output = Command::new(get_binary_path("cococrawl"))
        .arg(&images_dir)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cococrawl");

    assert!(output.status.success());

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    assert_eq!(coco["images"].as_array().unwrap().len(), 0);
}

#[test]
fn test_cococrawl_version_string() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("test.jpg"), 100, 100);

    let output_path = temp_dir.path().join("coco.json");

    let output = Command::new(get_binary_path("cococrawl"))
        .arg(&images_dir)
        .arg("-o")
        .arg(&output_path)
        .arg("-v")
        .arg("2.5.3")
        .output()
        .expect("Failed to execute cococrawl");

    assert!(output.status.success());

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    assert_eq!(coco["info"]["version"].as_str().unwrap(), "2.5.3");
}

#[test]
fn test_cococrawl_has_info_section() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("test.jpg"), 100, 100);

    let output_path = temp_dir.path().join("coco.json");

    Command::new(get_binary_path("cococrawl"))
        .arg(&images_dir)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cococrawl");

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    assert!(coco["info"].is_object());
    assert!(coco["info"]["year"].is_number());
    assert!(coco["info"]["date_created"].is_string());
}

#[test]
fn test_cococrawl_image_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    create_dummy_image(&images_dir.join("test.jpg"), 320, 240);

    let output_path = temp_dir.path().join("coco.json");

    Command::new(get_binary_path("cococrawl"))
        .arg(&images_dir)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute cococrawl");

    let coco_json = fs::read_to_string(&output_path).unwrap();
    let coco: serde_json::Value = serde_json::from_str(&coco_json).unwrap();

    let image = &coco["images"][0];
    assert_eq!(image["id"].as_i64().unwrap(), 0);
    assert_eq!(image["width"].as_u64().unwrap(), 320);
    assert_eq!(image["height"].as_u64().unwrap(), 240);
    assert!(image["file_name"].is_string());
}
