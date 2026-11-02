//! Transactional JSON output for regular files. Symlink and special-file destinations
//! are rejected and rechecked just before replacement. Callers should
//! finish reading inputs and computing their output before invoking this helper.

use std::fs::{self, File, Permissions};
use std::io::{self, BufWriter, Write};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use tempfile::Builder;

use crate::path_utils::resolve_output_path;

/// Write JSON to a temporary file in the destination directory, then atomically
/// replace the destination only after serialization, flush, and file sync succeed.
/// Existing regular-file permissions are retained; new files use tempfile's private
/// permissions. Ownership, ACLs, and hard-link identity are not preserved.
///
/// This is not a lock against concurrent writers, and does not sync the directory
/// for crash durability. The final destination check cannot prevent a concurrent
/// directory-entry change between that check and the atomic rename.
pub fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    write_atomic(path, |file| write_json_buffered(file, value))
}

/// Reject existing symlinks and special destinations using metadata only, without
/// opening them. Missing destinations or parents are allowed for copy preflight;
/// nothing is created. This is an early check, not a reservation: the atomic writer
/// still validates the destination again before replacing it.
pub fn validate_output_destination(path: &Path) -> Result<()> {
    destination_permissions(path).map(|_| ())
}

fn destination_permissions(path: &Path) -> Result<Option<Permissions>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(Some(metadata.permissions())),
        Ok(_) => bail!(
            "Refusing non-regular output destination {} (symlinks and special files are not supported)",
            path.display()
        ),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error)
            .with_context(|| format!("Could not inspect output destination {}", path.display())),
    }
}

fn write_atomic(path: &Path, write: impl FnOnce(&mut File) -> Result<()>) -> Result<()> {
    let path = resolve_output_path(path)?;
    validate_output_destination(&path)?;
    let parent = path.parent().context("Output path has no parent directory")?;
    let mut temporary = Builder::new()
        .prefix(".cococrawl-")
        .tempfile_in(parent)
        .with_context(|| format!("Could not create temporary output in {}", parent.display()))?;

    write(temporary.as_file_mut())
        .with_context(|| format!("Could not write JSON output {}", path.display()))?;

    // Inspect again: a symlink or special file may have appeared while writing.
    if let Some(permissions) = destination_permissions(&path)? {
        temporary
            .as_file()
            .set_permissions(permissions)
            .context("Could not preserve output file permissions")?;
    }
    temporary
        .as_file()
        .sync_all()
        .context("Could not sync JSON output")?;
    validate_output_destination(&path)?;
    // Discard PersistError's temporary file before returning its I/O error, so cleanup
    // does not depend on when the caller drops the returned error.
    temporary
        .persist(&path)
        .map_err(|error| error.error)
        .with_context(|| format!("Could not replace output destination {}", path.display()))?;
    Ok(())
}

