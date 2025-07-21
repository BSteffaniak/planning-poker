# AI Agent Instructions for Planning Poker

## NixOS Environment

- **Always use shell.nix**: Run commands with `nix-shell --run "command"` on NixOS systems
- **Example**: `nix-shell --run "cargo build"` or `nix-shell --run "cargo test"`
- **Shell provides**: Rust toolchain, GUI libraries (egui/gtk), PostgreSQL, system dependencies

## Git Usage Restrictions

- **READ-ONLY git operations**: Only use `git status`, `git log`, `git diff`, `git show`, `git branch -v`
- **NO modifications**: Never use `git add`, `git commit`, `git push`, `git pull`, `git rebase`, `git reset`, `git merge`
- **View only**: Agents should only inspect git state, never change it

## Build/Test Commands

```bash
cargo build                                                    # Build workspace
cargo test --workspace --all-features                         # Run all tests
cargo test -p planning_poker_models                           # Run single package tests
cargo fmt --all                                               # Format code
cargo clippy --workspace --all-targets --all-features -- -D warnings  # Lint
cargo run --bin planning-poker-server                         # Start server
cargo run --bin planning-poker-app --features desktop         # Start desktop app
```

## Code Style

- Use `Result<T, E>` for error handling, never `unwrap()` in production code
- Import order: std, external crates, workspace crates, local modules
- Use `tracing` for logging, not `println!` or `log`
- Document public APIs with `///` doc comments
- Use `snake_case` for functions/variables, `PascalCase` for types/enums
- Prefer explicit types over `auto`/inference in public APIs
- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` for data models
- Handle `Option` and `Result` explicitly, avoid `.unwrap()` except in tests

## Architecture

Modular workspace: models (pure data) → poker (business logic) → session/database → websocket/api → server/app/ui
