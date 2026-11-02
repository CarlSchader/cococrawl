use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use tempfile::TempDir;

fn write_file(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn manifest(root: &Path, names: &[&str]) -> PathBuf {
    let images: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(id, name)| json!({"id": id, "width": 1, "height": 1, "file_name": name}))
        .collect();
    let path = root.join("dataset.json");
    write_file(
        &path,
        &serde_json::to_vec(&json!({"images": images, "annotations": []})).unwrap(),
    );
    path
}

fn copy(input: &Path, output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cococp"))
        .arg(input)
        .arg("-o")
        .arg(output)
        .output()
        .unwrap()
}

fn assert_success(output: Output) {
    assert!(
        output.status.success(),
        "cococp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(output: Output, message: &str) {
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(message),
        "unexpected error: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn output_images(root: &Path) -> Vec<Value> {
    let dataset: Value =
        serde_json::from_slice(&fs::read(root.join("dataset.json")).unwrap()).unwrap();
    dataset["images"].as_array().unwrap().clone()
}

#[test]
fn copying_consolidated_dataset_onto_itself_preserves_source() {
    let temp = TempDir::new().unwrap();
    let image = temp.path().join("images/x.png");
    write_file(&image, b"source image bytes must survive");
    let input = manifest(temp.path(), &["images/x.png"]);

    assert_success(copy(&input, temp.path()));

    assert_eq!(fs::read(image).unwrap(), b"source image bytes must survive");
    assert_eq!(output_images(temp.path())[0]["file_name"], "images/x.png");
}

#[test]
fn hard_link_destination_is_a_safe_noop() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source/x.png");
    write_file(&source, b"hard linked source");
    let output = temp.path().join("output");
    fs::create_dir_all(output.join("images")).unwrap();
    let destination = output.join("images/x.png");
    fs::hard_link(&source, &destination).unwrap();
    let input = manifest(temp.path(), &["source/x.png"]);

    assert_success(copy(&input, &output));

    assert_eq!(fs::read(source).unwrap(), b"hard linked source");
    assert_eq!(fs::read(destination).unwrap(), b"hard linked source");
}

#[cfg(unix)]
#[test]
fn symlink_destination_is_a_safe_noop() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source/x.png");
    write_file(&source, b"symlinked source");
    let output = temp.path().join("output");
    fs::create_dir_all(output.join("images")).unwrap();
    let destination = output.join("images/x.png");
    std::os::unix::fs::symlink(&source, &destination).unwrap();
    let input = manifest(temp.path(), &["source/x.png"]);

    assert_success(copy(&input, &output));

    assert_eq!(fs::read(source).unwrap(), b"symlinked source");
    assert_eq!(fs::read(destination).unwrap(), b"symlinked source");
}

#[cfg(unix)]
#[test]
fn symlink_source_and_output_directory_alias_preserve_source() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("images/x.png");
    write_file(&source, b"aliased source and output directory");
    fs::create_dir(temp.path().join("source")).unwrap();
    std::os::unix::fs::symlink(&source, temp.path().join("source/x.png")).unwrap();
    let output = temp.path().join("output_alias");
    std::os::unix::fs::symlink(temp.path(), &output).unwrap();
    let input = manifest(temp.path(), &["source/x.png"]);

    assert_success(copy(&input, &output));

    assert_eq!(
        fs::read(source).unwrap(),
        b"aliased source and output directory"
    );
}

#[test]
fn distinct_sources_with_same_basename_fail_before_any_mutation() {
    let temp = TempDir::new().unwrap();
    write_file(&temp.path().join("source/first.png"), b"first source");
    write_file(&temp.path().join("a/x.png"), b"source a");
    write_file(&temp.path().join("b/x.png"), b"source b");
    let input = manifest(temp.path(), &["source/first.png", "a/x.png", "b/x.png"]);
    let original_manifest = fs::read(&input).unwrap();
    let output = temp.path().join("new/output");

    assert_failure(copy(&input, &output), "Destination collision");

    assert!(!temp.path().join("new").exists());
    assert_eq!(fs::read(input).unwrap(), original_manifest);
    assert_eq!(fs::read(temp.path().join("a/x.png")).unwrap(), b"source a");
    assert_eq!(fs::read(temp.path().join("b/x.png")).unwrap(), b"source b");
}

#[test]
fn distinct_files_with_identical_bytes_still_collide() {
    let temp = TempDir::new().unwrap();
    write_file(&temp.path().join("a/x.png"), b"same bytes");
    write_file(&temp.path().join("b/x.png"), b"same bytes");
    let input = manifest(temp.path(), &["a/x.png", "b/x.png"]);
    let output = temp.path().join("output");

    assert_failure(copy(&input, &output), "Destination collision");
    assert!(!output.exists());
}

