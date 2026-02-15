# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Wormhole is a URL shortener service written in Rust. It uses a workspace-based architecture with separate crates for different concerns.

## Development Environment

This project uses [devenv](https://devenv.sh) for reproducible development environments. The shell environment is managed via Nix and configured in `devenv.nix`.

**Required setup:**
- Run `direnv allow` to enter the development environment automatically
- The environment sets `PROTOC` for protobuf compilation
- Pre-commit hooks are configured for clippy, rustfmt, nixfmt, taplo, and yamlfmt

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

## Development Commands

The project includes a `justfile` for common development tasks:

```bash
# Start development infrastructure (Redis with Sentinel)
just dev-up

# View logs from Redis containers
just logs
```

## Architecture

### Workspace Structure

The project is organized as a Cargo workspace with seven crates:

- **`wormhole-core`**: Core types and traits shared across the workspace
- **`wormhole-storage`**: Storage implementations (in-memory repository)
- **`wormhole-shortener`**: Service implementation and code generators
- **`wormhole-proto-schema`**: gRPC protocol buffer definitions and generated code
- **`wormhole-gateway`**: HTTP API server (Axum) - the entry point for clients
- **`wormhole-redirector`**: Redirect handling with caching support (Redis, Moka)
- **`wormhole-test-infra`**: Test infrastructure for Redis HA setups

### Crate Dependencies

- `wormhole-core` has no internal dependencies and provides the foundation
- `wormhole-storage` depends on `wormhole-core`
- `wormhole-shortener` depends on `wormhole-core` and `wormhole-proto-schema`
- `wormhole-redirector` depends on `wormhole-core` and `wormhole-storage`
- `wormhole-gateway` depends on `wormhole-shortener` and `wormhole-proto-schema`
- `wormhole-test-infra` provides Redis test fixtures

### Core Types and Traits (in `wormhole-core`)

This crate defines the shared domain model:

1. **`Shortener`** (`shortener.rs`): High-level service trait
   - `shorten(params: ShortenParams) -> Result<ShortCode>`
   - `resolve(code) -> Result<Option<UrlRecord>>`
   - `delete(code) -> Result<bool>`

2. **`Repository`** / **`ReadRepository`** (`repository.rs`): Storage abstraction
   - `Repository` extends `ReadRepository` with write operations
   - `ReadRepository` provides `get()` and `exists()` for read-only access

3. **`UrlCache`** (`cache.rs`): Caching abstraction for URL records
   - `get_or_compute()` with single-flight support for request coalescing
   - Used by the redirector to cache lookups

4. **`ShortCode`** (`shortcode.rs`): Validated short code type (enum)
   - `ShortCode::Generated(ShortCodeBase58)` - system-generated codes
   - `ShortCode::Custom(String)` - user-provided custom aliases
   - Enforces 3-32 character length, alphanumeric/hyphens/underscores only
   - Use `ShortCode::new()` for validation, `new_unchecked()` for trusted input
   - Use `ShortCode::generated()` to create from `SlimId` or `ShortCodeBase58`

5. **`SlimId`** (`slim_id.rs`): 40-bit distributed ID generator
   - 30 bits timestamp (seconds since custom epoch)
   - 8 bits sequence number (resets every second)
   - 2 bits node ID (allows 4 nodes)
   - Uses `modular_bitfield` for compact representation

6. **`ShortCodeBase58`** (`base58.rs`): Base58-encoded short code
   - Encodes 8-byte `SlimId` as compact base58 string
   - Uses `SmolStr` for efficient string storage

7. **`UrlRecord`** (`repository.rs`): Stored URL record with expiration

8. **`Error`** (`error.rs`): Domain errors including `AliasConflict`, `InvalidUrl`

### Storage Implementation (in `wormhole-storage`)

**`InMemoryRepository`** (`memory.rs`): Thread-safe in-memory storage
- Uses `DashMap` for concurrent access (sharded locks for better concurrency)
- Handles expiration by checking `expire_at` on read
- Expired entries are lazily cleaned up on access

### Service Implementation (in `wormhole-shortener`)

1. **`ShortenerService<R, G>`** (`service.rs`): Concrete `Shortener` implementation
   - Composes a `Repository` and `Generator`
   - URL validation requires http/https scheme
   - Custom alias conflict detection
   - Expiration policy conversion using `jiff::Timestamp`

2. **`Generator`** (`generator.rs`): Short code generation trait
   - Implemented by `UniqueGenerator` (`generator/seq.rs`)
   - Sequential counter with configurable prefix
   - Pure generator (no storage interaction)

### Redirector Service (in `wormhole-redirector`)

**`RedirectorService<R>`** (`service.rs`): Resolves short codes to URLs
- Uses `ReadRepository` for read-only access
- Handles expiration checks

**Cache Implementations** (`cache/`):
- **`MokaUrlCache`**: In-memory cache using Moka (LRU eviction)
- **`RedisUrlCache`**: Redis-based cache with JSON serialization
- **`RedisHAUrlCache`**: High-availability Redis with Sentinel support
- **`LayeredCache`**: Multi-tier caching (e.g., local + remote)

**`CachedRepository<R, C>`** (`repository/cached.rs`): Decorator pattern
- Wraps any `ReadRepository` with a `UrlCache`
- Transparent caching with single-flight request coalescing
- Falls back to inner repository on cache miss

### gRPC Integration

The `wormhole-proto-schema` crate compiles `.proto` files at build time using `tonic-prost-build`. The build script is at `crates/wormhole-proto-schema/build.rs`.

Proto files are located at `crates/wormhole-proto-schema/proto/shortener/v1/shortener.proto`.

Current service definition:
- `ShortenerService.Create`: Creates a short URL

### Dependencies to Know

- **Time**: `jiff` crate for timestamp handling (not `chrono`)
- **Concurrency**: `DashMap` for concurrent in-memory storage
- **Async**: `tokio` runtime with `async-trait` for trait async methods
- **Error handling**: `thiserror` for defining error enums
- **Serialization**: `serde` for serialization
- **gRPC**: `tonic` (server/client), `prost` (protobuf)
- **IDs**: `modular_bitfield` for `SlimId`, `bs58` for base58 encoding
- **Caching**: `moka` for in-memory caching, `redis` crate for Redis

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

Integration tests for Redis are in `wormhole-redirector/tests/` and use `wormhole-test-infra` to spin up Redis with Sentinel for HA testing.
