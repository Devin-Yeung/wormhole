# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace. Core code lives in `crates/`, with one crate per concern:
- `wormhole-core`: shared domain types and traits
- `wormhole-storage`: storage implementations
- `wormhole-shortener`: short URL creation logic
- `wormhole-redirector`: lookup and caching
- `wormhole-gateway`: service entrypoint
- `wormhole-proto-schema`: protobuf/gRPC schema and codegen
- `wormhole-test-infra`: Redis test fixtures

Environment and operations files live at repo root:
- `devenv.nix`, `rust-toolchain.toml`, `justfile`
- Docker Compose files under `infra/dev/`

## Build, Test, and Development Commands
Run `direnv allow` (or enter `devenv shell`) first so toolchain paths and `PROTOC` are available.

- `cargo check --workspace`: fast compile check
- `cargo build --workspace`: build all crates
- `cargo test --locked --all-features --all-targets`: CI-aligned test run
- `cargo test -p wormhole-shortener`: run a single crate test suite
- `cargo fmt --check`: formatting check
- `cargo clippy --all-features --all-targets`: lints
- `just dev-up`: start local Redis/Sentinel stack
- `just logs`: inspect Redis container logs

## Coding Style & Naming Conventions
- Rust 2021 on stable toolchain; use default `rustfmt` formatting (4-space indentation).
- Follow Clippy guidance; fix warnings in any code you touch.
- Naming conventions:
  - `snake_case` for files, modules, and functions
  - `PascalCase` for structs, enums, and traits
  - `SCREAMING_SNAKE_CASE` for constants
- Prefer explicit domain names (for example `short_code`, `url_record`) over ambiguous abbreviations.
- Comments should explain intent and tradeoffs (the "why"), not obvious mechanics.
- Use `TODO:` comments when intentionally deferring a follow-up or edge case.

## Documentation & Design Notes
- Keep code readable as a narrative: explain decisions near the code they affect.
- For non-trivial logic, document why the chosen approach exists and what alternatives were rejected.
- Avoid over-abstraction when a small, well-documented sequential block is clearer.
- When omitting expected behavior intentionally, document the omission and rationale.

## Testing Guidelines
- Keep unit tests near implementation (`#[cfg(test)]` modules in `src/`).
- Use `#[tokio::test]` for async behavior.
- Put cross-component tests in crate-level `tests/` (for example `crates/wormhole-redirector/tests`).
- Redis integration/HA tests require Docker; run `just dev-up` before running them.
- No hard coverage gate is defined; new features and bug fixes should include regression tests.

## Commit & Pull Request Guidelines
- Follow Conventional Commits used in history: `feat:`, `fix:`, `refactor:`, `chore:` (optional scope such as `fix(ci): ...`).
- Keep each commit focused on one logical change.
- PRs should include:
  - concise summary of the change and intent
  - related issue link (if any)
  - exact verification commands you ran
- Explicitly call out protobuf, infra, or config behavior changes in the PR description.

## Problem-Framing (Avoid the XY Problem)
- Confirm the real goal before optimizing a proposed implementation detail.
- If a request appears narrow or unusually specific, ask: "What outcome are we trying to achieve?"
- Prefer solutions that address root intent (`X`) instead of only the proposed tactic (`Y`).
