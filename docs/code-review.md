# Code review findings

Review scope: the shared COCO model and all five CLI tools. Locations below refer to the code at review time. Findings were reproduced using temporary fixtures; the existing 99 tests and `cargo build --release` passed. Clippy and rustfmt were unavailable in the review environment.

## High priority

### 1. Copying a dataset onto itself destroys source images

- **Location:** `src/bin/cococp.rs:71–75`
- **Trigger:** A manifest references `images/x.png` and `cococp` targets that manifest's directory.
- **Observed:** `fs::copy` receives the same source and destination file; a 69-byte PNG became empty while the command returned success.
- **Remedy:** Detect identical files before copying, including hard-link and symlink aliases. Preserve source bytes and explicitly define same-file behavior.
- **Regression coverage:** Same-path and aliased-file copies must not truncate the source.

### 2. Duplicate basenames silently overwrite images

- **Location:** `src/bin/cococp.rs:68–71`
- **Trigger:** Distinct input images `a/x.png` and `b/x.png`.
- **Observed:** Both copy to `images/x.png`; both manifest records survive but reference a single output file, corrupting image/annotation associations.
- **Remedy:** Preflight destination collisions before mutation; reject collisions or choose unique destinations without losing either image.
- **Regression coverage:** Distinct images sharing a basename cannot silently overwrite one another, including existing destination files.

### 3. Merge assigns duplicate category and license IDs

- **Location:** `src/bin/cocomerge.rs:90–94,116–120`
- **Trigger:** Merge categories `(1,A)` with `(1,B),(2,C)`; the same sequence applies to licenses.
- **Observed:** Newly allocated IDs are not recorded in the seen sets, so B and C both receive ID 2 and their references become ambiguous.
- **Remedy:** Reserve every allocated ID and preserve correct annotation/category and image/license mappings.
- **Regression coverage:** Multiple collisions followed by explicit IDs must leave unique IDs and correct references.

### 4. Merge corrupts panoptic mask references

- **Location:** `src/bin/cocomerge.rs:235–240`
- **Trigger:** Two images have panoptic masks encoding segment ID 1.
- **Observed:** Merge renumbers one JSON segment to 2 without changing mask pixels. Segment IDs encode pixel values and may legitimately repeat across images.
- **Remedy:** Preserve per-mask segment IDs while remapping category and image references.
- **Regression coverage:** Repeated segment IDs across images must remain consistent with unchanged PNG masks.

### 5. Failed operations can erase existing manifests

- **Location:** `src/bin/cocosplit.rs:64–65`; `src/bin/cocomerge.rs:55–56`; early output creation also occurs in `src/bin/cococrawl.rs:42`.
- **Trigger:** `cocosplit input.json -b val.json -o val.json`, or an in-place merge with a missing referenced image.
- **Observed:** Split erases its blacklist before reading it. Merge truncates its input before later path processing fails. Existing unrelated outputs are also vulnerable.
- **Remedy:** Read and validate inputs before replacing output. Write a temporary file and atomically replace the destination only after successful processing and checked writes. Path computation must not require truncating the final destination.
- **Regression coverage:** Failures preserve existing manifests byte-for-byte; successful in-place operations and new output paths still work.

### 6. Buffered write failures incorrectly report success

- **Location:** `src/bin/cococp.rs:106–108`, `src/bin/cococrawl.rs:112–114`, `src/bin/cocosplit.rs:164–167`, `src/bin/cocomerge.rs:327–329`.
- **Trigger:** Write a small manifest to `/dev/full` (for copy, via an output manifest symlink).
- **Observed:** All four commands return success because the final `BufWriter` flush happens during destruction, which discards errors.
- **Remedy:** Explicitly flush and propagate errors before reporting success or committing an atomic output. Define safe behavior for special files and symlink destinations.
- **Regression coverage:** A writer failing on the final buffered flush must produce an error; existing output must survive failed transactional writes.

## Other correctness issues

### 7. Seeded splits are not reproducible

- **Location:** `src/bin/cocosplit.rs:85–96`
- **Trigger:** Repeat the same input and `--shuffle 42 -c 5` in separate processes.
- **Observed:** Different images are selected because seeded shuffling starts from randomized `HashMap` iteration order.
- **Remedy:** Establish deterministic input ordering before shuffling.
- **Regression coverage:** Repeated seeded runs select the same ordered image IDs, including blacklist and annotated-only filtering.

### 8. Semantic deduplication violates the Hash/Eq contract

- **Location:** `src/lib.rs:78–86,306–377`; consumed by merge's category/license hash sets.
- **Trigger:** Merge otherwise identical categories or licenses with IDs 1 and 2.
- **Observed:** Both survive deduplication. Custom equality ignores IDs, but derived hashes include them.
- **Remedy:** Hash exactly the fields used by equality for licenses and every category variant.
- **Regression coverage:** Equal values with differing IDs have equal hashes, deduplicate in sets, and merge with correctly remapped references.

### 9. Standard COCO timestamps are rejected

- **Location:** `src/lib.rs:32,52`
- **Trigger:** Image `date_captured` is `2013-11-14 17:02:52`, or info `date_created` is `2017/09/01`.
- **Observed:** Deserializing the dataset fails because `DateTime<Utc>` expects a timezone-bearing datetime.
- **Remedy:** Accept conventional COCO date formats as well as existing RFC3339 values, with an explicit policy for timezone-free values, or preserve date strings.
- **Regression coverage:** Conventional dates, RFC3339 values, absent/null optional dates, and serialization roundtrips.

### 10. Compressed RLE masks are rejected

- **Location:** `src/lib.rs:427–429`
- **Trigger:** A segmentation contains `{"size":[2,2],"counts":"04"}`.
- **Observed:** `counts: Vec<u32>` rejects the compressed string representation and the annotation cannot deserialize.
- **Remedy:** Support both uncompressed integer arrays and compressed strings without losing their representation.
- **Regression coverage:** Both forms roundtrip through annotations and the CLI tools.

### 11. Crawl errors silently produce incomplete datasets

- **Location:** `src/bin/cococrawl.rs:50–52`
- **Trigger:** Crawl a nonexistent directory or encounter another traversal failure.
- **Observed:** `filter_map(Result::ok)` discards errors. A nonexistent input returned success and replaced an existing manifest with an empty dataset.
- **Remedy:** Propagate traversal failures with useful context and avoid replacing output on failure.
- **Regression coverage:** Missing inputs fail without damaging existing output; genuinely empty directories remain valid inputs.

## Implementation coordination

Related fixes should be coordinated rather than independently editing the same code:

- Copy safety: findings 1–2.
- Merge integrity and semantic hashing: findings 3–4 and 8.
- Transactional output and checked writes: findings 5–6, across all manifest-writing tools.
- Deterministic splitting: finding 7.
- COCO serialization compatibility: findings 9–10.
- Crawl traversal errors: finding 11; coordinate with transactional output.

Each fix needs regression tests. Separate worktree branches should be reviewed and integrated deliberately; do not assume independent branches merge without conflicts.
