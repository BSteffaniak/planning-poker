# Planning Poker

A cross-platform planning poker application built with Rust and HyperChad, supporting both web and desktop interfaces.

## Features

- **Cross-platform**: Runs on web browsers and as native desktop applications
- **Real-time collaboration**: WebSocket-based real-time communication
- **Multiple voting systems**: Fibonacci, T-shirt sizes, powers of two, or custom scales
- **Session management**: Persistent game sessions with SQLite or PostgreSQL
- **Owner controls**: Game owners can start voting, reveal votes, and reset sessions
- **Modern UI**: Built with HyperChad for consistent cross-platform experience

## Architecture

This project uses a modular workspace structure:

- **`packages/poker`** - Core game logic and state management
- **`packages/models`** - Shared data models and message types
- **`packages/session`** - Session management and persistence layer
- **`packages/database`** - Database abstraction supporting SQLite and PostgreSQL
- **`packages/websocket`** - WebSocket communication layer
- **`packages/api`** - REST API endpoints
- **`packages/server`** - WebSocket server application
- **`packages/app`** - Cross-platform client application
- **`packages/ui`** - Shared UI components and state management
- **`packages/config`** - Configuration management

## Technology Stack

- **Backend**: Rust with Actix Web for HTTP/WebSocket server
- **Frontend**: HyperChad UI framework with multiple renderers:
  - **Desktop**: Egui renderer for native applications
  - **Web**: HTML renderer for browser-based clients
- **Database**: Switchy database abstraction (SQLite for development, PostgreSQL for production)
- **Real-time**: WebSocket communication for live updates
- **Build**: Cargo workspace with comprehensive CI/CD

## Quick Start

### Prerequisites

- Rust stable (see `rust-toolchain.toml`)
- For development: Nix (optional, see `shell.nix`)

### Development Setup

1. **Clone the repository**:

   ```bash
   git clone https://github.com/planning-poker/planning-poker.git
   cd planning-poker
   ```

2. **Enter development environment** (with Nix):

   ```bash
   nix-shell
   ```

   Or install dependencies manually:

   - pkg-config, cmake, openssl-dev
   - For desktop: GTK3, X11/Wayland libraries
   - For database: SQLite3, PostgreSQL client

3. **Build the workspace**:

   ```bash
   # On NixOS systems:
   nix-shell --run "cargo build"

   # Or directly:
   cargo build
   ```

4. **Run tests**:

   ```bash
   # On NixOS systems:
   nix-shell --run "cargo test --workspace --all-features"

   # Or directly:
   cargo test --workspace --all-features
   ```

### Running the Application

#### Start the Server

```bash
# On NixOS systems:
nix-shell --run "cargo run --bin planning-poker-server"

# Or directly:
cargo run --bin planning-poker-server
```

Options:

- `--port 8080` - Server port (default: 8080)
- `--host 0.0.0.0` - Server host (default: 0.0.0.0)
- `--database-url sqlite://poker.db` - Database URL

#### Start the Desktop Client

```bash
# On NixOS systems:
nix-shell --run "cargo run --bin planning-poker-app"

# Or directly (desktop is the default feature):
cargo run --bin planning-poker-app
```

#### Start the Web Client

```bash
# On NixOS systems:
nix-shell --run "cargo run --bin planning-poker-app --features web"

# Or directly:
cargo run --bin planning-poker-app --features web
```

## Configuration

### Environment Variables

- `PLANNING_POKER_HOST` - Server host
- `PLANNING_POKER_PORT` - Server port
- `DATABASE_URL` - Database connection string
- `RUST_LOG` - Logging level

### Configuration File

Create a `config.toml` file:

```toml
[server]
host = "0.0.0.0"
port = 8080
cors_origins = ["*"]

database_url = "sqlite://planning_poker.db"

[logging]
level = "info"
format = "pretty"
```

## Database Setup

### SQLite (Development)

SQLite databases are created automatically:

```bash
cargo run --bin planning-poker-server --database-url sqlite://poker.db
```

### PostgreSQL (Production)

1. Create a PostgreSQL database
2. Set the connection string:
   ```bash
   export DATABASE_URL="postgresql://user:password@localhost/planning_poker"
   ```
3. Run migrations (TODO: implement migrations)

## API Documentation

### REST Endpoints

- `POST /api/v1/games` - Create a new game
- `GET /api/v1/games/{id}` - Get game details
- `GET /api/v1/ws` - WebSocket endpoint

### WebSocket Messages

#### Client → Server

```json
{"type": "JoinGame", "game_id": "uuid", "player_name": "string"}
{"type": "CastVote", "value": "string"}
{"type": "StartVoting", "story": "string"}
{"type": "RevealVotes"}
{"type": "ResetVoting"}
{"type": "LeaveGame"}
```

#### Server → Client

```json
{"type": "GameJoined", "game": {...}, "players": [...]}
{"type": "PlayerJoined", "player": {...}}
{"type": "PlayerLeft", "player_id": "uuid"}
{"type": "VotingStarted", "story": "string"}
{"type": "VoteCast", "player_id": "uuid", "has_voted": true}
{"type": "VotesRevealed", "votes": [...]}
{"type": "VotingReset"}
{"type": "Error", "message": "string"}
```

## Development

### Code Style

This project follows Rust standard formatting and linting:

```bash
# On NixOS systems:
nix-shell --run "cargo fmt --all"
nix-shell --run "cargo clippy --workspace --all-targets --all-features -- -D warnings"

# Or directly:
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Testing

Run the full test suite:

```bash
# On NixOS systems:
nix-shell --run "cargo test --workspace --all-features"

# Or directly:
cargo test --workspace --all-features

# Run tests for a specific package:
cargo test -p planning_poker_models
```

### Building for Production

```bash
# On NixOS systems:
nix-shell --run "cargo build --release --workspace"

# Or directly:
cargo build --release --workspace
```

## Deployment

### Docker

Build the server image:

```bash
docker build -t planning-poker-server .
```

Run with Docker Compose:

```bash
docker-compose up
```

### Native Binaries

The CI system builds native binaries for:

- Linux (x86_64)
- macOS (x86_64, ARM64)
- Windows (x86_64)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test --workspace --all-features`
6. Format code: `cargo fmt --all`
7. Check lints: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
8. Submit a pull request

## License

This project is licensed under the Mozilla Public License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [HyperChad](https://github.com/MoosicBox/MoosicBox/tree/master/packages/hyperchad) UI framework
- Uses [Switchy](https://github.com/MoosicBox/MoosicBox/tree/master/packages) database abstraction
- Inspired by Planning Poker methodology for agile estimation
