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

The project is organized as a Cargo workspace with four crates:

- **`wormhole-gateway`**: HTTP API server (Axum) - the entry point for clients
- **`wormhole-shortener`**: Core domain logic with trait-based abstractions
- **`wormhole-proto-schema`**: gRPC protocol buffer definitions and generated code
- **`wormhole-redirector`**: Redirect handling logic (placeholder)

### Key Abstractions (in `wormhole-shortener`)

The shortener crate defines three core traits:

1. **`Shortener`** (`shortener.rs`): High-level service trait for URL shortening operations
   - `shorten(params: ShortenParams) -> Result<ShortCode>`
   - `resolve(code) -> Result<Option<UrlRecord>>`
   - `delete(code) -> Result<bool>`

2. **`Repository`** (`repository.rs`): Storage abstraction for URL records
   - Implemented by `InMemoryRepository` using `DashMap` for concurrent access
   - Handles expiration logic transparently

3. **`Generator`** (`generator.rs`): Short code generation strategy
   - Implemented by `UniqueGenerator` (sequential with prefix)
   - Generators are pure functions that don't interact with storage

4. **`ShortCode`** (`shortcode.rs`): Validated short code type
   - Enforces 3-32 character length
   - Only allows alphanumeric, hyphens, and underscores
   - Use `ShortCode::new()` for validation, `new_unchecked()` for trusted input

### Service Implementation

`ShortenerService<R, G>` (`service.rs`) is the concrete implementation that composes a `Repository` and `Generator`. It handles:
- URL validation (scheme must be http/https)
- Custom alias conflict detection
- Expiration policy conversion (using `jiff::Timestamp`)

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
