# Agent Guidelines for cococrawl

## Build/Test Commands
- **Build**: `cargo build --release`
- **Test all**: `cargo test`
- **Test single**: `cargo test test_name`
- **Run binary**: `cargo run --bin cococrawl -- <args>` (or cococp, cococount, cocosplit)
- **Lint**: `cargo clippy` (if available)
- **Format check**: `cargo fmt --check`

## Code Style
- **Language**: Rust 2024 edition
- **Imports**: Group std imports, then external crates, then internal modules; use explicit imports
- **Types**: Use explicit types (u32, i64, f32) matching COCO spec; use Option for nullable fields
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE_CASE for constants
- **Error handling**: Use .unwrap() for prototyping, .expect() with messages for main code, Result for library functions
- **Parallelism**: Use rayon's par_iter() with indicatif's .progress() for parallel operations on collections
- **Serialization**: Use serde attributes (#[serde(skip_serializing_if = "Option::is_none")], custom deserializers)
- **Formatting**: Use rustfmt defaults (4-space indentation, 100 char line length)
- **Tests**: Place unit tests in #[cfg(test)] mod tests at bottom of files; test serialization/deserialization roundtrips
- **Clap**: Use derive macro with #[clap()] attributes for CLI parsing
