use std::collections::HashSet;
use clap::Parser;
//use rayon::prelude::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Directories to crawl positional arguments
    #[clap(required = true)]
    directories: Vec<String>,
}

const IMAGE_EXTENSIONS: [&str; 8] = ["png", "jpg", "jpeg", "gif", "bmp", "tiff", "svg", "webp"];

fn main() {
    let args = Args::parse();

    let extension_set: HashSet<&str> = IMAGE_EXTENSIONS.iter().cloned().collect();

    let found_files: Vec<String> = args.directories.iter().flat_map(|dir| {
        walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext_str| extension_set.contains(&ext_str.to_lowercase().as_str()))
            })
            .map(|entry| entry.path().to_string_lossy().to_string())
    }).collect();

    println!("Found {} image files", found_files.len());
}