#[test]
fn unrelated_existing_destination_prevents_all_copies_and_manifest_changes() {
    let temp = TempDir::new().unwrap();
    write_file(&temp.path().join("source/first.png"), b"first source");
    write_file(&temp.path().join("source/x.png"), b"new source");
    let input = manifest(temp.path(), &["source/first.png", "source/x.png"]);
    let output = temp.path().join("output");
    write_file(&output.join("images/x.png"), b"existing destination");
    write_file(&output.join("dataset.json"), b"existing manifest");

    assert_failure(
        copy(&input, &output),
        "Refusing to overwrite existing destination",
    );

    assert!(!output.join("images/first.png").exists());
    assert_eq!(
        fs::read(output.join("images/x.png")).unwrap(),
        b"existing destination"
    );
    assert_eq!(
        fs::read(output.join("dataset.json")).unwrap(),
        b"existing manifest"
    );
    assert_eq!(
        fs::read(temp.path().join("source/x.png")).unwrap(),
        b"new source"
    );
}

#[cfg(unix)]
#[test]
fn unrelated_destination_symlink_cannot_overwrite_another_source() {
    let temp = TempDir::new().unwrap();
    write_file(&temp.path().join("source/x.png"), b"source x");
    let other_source = temp.path().join("source/y.png");
    write_file(&other_source, b"source y");
    let input = manifest(temp.path(), &["source/y.png", "source/x.png"]);
    let output = temp.path().join("output");
    fs::create_dir_all(output.join("images")).unwrap();
    std::os::unix::fs::symlink(&other_source, output.join("images/x.png")).unwrap();

    assert_failure(
        copy(&input, &output),
        "Refusing to overwrite existing destination",
    );

    assert!(!output.join("images/y.png").exists());
    assert_eq!(fs::read(other_source).unwrap(), b"source y");
}

#[cfg(unix)]
#[test]
fn dangling_destination_symlink_is_rejected_without_creating_its_target() {
    let temp = TempDir::new().unwrap();
    write_file(&temp.path().join("source/x.png"), b"source x");
    let input = manifest(temp.path(), &["source/x.png"]);
    let output = temp.path().join("output");
    fs::create_dir_all(output.join("images")).unwrap();
    let target = temp.path().join("must-not-be-created");
    std::os::unix::fs::symlink(&target, output.join("images/x.png")).unwrap();

    assert_failure(
        copy(&input, &output),
        "Could not verify existing destination",
    );

    assert!(!target.exists());
    assert!(!output.join("dataset.json").exists());
    assert!(
        fs::symlink_metadata(output.join("images/x.png"))
            .unwrap()
            .is_symlink()
    );
}

#[test]
fn repeated_source_references_share_one_output_and_keep_all_records() {
    let temp = TempDir::new().unwrap();
    let bytes = vec![42; 128 * 1024];
    write_file(&temp.path().join("source/x.png"), &bytes);
    let input = manifest(temp.path(), &vec!["source/x.png"; 128]);
    let output = temp.path().join("output");

    assert_success(copy(&input, &output));

    assert_eq!(fs::read(output.join("images/x.png")).unwrap(), bytes);
    let images = output_images(&output);
    assert_eq!(images.len(), 128);
    for (id, image) in images.iter().enumerate() {
        assert_eq!(image["id"], id);
        assert_eq!(image["file_name"], "images/x.png");
    }
}

#[test]
fn hard_link_source_aliases_can_share_a_destination() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("a/x.png");
    write_file(&source, b"shared source");
    fs::create_dir(temp.path().join("b")).unwrap();
    fs::hard_link(&source, temp.path().join("b/x.png")).unwrap();
    let input = manifest(temp.path(), &["a/x.png", "b/x.png"]);
    let output = temp.path().join("output");

    assert_success(copy(&input, &output));

    assert_eq!(fs::read(output.join("images/x.png")).unwrap(), b"shared source");
    let images = output_images(&output);
    assert_eq!(images.len(), 2);
    assert!(images.iter().all(|image| image["file_name"] == "images/x.png"));
}

#[cfg(unix)]
#[test]
fn symlink_source_aliases_can_share_a_destination() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("a/x.png");
    write_file(&source, b"shared source");
    fs::create_dir(temp.path().join("b")).unwrap();
    std::os::unix::fs::symlink(&source, temp.path().join("b/x.png")).unwrap();
    let input = manifest(temp.path(), &["a/x.png", "b/x.png"]);
    let output = temp.path().join("output");

    assert_success(copy(&input, &output));

    assert_eq!(fs::read(output.join("images/x.png")).unwrap(), b"shared source");
    assert_eq!(output_images(&output).len(), 2);
}

#[test]
fn missing_source_keeps_its_reference_and_warning() {
    let temp = TempDir::new().unwrap();
    write_file(&temp.path().join("source/x.png"), b"existing source");
    let input = manifest(temp.path(), &["source/x.png", "missing/x.png"]);
    let output = temp.path().join("output");

    let result = copy(&input, &output);
    assert!(String::from_utf8_lossy(&result.stderr).contains("Warning"));
    assert_success(result);

    let images = output_images(&output);
    assert_eq!(images[0]["file_name"], "images/x.png");
    assert_eq!(images[1]["file_name"], "missing/x.png");
    assert_eq!(
        fs::read(output.join("images/x.png")).unwrap(),
        b"existing source"
    );
}
