# Planning Poker Implementation Patterns

## Purpose

This document captures reusable patterns discovered during Planning Poker development. These patterns ensure consistency, reliability, and maintainability across the application.

## Pattern Categories

### 1. WebSocket Communication Patterns

#### Message Envelope Pattern

**Problem:** Inconsistent message formats lead to parsing errors and difficult debugging.

**Solution:** Use a standardized message envelope for all WebSocket communication.

```rust
// packages/state/src/message.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    pub message_type: String,
    pub correlation_id: String,
    pub timestamp: i64,
    pub payload: T,
    pub version: u8,
}

impl<T> MessageEnvelope<T> {
    pub fn new(message_type: &str, payload: T) -> Self {
        Self {
            message_type: message_type.to_string(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            payload,
            version: 1,
        }
    }
}
```

**Usage:**

- All WebSocket messages MUST use this envelope
- Include correlation_id for request/response tracking
- Version field enables protocol evolution
- Timestamp helps with message ordering and debugging

**Examples:**

- `packages/session/src/websocket.rs:45-67` - Player join messages
- `packages/poker/src/events.rs:123-145` - Vote submission events

---

#### Connection State Management Pattern

**Problem:** WebSocket connections can drop unexpectedly, leaving clients in inconsistent states.

**Solution:** Implement a robust connection state machine with automatic recovery.

```rust
// packages/session/src/connection.rs
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Reconnecting { attempt: u32, backoff_ms: u64 },
    Disconnected,
    Failed,
}

pub struct ConnectionManager {
    state: ConnectionState,
    last_ping: Option<Instant>,
    reconnect_config: ReconnectConfig,
}

impl ConnectionManager {
    pub fn handle_disconnect(&mut self) -> ConnectionAction {
        match self.state {
            ConnectionState::Connected => {
                self.state = ConnectionState::Reconnecting {
                    attempt: 1,
                    backoff_ms: self.reconnect_config.initial_backoff
                };
                ConnectionAction::Reconnect
            }
            ConnectionState::Reconnecting { attempt, .. } if attempt < self.reconnect_config.max_attempts => {
                let backoff = self.calculate_backoff(attempt);
                self.state = ConnectionState::Reconnecting {
                    attempt: attempt + 1,
                    backoff_ms: backoff
                };
                ConnectionAction::RetryAfter(backoff)
            }
            _ => {
                self.state = ConnectionState::Failed;
                ConnectionAction::GiveUp
            }
        }
    }
}
```

**Usage:**

- Track connection state explicitly
- Implement exponential backoff for reconnection
- Provide user feedback for connection issues
- Synchronize state after successful reconnection

---

#### Broadcast with Acknowledgment Pattern

**Problem:** Critical messages (like vote reveals) need guaranteed delivery to all participants.

**Solution:** Implement acknowledgment-based broadcasting with retry logic.

```rust
// packages/session/src/broadcast.rs
pub struct ReliableBroadcast {
    pending_acks: HashMap<String, PendingMessage>,
    retry_config: RetryConfig,
}

#[derive(Debug)]
struct PendingMessage {
    message: MessageEnvelope<serde_json::Value>,
    recipients: HashSet<PlayerId>,
    pending_acks: HashSet<PlayerId>,
    created_at: Instant,
    retry_count: u32,
}

impl ReliableBroadcast {
    pub async fn broadcast_with_ack<T>(&mut self, message: T, recipients: Vec<PlayerId>) -> Result<(), BroadcastError>
    where
        T: Serialize + Clone,
    {
        let envelope = MessageEnvelope::new("broadcast", message);
        let correlation_id = envelope.correlation_id.clone();

        // Send to all recipients
        for recipient in &recipients {
            self.send_to_player(recipient, &envelope).await?;
        }

        // Track pending acknowledgments
        self.pending_acks.insert(correlation_id, PendingMessage {
            message: envelope,
            recipients: recipients.iter().cloned().collect(),
            pending_acks: recipients.iter().cloned().collect(),
            created_at: Instant::now(),
            retry_count: 0,
        });

        Ok(())
    }

    pub fn handle_acknowledgment(&mut self, correlation_id: &str, player_id: PlayerId) {
        if let Some(pending) = self.pending_acks.get_mut(correlation_id) {
            pending.pending_acks.remove(&player_id);

            if pending.pending_acks.is_empty() {
                self.pending_acks.remove(correlation_id);
            }
        }
    }
}
```