fn write_json_buffered<W: Write, T: Serialize + ?Sized>(writer: W, value: &T) -> Result<()> {
    let mut writer = BufWriter::new(writer);
    serde_json::to_writer_pretty(&mut writer, value).context("Could not serialize JSON output")?;
    writer.flush().context("Could not flush JSON output")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serializer;
    use serde::ser::{Error, SerializeSeq};
    use tempfile::TempDir;

    struct SerializationFailure;

    impl Serialize for SerializationFailure {
        fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
            let mut sequence = serializer.serialize_seq(Some(2))?;
            // Ensure some bytes reach the temporary file before serialization fails.
            sequence.serialize_element(&"x".repeat(16 * 1024))?;
            Err(S::Error::custom("injected serialization failure"))
        }
    }

    struct FailingWriter<W> {
        inner: W,
        fail_write: bool,
    }

    impl<W: Write> Write for FailingWriter<W> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.fail_write {
                Err(io::Error::other("injected write failure"))
            } else {
                self.inner.write(bytes)
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("injected flush failure"))
        }
    }

    fn assert_preserved_and_clean(directory: &TempDir, path: &Path) {
        assert_eq!(fs::read(path).unwrap(), b"original manifest\n");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn early_validation_allows_missing_parents_without_creating_them() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("missing/nested/output.json");
        validate_output_destination(&path).unwrap();
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
    }

    #[test]
    fn serialization_failure_preserves_existing_output_and_cleans_temporary() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("output.json");
        fs::write(&path, b"original manifest\n").unwrap();
        let error = write_json_atomic(&path, &SerializationFailure).unwrap_err();
        assert!(format!("{error:#}").contains("injected serialization failure"));
        assert_preserved_and_clean(&directory, &path);
    }

    #[test]
    fn serialization_failure_does_not_create_destination() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("new.json");
        assert!(write_json_atomic(&path, &SerializationFailure).is_err());
        assert!(!path.exists());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
    }

    #[test]
    fn write_failure_during_serialization_preserves_output() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("output.json");
        fs::write(&path, b"original manifest\n").unwrap();
        let error = write_atomic(&path, |file| {
            write_json_buffered(
                FailingWriter {
                    inner: file,
                    fail_write: true,
                },
                &"x".repeat(16 * 1024),
            )
        })
        .unwrap_err();
        assert!(format!("{error:#}").contains("Could not serialize JSON output"));
        assert_preserved_and_clean(&directory, &path);
    }

    #[test]
    fn write_failure_on_final_buffer_drain_preserves_output() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("output.json");
        fs::write(&path, b"original manifest\n").unwrap();
        let error = write_atomic(&path, |file| {
            write_json_buffered(
                FailingWriter {
                    inner: file,
                    fail_write: true,
                },
                &vec![1, 2],
            )
        })
        .unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("Could not flush JSON output"));
        assert!(message.contains("injected write failure"));
        assert_preserved_and_clean(&directory, &path);
    }

    #[test]
    fn writer_failure_on_final_flush_preserves_output() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("output.json");
        fs::write(&path, b"original manifest\n").unwrap();
        let error = write_atomic(&path, |file| {
            write_json_buffered(
                FailingWriter {
                    inner: file,
                    fail_write: false,
                },
                &vec![1, 2],
            )
        })
        .unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("Could not flush JSON output"));
        assert!(message.contains("injected flush failure"));
        assert_preserved_and_clean(&directory, &path);
    }

    #[test]
    fn success_replaces_output_and_cleans_temporary() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("output.json");
        fs::write(&path, "old").unwrap();
        write_json_atomic(&path, &vec![1, 2]).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "[\n  1,\n  2\n]");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn success_preserves_existing_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let directory = TempDir::new().unwrap();
        let path = directory.path().join("output.json");
        fs::write(&path, "old").unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o640)).unwrap();
        write_json_atomic(&path, &vec![1, 2]).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }

    #[test]
    fn directory_appearing_during_write_is_rejected_and_temporary_is_cleaned() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("output.json");
        let error = write_atomic(&path, |file| {
            write_json_buffered(file, &vec![1, 2])?;
            fs::create_dir(&path)?;
            Ok(())
        })
        .unwrap_err();
        assert!(format!("{error:#}").contains("Refusing non-regular output"));
        assert!(path.is_dir());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_appearing_during_write_is_rejected_without_touching_target() {
        use std::os::unix::fs::symlink;

        let directory = TempDir::new().unwrap();
        let path = directory.path().join("output.json");
        let target = directory.path().join("target.json");
        fs::write(&target, b"original manifest\n").unwrap();
        let error = write_atomic(&path, |file| {
            write_json_buffered(file, &vec![1, 2])?;
            symlink(&target, &path)?;
            Ok(())
        })
        .unwrap_err();
        assert!(format!("{error:#}").contains("Refusing non-regular output"));
        assert_eq!(fs::read(&target).unwrap(), b"original manifest\n");
        assert_eq!(fs::read_link(&path).unwrap(), target);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }
}
