# Planning Poker Simulator

Deterministic simulation testing framework for the Planning Poker application using the simvar harness.

## Features

- **Multi-player game simulation**: Test concurrent players joining, voting, and leaving games
- **HTTP API simulation**: Simulate REST API calls and hyperchad form submissions
- **Database consistency testing**: Verify state consistency under concurrent operations
- **Time-based event simulation**: Test session timeouts and time-sensitive operations
- **Fault injection**: Simulate various failure scenarios for robust testing

## Architecture

The simulator tests the actual Planning Poker architecture using deterministic simulation:
- **HTTP REST API**: `/api/v1/games` endpoints for game management via simulated TCP
- **Hyperchad forms**: Form-based interactions for joining games and voting
- **Database operations**: SQLite database with session management
- **Switchy runtime**: All async operations use switchy's deterministic primitives
- **Simvar network**: TCP connections go through simvar's network simulation
- **No real networking**: Everything runs within the deterministic simulation environment

## Usage

Run simulations with:

```bash
cargo run --bin planning-poker-simulator
```

## Test Scenarios

- **Basic game flow**: Create game, join as player, cast votes, get results via HTTP
- **Network partitions**: Test behavior during HTTP connection failures
- **Concurrent operations**: Multiple players performing HTTP requests simultaneously
- **Edge cases**: Empty games, single player games, rapid join/leave cycles