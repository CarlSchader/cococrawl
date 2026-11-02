use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use tempfile::TempDir;

const WRITERS: [&str; 4] = ["cococrawl", "cococp", "cocosplit", "cocomerge"];

fn binary(name: &str) -> &'static str {
    match name {
        "cococrawl" => env!("CARGO_BIN_EXE_cococrawl"),
        "cococp" => env!("CARGO_BIN_EXE_cococp"),
        "cocosplit" => env!("CARGO_BIN_EXE_cocosplit"),
        "cocomerge" => env!("CARGO_BIN_EXE_cocomerge"),
        _ => panic!("unknown binary {name}"),
    }
}

fn write_dataset(path: &Path, images: &[(i64, &str)]) {
    let images: Vec<Value> = images
        .iter()
        .map(|(id, name)| json!({"id": id, "width": 1, "height": 1, "file_name": name}))
        .collect();
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "images": images,
            "annotations": [],
        }))
        .unwrap(),
    )
    .unwrap();
}

fn fixture() -> TempDir {
    let directory = TempDir::new().unwrap();
    fs::create_dir(directory.path().join("images")).unwrap();
    image::RgbImage::new(1, 1)
        .save(directory.path().join("images/one.png"))
        .unwrap();
    image::RgbImage::new(1, 1)
        .save(directory.path().join("images/two.png"))
        .unwrap();
    write_dataset(
        &directory.path().join("input.json"),
        &[(1, "images/one.png"), (2, "images/two.png")],
    );
    directory
}

fn read_dataset(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn assert_success(output: &Output) {
    assert!(output.status.success(), "command failed: {output:?}");
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded: {output:?}"
    );
}

fn assert_no_temporaries(directory: &Path) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        assert!(!entry.file_name().to_string_lossy().starts_with(".cococrawl-"));
    }
}

#[test]
fn split_reads_blacklist_before_replacing_the_same_output() {
    let directory = fixture();
    let blacklist = directory.path().join("val.json");
    write_dataset(&blacklist, &[(1, "images/one.png")]);
    let output = Command::new(binary("cocosplit"))
        .current_dir(directory.path())
        .args(["input.json", "-b", "val.json", "-o", "val.json"])
        .output()
        .unwrap();
    assert_success(&output);
    let result = read_dataset(&blacklist);
    assert_eq!(result["images"].as_array().unwrap().len(), 1);
    assert_eq!(result["images"][0]["id"], 2);
    assert_eq!(result["images"][0]["file_name"], "images/two.png");
    assert_no_temporaries(directory.path());
}

#[test]
fn split_failure_preserves_blacklist_when_it_is_also_the_output() {
    let directory = fixture();
    let blacklist = directory.path().join("val.json");
    write_dataset(&blacklist, &[(1, "images/one.png")]);
    let original = fs::read(&blacklist).unwrap();
    fs::remove_file(directory.path().join("images/two.png")).unwrap();
    let output = Command::new(binary("cocosplit"))
        .current_dir(directory.path())
        .args(["input.json", "-b", "val.json", "-o", "val.json"])
        .output()
        .unwrap();
    assert_failure(&output);
    assert_eq!(fs::read(&blacklist).unwrap(), original);
    assert_no_temporaries(directory.path());
}

#[test]
fn invalid_blacklist_equal_to_output_is_preserved() {
    let directory = fixture();
    let blacklist = directory.path().join("invalid.json");
    let original = b"not JSON, but must not be truncated\n";
    fs::write(&blacklist, original).unwrap();
    let output = Command::new(binary("cocosplit"))
        .current_dir(directory.path())
        .args(["input.json", "-b", "invalid.json", "-o", "invalid.json"])
        .output()
        .unwrap();
    assert_failure(&output);
    assert_eq!(fs::read(&blacklist).unwrap(), original);
}

