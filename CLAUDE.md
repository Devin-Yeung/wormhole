# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Wormhole is a URL shortener service written in Rust. It uses a workspace-based architecture with separate crates for different concerns.

## Development Environment

This project uses [devenv](https://devenv.sh) for reproducible development environments. The shell environment is managed via Nix and configured in `devenv.nix`.

**Required setup:**
- Run `direnv allow` to enter the development environment automatically
- The environment sets `PROTOC` for protobuf compilation
- Pre-commit hooks are configured for clippy, rustfmt, nixfmt, and taplo

## Build Commands

```bash
# Build all workspace crates
cargo build

# Build in release mode
cargo build --release

# Check without building (faster feedback)
cargo check

# Run Clippy lints
cargo clippy --workspace --all-targets

# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p wormhole-shortener

# Run a specific test
cargo test -p wormhole-shortener shorten_with_custom_alias

# Format code
cargo fmt

# Check for unused dependencies
cargo machete

# Check licenses and security advisories
cargo deny check
```

## Architecture

### Workspace Structure

The project is organized as a Cargo workspace with five crates:

- **`wormhole-core`**: Core types and traits shared across the workspace
- **`wormhole-shortener`**: Service implementation and code generators
- **`wormhole-proto-schema`**: gRPC protocol buffer definitions and generated code
- **`wormhole-gateway`**: HTTP API server (Axum) - the entry point for clients
- **`wormhole-redirector`**: Redirect handling logic (placeholder)

### Crate Dependencies

- `wormhole-core` has no internal dependencies and provides the foundation
- `wormhole-shortener` depends on `wormhole-core` and `wormhole-proto-schema`
- `wormhole-gateway` depends on `wormhole-shortener` and `wormhole-proto-schema`

### Core Types and Traits (in `wormhole-core`)

This crate defines the shared domain model:

1. **`Shortener`** (`shortener.rs`): High-level service trait
   - `shorten(params: ShortenParams) -> Result<ShortCode>`
   - `resolve(code) -> Result<Option<UrlRecord>>`
   - `delete(code) -> Result<bool>`

2. **`Repository`** / **`ReadRepository`** (`repository.rs`): Storage abstraction
   - `Repository` extends `ReadRepository` with write operations
   - `InMemoryRepository` (in `repository/memory.rs`) uses `DashMap` for concurrent access

3. **`UrlCache`** (`cache.rs`): Caching abstraction for URL records
   - `get_or_compute()` with single-flight support for request coalescing
   - Used by the redirector to cache lookups

4. **`ShortCode`** (`shortcode.rs`): Validated short code type
   - Enforces 3-32 character length
   - Only allows alphanumeric, hyphens, and underscores
   - Use `ShortCode::new()` for validation, `new_unchecked()` for trusted input

5. **`UrlRecord`** (`repository.rs`): Stored URL record with expiration

6. **`Error`** (`error.rs`): Domain errors including `AliasConflict`, `InvalidUrl`

### Service Implementation (in `wormhole-shortener`)

1. **`ShortenerService<R, G>`** (`service.rs`): Concrete `Shortener` implementation
   - Composes a `Repository` and `Generator`
   - Handles URL validation (scheme must be http/https)
   - Custom alias conflict detection
   - Expiration policy conversion (using `jiff::Timestamp`)

2. **`Generator`** (`generator.rs`): Short code generation trait
   - Implemented by `UniqueGenerator` (`generator/seq.rs`) - sequential with prefix
   - Generators are pure functions that don't interact with storage

### gRPC Integration

The `wormhole-proto-schema` crate compiles `.proto` files at build time using `tonic-prost-build`. The build script is at `crates/wormhole-proto-schema/build.rs`.

Proto files are located at `crates/wormhole-proto-schema/proto/shortener/v1/shortener.proto`.

### Dependencies to Know

- **Time**: `jiff` crate for timestamp handling (not `chrono`)
- **Concurrency**: `DashMap` for concurrent in-memory storage
- **Async**: `tokio` runtime with `async-trait` for trait async methods
- **Error handling**: `thiserror` for defining error enums
- **Serialization**: `serde` for serialization
- **gRPC**: `tonic` (server/client), `prost` (protobuf)

## Testing Patterns

Tests are co-located in source files using `#[cfg(test)]` modules. The project uses:
- `tokio::test` for async tests
- `InMemoryRepository` and `UniqueGenerator` for test fixtures

Example test pattern from `service.rs`:
```rust
fn test_service() -> ShortenerService<InMemoryRepository, UniqueGenerator> {
    let repo = InMemoryRepository::new();
    let generator = UniqueGenerator::with_prefix("wh");
    ShortenerService::new(repo, generator)
}
```
