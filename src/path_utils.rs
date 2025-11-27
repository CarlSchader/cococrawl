use std::path::{Path, PathBuf};
use anyhow::Result;

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

    let dataset_file_parent = dataset_file_path.parent().expect(format!("unable to get parent dir for {}", dataset_file_path.to_string_lossy()).as_str());

    if is_in_directory_tree(image_file_path, dataset_file_parent)? {
        Ok(
            image_file_path
            .strip_prefix(dataset_file_parent)?
            .to_path_buf()
        )
    } else {
        Ok(image_file_path.canonicalize()?)
    }
}