**Usage:**

- Use for critical game events (vote reveals, game state changes)
- Implement timeout and retry logic
- Provide fallback for unacknowledged messages
- Log delivery failures for monitoring

---

### 2. State Management Patterns

#### Server-Authoritative State Pattern

**Problem:** Clients can get out of sync, leading to inconsistent game states.

**Solution:** Server maintains authoritative state; clients use optimistic updates with rollback.

```rust
// packages/poker/src/state.rs
pub struct GameState {
    pub game_id: GameId,
    pub phase: GamePhase,
    pub players: HashMap<PlayerId, Player>,
    pub current_story: Option<Story>,
    pub votes: HashMap<PlayerId, Vote>,
    pub version: u64, // For optimistic concurrency control
}

impl GameState {
    pub fn apply_command(&mut self, command: GameCommand) -> Result<Vec<GameEvent>, GameError> {
        // Validate command against current state
        self.validate_command(&command)?;

        // Apply command and generate events
        let events = match command {
            GameCommand::SubmitVote { player_id, vote } => {
                self.votes.insert(player_id, vote);
                vec![GameEvent::VoteSubmitted { player_id, vote }]
            }
            GameCommand::RevealVotes => {
                if self.all_players_voted() {
                    self.phase = GamePhase::VotesRevealed;
                    vec![GameEvent::VotesRevealed {
                        votes: self.votes.clone(),
                        consensus: self.calculate_consensus(),
                    }]
                } else {
                    return Err(GameError::NotAllPlayersVoted);
                }
            }
            // ... other commands
        };

        self.version += 1;
        Ok(events)
    }
}
```

**Usage:**

- Server processes all state changes through commands
- Generate events for state changes to broadcast to clients
- Use version numbers for optimistic concurrency control
- Clients can optimistically update UI but must handle rollbacks

---

#### Event Sourcing Pattern

**Problem:** Need to track game history and enable replay/debugging capabilities.

**Solution:** Store events instead of just current state, rebuild state from events.

```rust
// packages/poker/src/events.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    GameCreated { game_id: GameId, facilitator: PlayerId },
    PlayerJoined { player_id: PlayerId, name: String },
    PlayerLeft { player_id: PlayerId },
    StoryStarted { story: Story },
    VoteSubmitted { player_id: PlayerId, vote: Vote },
    VotesRevealed { votes: HashMap<PlayerId, Vote>, consensus: Option<Consensus> },
    GameEnded { final_estimates: Vec<StoryEstimate> },
}

pub struct EventStore {
    events: Vec<(GameId, GameEvent, Instant)>,
}

impl EventStore {
    pub fn append_event(&mut self, game_id: GameId, event: GameEvent) {
        self.events.push((game_id, event, Instant::now()));
    }

    pub fn rebuild_state(&self, game_id: GameId) -> GameState {
        let mut state = GameState::new(game_id);

        for (id, event, _) in &self.events {
            if *id == game_id {
                state.apply_event(event.clone());
            }
        }

        state
    }
}
```

**Usage:**

- Store all game events for audit trail
- Enable game replay for debugging
- Support state reconstruction after server restart
- Facilitate analytics and game statistics

---

### 3. Error Handling Patterns

#### Graceful Degradation Pattern

**Problem:** Partial system failures shouldn't break the entire user experience.

**Solution:** Implement fallback behaviors and cached state display.