#[test]
fn in_place_merge_failure_preserves_input_bytes() {
    let directory = fixture();
    let input = directory.path().join("input.json");
    let original = fs::read(&input).unwrap();
    fs::remove_file(directory.path().join("images/two.png")).unwrap();
    let output = Command::new(binary("cocomerge"))
        .current_dir(directory.path())
        .args(["input.json", "-o", "input.json"])
        .output()
        .unwrap();
    assert_failure(&output);
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_no_temporaries(directory.path());
}

#[test]
fn processing_failure_preserves_unrelated_output_and_does_not_create_new_output() {
    for name in ["cocosplit", "cocomerge"] {
        let directory = fixture();
        fs::remove_file(directory.path().join("images/two.png")).unwrap();
        let existing = directory.path().join("existing.json");
        let original = b"unrelated existing output\n";
        fs::write(&existing, original).unwrap();
        for output_name in ["existing.json", "new.json"] {
            let output = Command::new(binary(name))
                .current_dir(directory.path())
                .args(["input.json", "-o", output_name])
                .output()
                .unwrap();
            assert_failure(&output);
        }
        assert_eq!(fs::read(&existing).unwrap(), original);
        assert!(!directory.path().join("new.json").exists());
        assert_no_temporaries(directory.path());
    }
}

#[test]
fn split_and_merge_support_successful_in_place_output() {
    for name in ["cocosplit", "cocomerge"] {
        let directory = fixture();
        let output = Command::new(binary(name))
            .current_dir(directory.path())
            .args(["input.json", "-o", "input.json"])
            .output()
            .unwrap();
        assert_success(&output);
        let result = read_dataset(&directory.path().join("input.json"));
        assert_eq!(result["images"].as_array().unwrap().len(), 2);
        for image in result["images"].as_array().unwrap() {
            let file_name = Path::new(image["file_name"].as_str().unwrap());
            assert!(file_name.is_relative());
            assert!(directory.path().join(file_name).is_file());
        }
        assert_no_temporaries(directory.path());
    }
}

#[test]
fn all_writers_support_new_output_names_and_relative_or_absolute_image_paths() {
    for name in WRITERS {
        for absolute in [false, true] {
            let directory = fixture();
            let mut command = Command::new(binary(name));
            command.current_dir(directory.path());
            command.arg(if name == "cococrawl" {
                "images"
            } else {
                "input.json"
            });
            let destination = if name == "cococp" {
                command.args(["-o", "copied"]);
                directory.path().join("copied/input.json")
            } else {
                command.args(["-o", "new.json"]);
                directory.path().join("new.json")
            };
            if absolute {
                command.arg("--absolute-paths");
            }
            assert!(!destination.exists());
            let output = command.output().unwrap();
            assert_success(&output);
            let result = read_dataset(&destination);
            assert_eq!(result["images"].as_array().unwrap().len(), 2);
            for image in result["images"].as_array().unwrap() {
                let file_name = Path::new(image["file_name"].as_str().unwrap());
                assert_eq!(file_name.is_absolute(), absolute);
                if !absolute {
                    assert!(file_name.starts_with("images"));
                }
                assert!(destination.parent().unwrap().join(file_name).is_file());
            }
            assert_no_temporaries(destination.parent().unwrap());
        }
    }
}

#[test]
fn images_outside_output_directory_remain_absolute() {
    for name in ["cococrawl", "cocosplit", "cocomerge"] {
        let directory = fixture();
        fs::create_dir(directory.path().join("output")).unwrap();
        let output = Command::new(binary(name))
            .current_dir(directory.path())
            .arg(if name == "cococrawl" {
                "images"
            } else {
                "input.json"
            })
            .args(["-o", "output/new.json"])
            .output()
            .unwrap();
        assert_success(&output);
        let result = read_dataset(&directory.path().join("output/new.json"));
        for image in result["images"].as_array().unwrap() {
            let file_name = Path::new(image["file_name"].as_str().unwrap());
            assert!(file_name.is_absolute());
            assert!(file_name.is_file());
        }
    }
}

