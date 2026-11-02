use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn crawl(inputs: &[&Path], output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cococrawl"))
        .args(inputs)
        .arg("--output")
        .arg(output)
        .output()
        .unwrap()
}

fn assert_path_error(result: &Output, path: &Path) {
    assert!(!result.status.success(), "unexpected success: {result:?}");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains(&*path.to_string_lossy()), "{stderr}");
}

#[test]
fn missing_single_input_fails_without_creating_output() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("missing");
    let output = temp.path().join("coco.json");
    assert_path_error(&crawl(&[&missing], &output), &missing);
    assert!(!output.exists());
}

#[test]
fn mixed_existing_and_missing_inputs_fail_without_partial_output() {
    let temp = TempDir::new().unwrap();
    let existing = temp.path().join("images");
    fs::create_dir(&existing).unwrap();
    image::RgbImage::new(2, 3).save(existing.join("image.png")).unwrap();
    let missing = temp.path().join("missing");
    let output = temp.path().join("coco.json");
    for inputs in [[existing.as_path(), missing.as_path()], [missing.as_path(), existing.as_path()]] {
        assert_path_error(&crawl(&inputs, &output), &missing);
        assert!(!output.exists());
    }
}

#[test]
fn traversal_failure_preserves_existing_output() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("missing");
    let output = temp.path().join("coco.json");
    let original = b"existing manifest must survive\n";
    fs::write(&output, original).unwrap();
    assert_path_error(&crawl(&[&missing], &output), &missing);
    assert_eq!(fs::read(&output).unwrap(), original);
}

#[test]
fn genuinely_empty_directory_succeeds() {
    let temp = TempDir::new().unwrap();
    let empty = temp.path().join("empty");
    fs::create_dir(&empty).unwrap();
    let output = temp.path().join("coco.json");
    let result = crawl(&[&empty], &output);
    assert!(result.status.success(), "{result:?}");
    let json: serde_json::Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    assert_eq!(json["images"], serde_json::json!([]));
}

#[cfg(unix)]
#[test]
fn unreadable_nested_directory_or_image_preserves_output() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let input = temp.path().join("images");
    let nested = input.join("restricted");
    fs::create_dir_all(&nested).unwrap();
    let image = input.join("image.png");
    image::RgbImage::new(2, 3).save(&image).unwrap();
    let output = temp.path().join("coco.json");
    let original = b"preserve this manifest";
    fs::write(&output, original).unwrap();

    fs::set_permissions(&nested, fs::Permissions::from_mode(0o000)).unwrap();
    // Root or a process with DAC override can still read mode-000 directories.
    if fs::read_dir(&nested).is_err() {
        let result = crawl(&[&input], &output);
        assert_path_error(&result, &nested);
        assert_eq!(fs::read(&output).unwrap(), original);
    }
    fs::set_permissions(&nested, fs::Permissions::from_mode(0o700)).unwrap();

    fs::set_permissions(&image, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::File::open(&image).is_err() {
        let result = crawl(&[&input], &output);
        assert_path_error(&result, &image);
        assert_eq!(fs::read(&output).unwrap(), original);
    }
    fs::set_permissions(&image, fs::Permissions::from_mode(0o600)).unwrap();
}