```rust
// packages/ui/src/error_handling.rs
pub enum ServiceState {
    Available,
    Degraded { reason: String, fallback: FallbackStrategy },
    Unavailable { reason: String, retry_after: Option<Duration> },
}

pub enum FallbackStrategy {
    ShowCachedData,
    ReadOnlyMode,
    OfflineMode,
}

pub struct ServiceManager {
    websocket_state: ServiceState,
    database_state: ServiceState,
    cached_game_state: Option<GameState>,
}

impl ServiceManager {
    pub fn handle_websocket_failure(&mut self, error: WebSocketError) {
        match error {
            WebSocketError::ConnectionLost => {
                self.websocket_state = ServiceState::Degraded {
                    reason: "Connection lost, attempting to reconnect".to_string(),
                    fallback: FallbackStrategy::ShowCachedData,
                };
            }
            WebSocketError::ServerError => {
                self.websocket_state = ServiceState::Unavailable {
                    reason: "Server error, please try again later".to_string(),
                    retry_after: Some(Duration::from_secs(30)),
                };
            }
        }
    }

    pub fn get_game_state(&self) -> Result<GameState, ServiceError> {
        match self.websocket_state {
            ServiceState::Available => self.fetch_live_state(),
            ServiceState::Degraded { ref fallback, .. } => {
                match fallback {
                    FallbackStrategy::ShowCachedData => {
                        self.cached_game_state.clone()
                            .ok_or(ServiceError::NoFallbackData)
                    }
                    _ => Err(ServiceError::ServiceDegraded),
                }
            }
            ServiceState::Unavailable { .. } => Err(ServiceError::ServiceUnavailable),
        }
    }
}
```

**Usage:**

- Detect service failures and switch to degraded mode
- Show cached data when live updates aren't available
- Provide clear user feedback about system state
- Implement automatic recovery when services restore

---

#### Result Chain Pattern

**Problem:** Complex operations involve multiple fallible steps that need proper error propagation.

**Solution:** Use Result chaining with context-aware error types.

```rust
// packages/poker/src/game_operations.rs
#[derive(Debug, thiserror::Error)]
pub enum GameOperationError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] WebSocketError),
    #[error("Invalid game state: {message}")]
    InvalidState { message: String },
    #[error("Player not found: {player_id}")]
    PlayerNotFound { player_id: PlayerId },
}

pub async fn submit_vote(
    game_id: GameId,
    player_id: PlayerId,
    vote: Vote,
    db: &Database,
    websocket: &WebSocketManager,
) -> Result<(), GameOperationError> {
    // Validate player exists and is in game
    let player = db.get_player(player_id).await?
        .ok_or(GameOperationError::PlayerNotFound { player_id })?;

    // Validate game state allows voting
    let game_state = db.get_game_state(game_id).await?;
    if game_state.phase != GamePhase::Voting {
        return Err(GameOperationError::InvalidState {
            message: format!("Game is in {:?} phase, voting not allowed", game_state.phase),
        });
    }

    // Store vote in database
    db.store_vote(game_id, player_id, vote.clone()).await?;

    // Broadcast vote event to other players
    let event = GameEvent::VoteSubmitted { player_id, vote };
    websocket.broadcast_to_game(game_id, event).await?;

    Ok(())
}
```

**Usage:**

- Use `?` operator for clean error propagation
- Provide context-specific error messages
- Implement `From` traits for error conversion
- Log errors at appropriate levels

---

### 4. Testing Patterns

#### Simulation Testing Pattern

**Problem:** Real-time multiplayer features are difficult to test with traditional unit tests.

**Solution:** Create simulation framework that can orchestrate multiple virtual clients.

