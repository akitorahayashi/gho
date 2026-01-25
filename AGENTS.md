# gho Development Overview

## Project Summary
`gho` is a GitHub operator CLI for multi-account workflows. It provides account management, repository operations, and pull request listing with support for switching between multiple GitHub identities. Tokens are stored securely in macOS Keychain.

## Tech Stack
- **Language**: Rust
- **CLI Parsing**: `clap`
- **Serialization**: `serde`, `serde_json`
- **Keychain**: `keyring`
- **HTTP Client**: `reqwest`
- **Interactive Selection**: `inquire`
- **Error Handling**: `thiserror`
- **Development Dependencies**:
  - `assert_cmd`
  - `assert_fs`
  - `predicates`
  - `serial_test`
  - `tempfile`

## Coding Standards
- **Formatter**: `rustfmt` is used for code formatting. Key rules include a maximum line width of 100 characters, crate-level import granularity, and grouping imports by standard, external, and crate modules.
- **Linter**: `clippy` is used for linting, with a strict policy of treating all warnings as errors (`-D warnings`).

## Naming Conventions
- **Structs and Enums**: `PascalCase` (e.g., `Account`, `Protocol`)
- **Functions and Variables**: `snake_case` (e.g., `get_token`, `active_account`)
- **Modules**: `snake_case` (e.g., `account.rs`, `keychain.rs`)

## Key Commands
- **Build (Debug)**: `cargo build`
- **Build (Release)**: `cargo build --release`
- **Format Check**: `cargo fmt --check`
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Test**: `cargo test --all-targets --all-features`

## Testing Strategy
- **Unit Tests**: Located within the `src/` directory alongside the code they test.
- **Command Logic Tests**: Found in `src/commands/`, utilizing mock storage to test business logic in isolation.
- **Integration Tests**: Housed in the `tests/` directory, covering CLI workflows and API.

## Architectural Highlights
- **Two-tier structure**: `src/main.rs` handles CLI parsing, `src/lib.rs` exposes public APIs.
- **Command modules**: `src/commands/` contains domain-specific commands (account, repo, pr).
- **Storage abstraction**: `src/storage.rs` defines a `Storage` trait for accounts and state.
- **Keychain integration**: `src/keychain.rs` handles secure token storage.
- **GitHub API client**: `src/github.rs` provides direct API access without shelling out to `gh`.
- **Storage Layout**: Config stored in `~/.config/gho/` with `accounts.json` and `state.json`.