#[cfg(unix)]
#[test]
fn all_writers_preserve_existing_output_on_serialization_failure() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    for name in WRITERS {
        let directory = TempDir::new().unwrap();
        let source = directory
            .path()
            .join(OsString::from_vec(b"non-utf8-\xff".to_vec()));
        fs::create_dir_all(source.join("images")).unwrap();
        image::RgbImage::new(1, 1)
            .save(source.join("images/one.png"))
            .unwrap();
        let input = source.join("input.json");
        write_dataset(&input, &[(1, "images/one.png")]);
        let output_dir = source.join("output");
        fs::create_dir(&output_dir).unwrap();
        let destination = output_dir.join("input.json");
        let original = b"existing manifest must survive serialization failure\n";
        fs::write(&destination, original).unwrap();

        let mut command = Command::new(binary(name));
        // Crawl takes UTF-8 directory arguments, but can discover non-UTF-8 children.
        command.arg(if name == "cococrawl" {
            directory.path()
        } else {
            &input
        });
        command.args(["--absolute-paths", "-o"]);
        command.arg(if name == "cococp" {
            &output_dir
        } else {
            &destination
        });
        let output = command.output().unwrap();
        assert_failure(&output);
        assert!(String::from_utf8_lossy(&output.stderr).contains("Could not serialize JSON output"));
        assert_eq!(fs::read(&destination).unwrap(), original);
        assert_no_temporaries(&output_dir);
    }
}

// Use empty inputs to exercise the output policy independently of traversal,
// image copying, and image-path validation.
fn empty_writer_command(name: &str, root: &Path, destination: &Path) -> Command {
    let input = root.join("input.json");
    write_dataset(&input, &[]);
    let images = root.join("empty-images");
    fs::create_dir_all(&images).unwrap();
    let mut command = Command::new(binary(name));
    command.arg(if name == "cococrawl" { &images } else { &input });
    command.arg("-o");
    command.arg(if name == "cococp" {
        destination.parent().unwrap()
    } else {
        destination
    });
    command
}

fn special_destination(root: &Path) -> PathBuf {
    let parent = root.join("output");
    fs::create_dir(&parent).unwrap();
    // cococp keeps the input basename.
    parent.join("input.json")
}

fn assert_rejected(output: &Output) {
    assert_failure(output);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Refusing non-regular output"),
        "missing destination diagnostic: {output:?}"
    );
}

#[test]
fn all_writers_reject_directory_destinations_without_modifying_them() {
    for name in WRITERS {
        let directory = TempDir::new().unwrap();
        let destination = special_destination(directory.path());
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("keep"), b"original").unwrap();
        let output = empty_writer_command(name, directory.path(), &destination)
            .output()
            .unwrap();
        assert_rejected(&output);
        assert_eq!(fs::read(destination.join("keep")).unwrap(), b"original");
        assert_no_temporaries(destination.parent().unwrap());
    }
}

#[cfg(unix)]
#[test]
fn all_writers_reject_existing_and_dangling_output_symlinks() {
    use std::os::unix::fs::symlink;

    for name in WRITERS {
        for target_exists in [false, true] {
            let directory = TempDir::new().unwrap();
            let destination = special_destination(directory.path());
            let target = directory.path().join("target.json");
            if target_exists {
                fs::write(&target, b"do not replace or redirect\n").unwrap();
            }
            symlink(&target, &destination).unwrap();
            let output = empty_writer_command(name, directory.path(), &destination)
                .output()
                .unwrap();
            assert_rejected(&output);
            assert_eq!(fs::read_link(&destination).unwrap(), target);
            if target_exists {
                assert_eq!(fs::read(&target).unwrap(), b"do not replace or redirect\n");
            } else {
                assert!(!target.exists());
            }
            assert_no_temporaries(destination.parent().unwrap());
        }
    }
}

