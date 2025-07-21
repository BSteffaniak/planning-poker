use anyhow::Result;
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

// Implement egui App trait for desktop rendering
#[cfg(feature = "desktop")]
impl eframe::App for PlanningPokerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Planning Poker");

            if !self.state.connected {
                ui.label("Not connected to server");
                if ui.button("Connect").clicked() {
                    if let Err(e) = self.connect_to_server() {
                        self.state.error_message = Some(format!("Connection failed: {e}"));
                    }
                }
            } else {
                ui.label("Connected to server");

                if let Some(ref game) = self.state.current_game {
                    ui.separator();
                    ui.heading(&game.name);

                    if let Some(ref story) = game.current_story {
                        ui.label(format!("Current story: {story}"));
                    }

                    match game.state {
                        GameState::Waiting => {
                            ui.label("Waiting for voting to start...");

                            if self.is_game_owner() {
                                ui.separator();
                                ui.label("Game Owner Controls:");

                                ui.text_edit_singleline(&mut self.state.story_input);
                                if ui.button("Start Voting").clicked() {
                                    if let Err(e) =
                                        self.start_voting(self.state.story_input.clone())
                                    {
                                        self.state.error_message =
                                            Some(format!("Failed to start voting: {e}"));
                                    }
                                }
                            }
                        }
                        GameState::Voting => {
                            ui.label("Voting in progress...");

                            if self.state.my_vote.is_none() {
                                ui.separator();
                                ui.label("Cast your vote:");

                                ui.horizontal_wrapped(|ui| {
                                    for option in self.get_voting_options() {
                                        if ui.button(&option).clicked() {
                                            if let Err(e) = self.cast_vote(option) {
                                                self.state.error_message =
                                                    Some(format!("Failed to cast vote: {e}"));
                                            }
                                        }
                                    }
                                });
                            } else {
                                ui.label(format!(
                                    "You voted: {}",
                                    self.state.my_vote.as_ref().unwrap()
                                ));
                            }

                            if self.is_game_owner() {
                                ui.separator();
                                if ui.button("Reveal Votes").clicked() {
                                    if let Err(e) = self.reveal_votes() {
                                        self.state.error_message =
                                            Some(format!("Failed to reveal votes: {e}"));
                                    }
                                }
                            }
                        }
                        GameState::Revealed => {
                            ui.label("Votes revealed!");

                            ui.separator();
                            for player in &self.state.players {
                                if let Some(vote) = self.state.votes.get(&player.id) {
                                    ui.label(format!("{}: {}", player.name, vote.value));
                                } else {
                                    ui.label(format!("{}: (no vote)", player.name));
                                }
                            }

                            if self.is_game_owner() {
                                ui.separator();
                                if ui.button("Reset Voting").clicked() {
                                    if let Err(e) = self.reset_voting() {
                                        self.state.error_message =
                                            Some(format!("Failed to reset voting: {e}"));
                                    }
                                }
                            }
                        }
                    }

                    ui.separator();
                    ui.label("Players:");
                    for player in &self.state.players {
                        ui.label(&player.name);
                    }
                } else {
                    ui.label("Not in a game");

                    ui.separator();
                    ui.label("Game ID:");
                    ui.text_edit_singleline(&mut self.state.game_id_input);

                    ui.label("Your name:");
                    ui.text_edit_singleline(&mut self.state.player_name_input);

                    if ui.button("Join Game").clicked() {
                        if let Ok(game_id) = Uuid::parse_str(&self.state.game_id_input) {
                            if let Err(e) =
                                self.join_game(game_id, self.state.player_name_input.clone())
                            {
                                self.state.error_message =
                                    Some(format!("Failed to join game: {e}"));
                            }
                        } else {
                            self.state.error_message = Some("Invalid game ID".to_string());
                        }
                    }
                }
            }

            if let Some(ref error) = self.state.error_message {
                ui.separator();
                ui.colored_label(egui::Color32::RED, format!("Error: {error}"));
                if ui.button("Clear Error").clicked() {
                    self.state.error_message = None;
                }
            }
        });
    }
}
