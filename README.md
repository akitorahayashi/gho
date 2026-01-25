# gho

`gho` is a GitHub operator CLI for multi-account workflows. It provides account management,
repository operations, and pull request listing with support for switching between multiple
GitHub identities (personal, work, etc.).

## Features

- **Multi-account support**: Store and switch between multiple GitHub accounts.
- **Keychain integration**: Tokens are stored securely in macOS Keychain.
- **Repository operations**: List and clone repositories with protocol preference (SSH/HTTPS).
- **PR listing**: View open pull requests with merge status.
- **Organization support**: Bulk clone repositories from organizations.

## Installation

```bash
cargo install --path .
# or
cargo build --release
```

## Quick Start

```bash
# Add an account
gho account add personal --username myuser --token ghp_xxxx

# List accounts
gho account list

# Switch active account
gho account use personal

# List repositories
gho repo list

# Clone a repository
gho repo clone owner/repo

# List open PRs
gho pr list owner/repo
```

## Commands

### Account Management

| Command | Alias | Description |
|---------|-------|-------------|
| `gho account add` | `gho a add` | Add a new GitHub account |
| `gho account list` | `gho a ls` | List all configured accounts |
| `gho account use [id]` | `gho a u` | Switch active account (interactive if no id) |
| `gho account show` | `gho a show` | Show active account details |
| `gho account remove <id>` | `gho a rm` | Remove an account |

### Repository Operations

| Command | Alias | Description |
|---------|-------|-------------|
| `gho repo list` | `gho r ls` | List repositories for active account |
| `gho repo clone <repo>` | `gho r cl` | Clone a repository |
| `gho repo clone --org <org>` | | Bulk clone from organization |

### Pull Requests

| Command | Alias | Description |
|---------|-------|-------------|
| `gho pr list [repo]` | `gho p ls` | List open PRs (detects repo from git) |

## Storage

Configuration is stored in `~/.config/gho/`:

- `accounts.json`: Account definitions and active account ID
- `state.json`: Runtime state (last org, last repo)

Tokens are stored in macOS Keychain under the service `gho`.

## Environment Variables

- `GH_TOKEN` / `GITHUB_TOKEN`: Override token from Keychain
- `GITHUB_REPOSITORY`: Provide repository context for PR operations

## Development Commands

- `cargo build` — build a debug binary.
- `cargo build --release` — build the optimized release binary.
- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings` — format check and lint.
- `cargo test --all-targets --all-features` — run all tests.

## Project Structure

```
gho/
├── src/
│   ├── main.rs           # CLI parsing (clap)
│   ├── lib.rs            # Public API exports
│   ├── config.rs         # Config paths
│   ├── error.rs          # AppError definitions
│   ├── models.rs         # Data models (Account, Repository, etc.)
│   ├── storage.rs        # JSON file storage
│   ├── keychain.rs       # macOS Keychain integration
│   ├── github.rs         # GitHub API client
│   └── commands/         # Command implementations
│       ├── mod.rs
│       ├── account.rs    # Account management
│       ├── repo.rs       # Repository operations
│       └── pr.rs         # Pull request operations
└── tests/
    └── ...
```
