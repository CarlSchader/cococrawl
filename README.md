# cococrawl

A fast, parallel image directory crawler that generates COCO (Common Objects in Context) dataset format JSON files.

## Overview

cococrawl recursively scans directories for image files and produces a COCO-formatted JSON manifest containing image metadata including dimensions, file paths, and timestamps. Built with Rust for performance, it uses parallel processing to handle large image collections efficiently.

## Features

- **Fast parallel processing** with Rayon
- **Progress tracking** with visual progress bar
- **Multiple image formats**: png, jpg, jpeg, gif, bmp, tiff, svg, webp
- **Flexible path handling**: relative (default) or absolute paths
- **Recursive directory traversal**
- **Metadata extraction**: dimensions, creation dates, file paths

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/cococrawl`.

## Usage

Basic usage:

```bash
cococrawl <DIRECTORY>...
```

With options:

```bash
cococrawl [OPTIONS] <DIRECTORIES>...
```

### Options

- `-o, --output <FILE>` - Output JSON file path (default: `coco.json`)
- `-v, --version-string <VERSION>` - Version string for COCO info section (default: `1.0.0`)
- `-a, --absolute-paths` - Use absolute paths instead of relative paths

### Examples

Crawl a single directory:

```bash
cococrawl ./images
```

Crawl multiple directories with custom output:

```bash
cococrawl ./images ./photos -o dataset.json
```

Use absolute paths:

```bash
cococrawl ./images --absolute-paths
```

## Output Format

The tool generates a JSON file following the COCO dataset format:

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
