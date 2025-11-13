use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use indicatif::ParallelProgressIterator;
use clap::Parser;
use serde_json;
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// coco JSON file path
    #[clap(required = true)]
    coco_file: PathBuf,

    /// JSON output path
    #[clap(short, long, default_value = "split.json")]
    output: String,

    /// number of images to create the split with
    /// if not provided, all images not in the blacklisted sets will be used
    #[clap(short, long)]
    count: Option<usize>,

    /// blacklist dataset JSON file paths
    /// cocosplit dataset.json -o val-set.json -c 10000
    /// cocosplit dataset.json -o test-set.json -c 20000 -b val-set.json 
    /// cocosplit dataset.json -o train-set.json -b test-set.json -b val-set.json 
    #[clap(short, long)]
    blacklist_file: Vec<PathBuf>
}

fn main() {
    let args = Args::parse();

    let coco_json = fs::read_to_string(&args.coco_file).expect("Could not read COCO JSON file");
    let coco_json_file_name = args.coco_file.file_name().unwrap().to_string_lossy();

    let output_file = File::create(&args.output).expect("Could not create output file");

    // Make directory for output if it doesn't exist
    fs::create_dir_all(&args.output_dir_path).expect("Could not create output directory");

    let images_output_path = PathBuf::from(&args.output_dir_path).join("images");
    fs::create_dir_all(&images_output_path).expect("Could not create images output directory");

    let mut coco_file: cococrawl::CocoFile = serde_json::from_str(&coco_json).expect("Could not parse COCO JSON");

    // Iterate over images and copy them to the output directory
    let images_count = coco_file.images.len() as u64;
    let digits = ((images_count as f64).log10().floor() as usize) + 1;
    coco_file.images.par_iter_mut().progress_count(images_count).for_each(|image| {
        let src_path = PathBuf::from(&image.file_name);
        if src_path.exists() && src_path.is_file() {
            let file_extension = src_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            // output file name should be padded id with leading zeros + original extension
            let file_name = format!("{:0width$}.{}", image.id, file_extension, width = digits);
            let dest_path = images_output_path.join(file_name);
            let file_name = dest_path.file_name().unwrap();
            fs::copy(&src_path, &dest_path).expect("Could not copy image file");
            // change the file_name in the image struct to the new relative path
            image.file_name = format!("images/{}", file_name.to_string_lossy());
        } else {
            eprintln!("Warning: Source image file does not exist or is not a file: {:?}", src_path);
        }
    });

    // Write updated COCO JSON to output directory
    let output_coco_path = PathBuf::from(&args.output_dir_path).join(coco_json_file_name.to_string());
    let output_file = File::create(&output_coco_path).expect("Could not create output COCO JSON file");
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &coco_file).expect("Could not write COCO JSON to output file");
}
