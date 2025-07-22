# Planning Poker Simulation Configuration

## Environment Variables

The simulator supports several environment variables for configuration:

### Basic Configuration
- `SIMULATOR_RUNS`: Number of simulation runs (default: 1)
- `SIMULATOR_MAX_PARALLEL`: Maximum parallel simulations (default: CPU cores)

### Logging
- `RUST_LOG`: Set logging level (e.g., `debug`, `info`, `warn`, `error`)
- `NO_TUI`: Disable terminal UI (set to any value)

## Running Simulations

### Basic Simulation
```bash
cargo run --bin planning-poker-simulator
```

### Debug Mode with Detailed Logging
```bash
RUST_LOG=debug cargo run --bin planning-poker-simulator
```

### Multiple Runs
```bash
SIMULATOR_RUNS=10 cargo run --bin planning-poker-simulator
```

### Parallel Execution
```bash
SIMULATOR_MAX_PARALLEL=4 SIMULATOR_RUNS=20 cargo run --bin planning-poker-simulator
```

### Disable TUI (for CI/CD)
```bash
NO_TUI=1 cargo run --bin planning-poker-simulator
```

## Simulation Scenarios

The simulator includes several test scenarios:

1. **Basic Game Flow**: Single player creates game, votes, and reveals results
2. **Concurrent Voting**: Multiple players voting simultaneously
3. **Network Partitions**: Players experiencing connection failures and reconnections
4. **Player Churn**: Players rapidly joining and leaving games

## Customization

To add new simulation scenarios:

1. Create a new module in `src/client/`
2. Implement a `start(sim: &mut impl Sim)` function
3. Add the module to `src/client/mod.rs`
4. Register it in `src/main.rs` in the `on_start` method

## Architecture Notes

The simulator uses:
- **Switchy runtime**: Deterministic async operations instead of tokio
- **Simvar network**: Simulated TCP connections instead of real networking
- **Deterministic time**: Controlled timing for reproducible tests
- **In-memory database**: SQLite database within the simulation

## Output

The simulator provides:
- Real-time progress updates
- Success/failure status for each run
- Detailed error information for failed runs
- Performance metrics (simulation time vs real time)
- Deterministic and reproducible results