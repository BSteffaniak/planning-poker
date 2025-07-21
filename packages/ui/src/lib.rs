use anyhow::Result;
use hyperchad::{
    renderer::View,
    router::{RouteRequest, Router},
    template::{self as hyperchad_template, container, Containers},
};
use planning_poker_models::{GameState, ServerMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub connected: bool,
    pub current_game: Option<planning_poker_models::Game>,
    pub players: Vec<planning_poker_models::Player>,
    pub votes: HashMap<Uuid, planning_poker_models::Vote>,
    pub my_player_id: Option<Uuid>,
    pub my_vote: Option<String>,
    pub server_url: String,
    pub error_message: Option<String>,
    pub story_input: String,
    pub game_id_input: String,
    pub player_name_input: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connected: false,
            current_game: None,
            players: Vec::new(),
            votes: HashMap::new(),
            my_player_id: None,
            my_vote: None,
            server_url: "ws://localhost:8080/api/v1/ws".to_string(),
            error_message: None,
            story_input: String::new(),
            game_id_input: String::new(),
            player_name_input: String::new(),
        }
    }
}

pub struct PlanningPokerApp {
    state: AppState,
}

impl PlanningPokerApp {
    pub fn new(server_url: String) -> Self {
        Self {
            state: AppState {
                server_url,
                ..Default::default()
            },
        }
    }

    pub fn connect_to_server(&mut self) -> Result<()> {
        // TODO: Implement WebSocket connection
        tracing::info!("Connecting to server: {}", self.state.server_url);
        self.state.connected = true;
        Ok(())
    }

    pub fn join_game(&mut self, game_id: Uuid, player_name: String) -> Result<()> {
        if !self.state.connected {
            return Err(anyhow::anyhow!("Not connected to server"));
        }

        // TODO: Send join game message via WebSocket
        tracing::info!("Joining game {} as {}", game_id, player_name);
        Ok(())
    }

    pub fn cast_vote(&mut self, value: String) -> Result<()> {
        if self.state.current_game.is_none() {
            return Err(anyhow::anyhow!("Not in a game"));
        }

        // TODO: Send vote message via WebSocket
        self.state.my_vote = Some(value.clone());
        tracing::info!("Casting vote: {}", value);
        Ok(())
    }

    pub fn start_voting(&mut self, story: String) -> Result<()> {
        if self.state.current_game.is_none() {
            return Err(anyhow::anyhow!("Not in a game"));
        }

        // TODO: Check if user is game owner
        // TODO: Send start voting message via WebSocket
        tracing::info!("Starting voting for story: {}", story);
        Ok(())
    }

    pub fn reveal_votes(&mut self) -> Result<()> {
        if self.state.current_game.is_none() {
            return Err(anyhow::anyhow!("Not in a game"));
        }

        // TODO: Check if user is game owner
        // TODO: Send reveal votes message via WebSocket
        tracing::info!("Revealing votes");
        Ok(())
    }

    pub fn reset_voting(&mut self) -> Result<()> {
        if self.state.current_game.is_none() {
            return Err(anyhow::anyhow!("Not in a game"));
        }

        // TODO: Check if user is game owner
        // TODO: Send reset voting message via WebSocket
        self.state.my_vote = None;
        self.state.votes.clear();
        tracing::info!("Resetting voting");
        Ok(())
    }

    pub fn handle_server_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::GameJoined { game, players } => {
                self.state.current_game = Some(game);
                self.state.players = players;
                tracing::info!("Joined game successfully");
            }
            ServerMessage::PlayerJoined { player } => {
                self.state.players.push(player);
                tracing::info!("Player joined the game");
            }
            ServerMessage::PlayerLeft { player_id } => {
                self.state.players.retain(|p| p.id != player_id);
                self.state.votes.remove(&player_id);
                tracing::info!("Player left the game");
            }
            ServerMessage::VotingStarted { story } => {
                if let Some(ref mut game) = self.state.current_game {
                    game.current_story = Some(story);
                    game.state = GameState::Voting;
                }
                self.state.my_vote = None;
                self.state.votes.clear();
                tracing::info!("Voting started");
            }
            ServerMessage::VoteCast {
                player_id,
                has_voted: _,
            } => {
                // Don't reveal the actual vote, just mark that player has voted
                tracing::info!("Player {} cast a vote", player_id);
            }
            ServerMessage::VotesRevealed { votes } => {
                if let Some(ref mut game) = self.state.current_game {
                    game.state = GameState::Revealed;
                }
                for vote in votes {
                    self.state.votes.insert(vote.player_id, vote);
                }
                tracing::info!("Votes revealed");
            }
            ServerMessage::VotingReset => {
                if let Some(ref mut game) = self.state.current_game {
                    game.state = GameState::Waiting;
                    game.current_story = None;
                }
                self.state.my_vote = None;
                self.state.votes.clear();
                tracing::info!("Voting reset");
            }
            ServerMessage::Error { message } => {
                self.state.error_message = Some(message);
                tracing::error!(
                    "Server error: {}",
                    self.state.error_message.as_ref().unwrap()
                );
            }
        }
    }

    pub fn get_voting_options(&self) -> Vec<String> {
        // Default Fibonacci sequence
        vec![
            "0".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "5".to_string(),
            "8".to_string(),
            "13".to_string(),
            "21".to_string(),
            "34".to_string(),
            "55".to_string(),
            "89".to_string(),
            "?".to_string(),
        ]
    }

    pub fn is_game_owner(&self) -> bool {
        if let (Some(game), Some(player_id)) = (&self.state.current_game, self.state.my_player_id) {
            game.owner_id == player_id
        } else {
            false
        }
    }
}

pub fn create_router() -> Router {
    Router::new().with_route_result("/", |_request: RouteRequest| {
        let content = home_page();
        // Convert Vec<Container> to single Container by taking the first one
        let container = content.into_iter().next().unwrap_or_default();
        async move {
            Ok::<View, anyhow::Error>(View {
                immediate: container,
                future: None,
            })
        }
    })
}

#[must_use]
pub fn home_page() -> Containers {
    container! {
        div width=100% height=100% padding=20 {
            h1 { "Planning Poker" }
            div { "Welcome to Planning Poker!" }

            div margin-top=20 {
                h2 { "Join a Game" }
                form {
                    div margin-bottom=10 {
                        span { "Game ID:" }
                        input type="text" name="game-id" placeholder="Enter game ID" margin-left=10;
                    }
                    div margin-bottom=10 {
                        span { "Your Name:" }
                        input type="text" name="player-name" placeholder="Enter your name" margin-left=10;
                    }
                    button type="submit" margin-top=10 padding=10 background="#007bff" color="#fff" border="none" border-radius=5 {
                        "Join Game"
                    }
                }
            }
        }
    }
}
