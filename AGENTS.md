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
```

## Server/App Runtime Restrictions

- **DO NOT run server commands**: Never execute `cargo run --bin planning-poker-server` or `cargo run --bin planning-poker-app`
- **Interactive processes hang**: These commands start interactive/infinite runtime processes that will hang the agent
- **Build/test only**: Only use build, test, format, and lint commands - never attempt to serve the application
- **Verification approach**: Use `cargo build` to verify the server/app compiles correctly

## Code Style

- Use `Result<T, E>` for error handling, never `unwrap()` in production code
- Import order: std, external crates, workspace crates, local modules
- Use `tracing` for logging, not `println!` or `log`
- Document public APIs with `///` doc comments
- Use `snake_case` for functions/variables, `PascalCase` for types/enums
- Prefer explicit types over `auto`/inference in public APIs
- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` for data models
- Handle `Option` and `Result` explicitly, avoid `.unwrap()` except in tests

## Frontend Technology

- **Uses hyperchad, NOT htmx**: This project uses a custom library called "hyperchad" that resembles htmx but is not htmx
- **Limited feature set**: hyperchad does not have all the features of htmx - check existing code patterns before assuming htmx functionality exists
- **Custom implementation**: When working with frontend interactions, refer to existing hyperchad usage patterns in the codebase

## Architecture

Modular workspace: models (pure data) → poker (business logic) → session/database → websocket/api → server/app/ui