#[cfg(unix)]
#[test]
fn all_writers_reject_socket_destinations() {
    use std::os::unix::fs::FileTypeExt;
    use std::os::unix::net::UnixListener;

    for name in WRITERS {
        let directory = TempDir::new().unwrap();
        let destination = special_destination(directory.path());
        let _socket = UnixListener::bind(&destination).unwrap();
        let output = empty_writer_command(name, directory.path(), &destination)
            .output()
            .unwrap();
        assert_rejected(&output);
        assert!(
            fs::symlink_metadata(&destination)
                .unwrap()
                .file_type()
                .is_socket()
        );
        assert_no_temporaries(destination.parent().unwrap());
    }
}

#[cfg(unix)]
fn run_with_timeout(command: &mut Command) -> Output {
    use std::process::Stdio;
    use std::thread;
    use std::time::{Duration, Instant};

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            child.kill().expect("Could not kill timed-out child");
            let output = child.wait_with_output().unwrap();
            panic!("command blocked on special output for 5 seconds: {output:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
fn assert_copy_rejects_fifo_manifest(symlink_output: bool) {
    use std::os::unix::fs::{FileTypeExt, MetadataExt, symlink};

    let directory = fixture();
    let input = directory.path().join("input.json");
    let original_input = fs::read(&input).unwrap();
    let originals: Vec<_> = ["images/one.png", "images/two.png"]
        .iter()
        .map(|name| {
            let path = directory.path().join(name);
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    let destination = special_destination(directory.path());
    let fifo = if symlink_output {
        directory.path().join("target.fifo")
    } else {
        destination.clone()
    };
    let created = Command::new("mkfifo")
        .arg(&fifo)
        .output()
        .expect("POSIX mkfifo must be available for FIFO regression tests");
    assert_success(&created);
    let original_fifo = fs::symlink_metadata(&fifo).unwrap();
    assert!(original_fifo.file_type().is_fifo());
    if symlink_output {
        symlink(&fifo, &destination).unwrap();
    }

    let output = run_with_timeout(
        Command::new(binary("cococp"))
            .arg(&input)
            .arg("-o")
            .arg(destination.parent().unwrap()),
    );
    assert_rejected(&output);
    let current_fifo = fs::symlink_metadata(&fifo).unwrap();
    assert!(current_fifo.file_type().is_fifo());
    assert_eq!(current_fifo.dev(), original_fifo.dev());
    assert_eq!(current_fifo.ino(), original_fifo.ino());
    if symlink_output {
        assert_eq!(fs::read_link(&destination).unwrap(), fifo);
    }
    assert_eq!(fs::read(&input).unwrap(), original_input);
    for (path, bytes) in originals {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    // Rejection must happen before preflight identity checks or any copy mutation.
    assert!(!destination.parent().unwrap().join("images").exists());
    assert_no_temporaries(destination.parent().unwrap());
}

#[cfg(unix)]
#[test]
fn cococp_rejects_fifo_manifest_with_nonempty_input_without_blocking() {
    assert_copy_rejects_fifo_manifest(false);
}

#[cfg(unix)]
#[test]
fn cococp_rejects_symlink_to_fifo_manifest_with_nonempty_input_without_blocking() {
    assert_copy_rejects_fifo_manifest(true);
}

#[cfg(target_os = "linux")]
#[test]
fn dev_full_is_rejected_not_replaced_or_reported_as_success() {
    use std::os::unix::fs::{FileTypeExt, MetadataExt, symlink};

    let full = Path::new("/dev/full");
    let original = fs::symlink_metadata(full).unwrap();
    assert!(original.file_type().is_char_device());
    for name in WRITERS {
        let directory = TempDir::new().unwrap();
        let destination = if name == "cococp" {
            let destination = special_destination(directory.path());
            symlink(full, &destination).unwrap();
            destination
        } else {
            full.to_path_buf()
        };
        let output = empty_writer_command(name, directory.path(), &destination)
            .output()
            .unwrap();
        assert_rejected(&output);
        let current = fs::symlink_metadata(full).unwrap();
        assert!(current.file_type().is_char_device());
        assert_eq!(current.ino(), original.ino());
        assert_eq!(current.rdev(), original.rdev());
        if name == "cococp" {
            assert_eq!(fs::read_link(destination).unwrap(), full);
        }
    }
}
