# cococrawl

A suite of fast, parallel tools for managing COCO (Common Objects in Context) datasets.

## Overview

This toolkit provides utilities for creating and managing COCO-formatted image datasets. Built with Rust for performance, all tools use parallel processing to handle large image collections efficiently with visual progress tracking.

## Tools

### cococrawl

Recursively scans directories for image files and generates a COCO-formatted JSON manifest containing image metadata including dimensions, file paths, and timestamps.

### cococp

Consolidates a COCO dataset by copying all referenced images into a single directory structure while preserving original filenames and updating the manifest paths.

### cococount

Displays statistics about a COCO dataset including the number of images and annotations.

### cocosplit

Creates dataset splits (train/val/test) from a COCO dataset with optional blacklisting to exclude images from previously created splits. Maintains image-annotation relationships.

## Features

- **Fast parallel processing** with Rayon
- **Progress tracking** with visual progress bars
- **Multiple image formats**: png, jpg, jpeg, gif, bmp, tiff, svg, webp
- **Flexible path handling**: relative or absolute paths
- **Recursive directory traversal**
- **Metadata extraction**: dimensions, creation dates, file paths

## Installation

```bash
cargo build --release
```

Binaries will be available in `target/release/`:
- `target/release/cococrawl`
- `target/release/cococp`
- `target/release/cococount`
- `target/release/cocosplit`

## Usage

### cococrawl

Generate a COCO manifest from image directories.

**Basic usage:**

```bash
cococrawl <DIRECTORY>...
```

**Options:**

- `-o, --output <FILE>` - Output JSON file path (default: `coco.json`)
- `-v, --version-string <VERSION>` - Version string for COCO info section (default: `1.0.0`)
- `-a, --absolute-paths` - Use absolute paths instead of relative paths

**Examples:**

```bash
# Crawl a single directory
cococrawl ./images

# Crawl multiple directories with custom output
cococrawl ./images ./photos -o dataset.json

# Use absolute paths
cococrawl ./images --absolute-paths
```

### cococp

Copy and consolidate a COCO dataset into a standardized directory structure.

**Basic usage:**

```bash
cococp <COCO_JSON_FILE>
```

**Options:**

- `-o, --output-dir-path <DIR>` - Output directory path (default: `coco-dataset`)

**Examples:**

```bash
# Consolidate dataset to default directory
cococp coco.json

# Specify custom output directory
cococp coco.json -o my-dataset
```

**Output structure:**

```
my-dataset/
├── coco.json          # Updated manifest with new paths
└── images/
    ├── img1.jpg       # Original filenames preserved
    ├── img2.png
    └── ...
```

### cococount

Display statistics about a COCO dataset.

**Basic usage:**

```bash
cococount <COCO_JSON_FILE>
```

**Example:**

```bash
cococount dataset.json
```

**Output:**

```
Coco File: dataset.json
Images: 50000
Annotations: 150000
```

### cocosplit

Create dataset splits from a COCO dataset with random shuffling and optional blacklisting.

**Basic usage:**

```bash
cocosplit <COCO_JSON_FILE>
```

**Options:**

- `-o, --output <FILE>` - Output JSON file path (default: `split.json`)
- `-c, --count <NUMBER>` - Number of images to include in the split (default: all non-blacklisted images)
- `-b, --blacklist-file <FILE>` - COCO JSON file(s) containing images to exclude (can be specified multiple times)
- `-s, --seed <NUMBER>` - Random seed for reproducible shuffling

**Examples:**

```bash
# Create validation set with 10,000 images
cocosplit dataset.json -o val-set.json -c 10000

# Create test set with 20,000 images, excluding validation set
cocosplit dataset.json -o test-set.json -c 20000 -b val-set.json

# Create training set with remaining images (excluding val and test)
cocosplit dataset.json -o train-set.json -b val-set.json -b test-set.json

# Use a seed for reproducible splits
cocosplit dataset.json -o val-set.json -c 10000 -s 42
```

**Notes:**

- Preserves image-annotation relationships
- Images are randomly shuffled before selection
- Blacklisted images are completely excluded from the output
- Without `-c`, all non-blacklisted images are included

## COCO Format

The tools work with JSON files following the [COCO dataset format](https://cocodataset.org/#format-data):

```json
{
  "info": {
    "year": 2025,
    "version": "1.0.0",
    "description": "",
    "contributor": "",
    "url": "",
    "date_created": "2025-10-05T06:22:54.043666509Z"
  },
  "images": [
    {
      "id": 0,
      "width": 854,
      "height": 1410,
      "file_name": "path/to/image.jpg",
      "license": 0,
      "flickr_url": "",
      "coco_url": "",
      "date_captured": "2025-10-04T10:33:22.330144689Z"
    }
  ],
  "annotations": []
}
```