```rust
// packages/simulator/src/game_simulation.rs
pub struct GameSimulation {
    server: TestServer,
    clients: Vec<TestClient>,
    scenario: SimulationScenario,
}

pub struct SimulationScenario {
    pub name: String,
    pub players: Vec<TestPlayer>,
    pub actions: Vec<SimulationAction>,
    pub expected_outcomes: Vec<ExpectedOutcome>,
}

#[derive(Debug)]
pub enum SimulationAction {
    PlayerJoin { player_id: PlayerId, delay_ms: u64 },
    SubmitVote { player_id: PlayerId, vote: Vote, delay_ms: u64 },
    DisconnectPlayer { player_id: PlayerId },
    ReconnectPlayer { player_id: PlayerId, delay_ms: u64 },
    RevealVotes { delay_ms: u64 },
}

impl GameSimulation {
    pub async fn run_scenario(&mut self, scenario: SimulationScenario) -> SimulationResult {
        let mut results = SimulationResult::new(&scenario.name);

        // Execute actions in sequence
        for action in scenario.actions {
            match action {
                SimulationAction::PlayerJoin { player_id, delay_ms } => {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    let client = self.create_client(player_id).await?;
                    client.join_game(self.game_id).await?;
                    results.record_action("player_join", player_id);
                }
                SimulationAction::SubmitVote { player_id, vote, delay_ms } => {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    let client = self.get_client(player_id)?;
                    client.submit_vote(vote).await?;
                    results.record_action("vote_submit", player_id);
                }
                // ... handle other actions
            }
        }

        // Validate expected outcomes
        for expected in scenario.expected_outcomes {
            let actual = self.get_actual_outcome(&expected).await?;
            results.validate_outcome(expected, actual);
        }

        results
    }
}
```

**Usage:**

- Test complex multiplayer scenarios
- Validate real-time synchronization
- Test error recovery and edge cases
- Performance testing with many concurrent clients

---

### 5. Configuration Patterns

#### Environment-Aware Configuration Pattern

**Problem:** Different environments (dev, staging, prod) need different configurations.

**Solution:** Layered configuration with environment-specific overrides.

```rust
// packages/config/src/lib.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub websocket: WebSocketConfig,
    pub logging: LoggingConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Config::builder()
            // Start with default values
            .add_source(File::with_name("config/default"))
            // Add environment-specific config
            .add_source(File::with_name(&format!("config/{}",
                std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
            )).required(false))
            // Add local overrides (not in version control)
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix
            .add_source(Environment::with_prefix("PLANNING_POKER"))
            .build()?;

        config.try_deserialize()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    #[serde(default = "default_request_timeout")]
    pub request_timeout_ms: u64,
}

fn default_request_timeout() -> u64 { 30000 }
```

**Usage:**

- Layer configurations: default → environment → local → env vars
- Use serde defaults for optional values
- Validate configuration on startup
- Support both file-based and environment variable configuration

---

## Pattern Usage Guidelines

### When to Create New Patterns

Create a new pattern when you find yourself:

1. Solving the same problem in multiple places
2. Writing similar code structures repeatedly
3. Handling the same type of error scenarios
4. Implementing similar validation logic

### Pattern Documentation Requirements

Each pattern must include:

- **Problem statement** - What issue does this solve?
- **Solution description** - How does the pattern work?
- **Code example** - Concrete implementation
- **Usage guidelines** - When and how to apply
- **Examples** - Where it's used in the codebase

### Pattern Evolution

Patterns should evolve as the codebase grows:

- Update patterns when better solutions are discovered
- Deprecate patterns that are no longer needed
- Migrate existing code to use improved patterns
- Document pattern changes with migration guides

## Anti-Patterns to Avoid

### WebSocket Anti-Patterns

- ❌ Sending raw JSON without message envelopes
- ❌ Not handling connection drops gracefully
- ❌ Blocking operations in WebSocket handlers
- ❌ Not validating incoming messages

### State Management Anti-Patterns

- ❌ Allowing clients to directly modify server state
- ❌ Not versioning state changes
- ❌ Storing sensitive data in client state
- ❌ Not handling concurrent state modifications

### Error Handling Anti-Patterns

- ❌ Using `unwrap()` or `expect()` in production code
- ❌ Swallowing errors without logging
- ❌ Not providing user-friendly error messages
- ❌ Not implementing proper error recovery

By following these patterns, the Planning Poker application maintains consistency, reliability, and maintainability across all components.
