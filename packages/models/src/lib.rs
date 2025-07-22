use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub voting_system: String,
    pub state: GameState,
    pub current_story: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameState {
    Waiting,
    Voting,
    Revealed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub is_observer: bool,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub player_id: Uuid,
    pub player_name: String,
    pub value: String,
    pub cast_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub game_id: Uuid,
    pub player_id: Uuid,
    pub connection_id: String,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    JoinGame { game_id: Uuid, player_name: String },
    LeaveGame,
    CastVote { value: String },
    StartVoting { story: String },
    RevealVotes,
    ResetVoting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    GameJoined { game: Game, players: Vec<Player> },
    PlayerJoined { player: Player },
    PlayerLeft { player_id: Uuid },
    VotingStarted { story: String },
    VoteCast { player_id: Uuid, has_voted: bool },
    VotesRevealed { votes: Vec<Vote> },
    VotingReset,
    Error { message: String },
}

// API request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub name: String,
    pub voting_system: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameResponse {
    pub game: Game,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetGameResponse {
    pub game: Game,
    pub players: Vec<Player>,
    pub votes: Option<Vec<Vote>>,
}
