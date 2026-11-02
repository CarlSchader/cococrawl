use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

/// Resolve the existing output directory without creating or following the final file.
pub fn resolve_output_path(output_path: &Path) -> Result<PathBuf> {
    let file_name = output_path
        .file_name()
        .with_context(|| format!("Output path has no file name: {}", output_path.display()))?;
    let parent = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = parent
        .canonicalize()
        .with_context(|| format!("Could not resolve output directory {}", parent.display()))?;
    Ok(parent.join(file_name))
}

pub fn is_in_directory_tree(file_path: &Path, directory: &Path) -> Result<bool> {
    let file_path = file_path.canonicalize()?;
    let directory = directory.canonicalize()?;

    Ok(file_path.starts_with(&directory))
}

pub fn create_coco_image_path(
    dataset_file_path: &Path,
    image_file_path: &Path,
    force_absolute: bool,
) -> Result<PathBuf> {
    if force_absolute {
        return Ok(image_file_path.canonicalize()?)
    }

    let absolute_dataset_file_path = resolve_output_path(dataset_file_path)?;
    let dataset_file_parent = absolute_dataset_file_path.parent().expect(format!("unable to get parent dir for {}", dataset_file_path.to_string_lossy()).as_str());

    if is_in_directory_tree(image_file_path, dataset_file_parent)? {
        Ok(
            image_file_path
            .canonicalize()?
            .strip_prefix(dataset_file_parent)?
            .to_path_buf()
        )
    } else {
        Ok(image_file_path.canonicalize()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolves_new_filename_without_creating_it() {
        let directory = TempDir::new().unwrap();
        let output = directory.path().join("new.json");
        assert_eq!(
            resolve_output_path(&output).unwrap(),
            directory.path().canonicalize().unwrap().join("new.json")
        );
        assert!(!output.exists());
    }

    #[test]
    fn resolves_bare_filename_against_current_directory() {
        assert_eq!(
            resolve_output_path(Path::new("new.json")).unwrap(),
            Path::new(".").canonicalize().unwrap().join("new.json")
        );
    }

    #[test]
    fn missing_parent_is_not_created() {
        let directory = TempDir::new().unwrap();
        let output = directory.path().join("missing/new.json");
        assert!(resolve_output_path(&output).is_err());
        assert!(!output.parent().unwrap().exists());
    }

    #[cfg(unix)]
    #[test]
    fn image_paths_are_relative_to_resolved_output_parent() {
        use std::fs;
        use std::os::unix::fs::symlink;

        let directory = TempDir::new().unwrap();
        let real_parent = directory.path().join("real");
        let linked_parent = directory.path().join("link");
        fs::create_dir(&real_parent).unwrap();
        symlink(&real_parent, &linked_parent).unwrap();
        let image = real_parent.join("image.png");
        fs::write(&image, "image bytes").unwrap();
        let output = linked_parent.join("new.json");
        assert_eq!(
            create_coco_image_path(&output, &image, false).unwrap(),
            Path::new("image.png")
        );
        assert_eq!(
            create_coco_image_path(&output, &image, true).unwrap(),
            image.canonicalize().unwrap()
        );
        assert!(!output.exists());
    }
}
