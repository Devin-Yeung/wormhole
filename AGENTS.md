# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace. Core code lives in `crates/`, with one crate per concern:
- `wormhole-core`: shared domain types/traits
- `wormhole-storage`: storage implementations
- `wormhole-shortener`: short URL creation logic
- `wormhole-redirector`: lookup + caching
- `wormhole-gateway`: service entrypoint
- `wormhole-proto-schema`: protobuf/gRPC schema and codegen
- `wormhole-test-infra`: Redis test fixtures

Environment and ops files live at root: `devenv.nix`, `rust-toolchain.toml`, `justfile`, and infra Compose files under `infra/dev/`.

## Build, Test, and Development Commands
Use `direnv allow` (or `devenv shell`) first so toolchain and `PROTOC` are set.

- `cargo check --workspace`: fast compile checks
- `cargo build --workspace`: build all crates
- `cargo test --locked --all-features --all-targets`: CI-aligned full test run
- `cargo test -p wormhole-shortener`: test one crate
- `cargo fmt --check`: formatting check
- `cargo clippy --all-features --all-targets`: linting
- `just dev-up`: start local Redis/Sentinel stack
- `just logs`: inspect Redis container logs

## Coding Style & Naming Conventions
- Rust 2021 on stable toolchain; use rustfmt defaults (4-space indentation).
- Follow Clippy guidance; fix warnings in code you touch.
- Naming: `snake_case` for files/modules/functions, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Prefer explicit domain names (for example `short_code`, `url_record`) over short abbreviations.
- Write comments for intent and tradeoffs (the "why"), not obvious mechanics.
- Add `TODO:` comments when intentionally deferring edge cases or follow-up work.

## Testing Guidelines
- Keep unit tests near implementation (`#[cfg(test)]` modules in `src/`).
- Use `#[tokio::test]` for async behavior.
- Put cross-component tests in crate-level `tests/` (for example `crates/wormhole-redirector/tests`).
- Redis integration/HA tests require Docker; run `just dev-up` before executing them.
- No hard coverage gate is defined; new features and bug fixes should include regression tests.

## Commit & Pull Request Guidelines
- Follow Conventional Commits used in history: `feat:`, `fix:`, `refactor:`, `chore:`, with optional scope (for example `fix(ci): ...`).
- Keep each commit focused on one logical change.
- PRs should include a clear summary, related issue link (if any), and commands you ran to validate changes.
- If changing protobuf, infra, or config behavior, call it out explicitly in the PR description.
