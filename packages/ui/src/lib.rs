use hyperchad::{
    renderer::View,
    router::{RouteRequest, Router},
    template::{self as hyperchad_template, container, Containers},
};
use planning_poker_models::{Game, GameState, Player, Vote};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub current_game: Option<planning_poker_models::Game>,
    pub players: Vec<planning_poker_models::Player>,
    pub votes: HashMap<Uuid, planning_poker_models::Vote>,
    pub my_player_id: Option<Uuid>,
    pub my_vote: Option<String>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub story_input: String,
    pub game_id_input: String,
    pub player_name_input: String,
}

pub struct PlanningPokerApp {
    state: AppState,
}

impl Default for PlanningPokerApp {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanningPokerApp {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
        }
    }

    pub fn get_state(&self) -> &AppState {
        &self.state
    }

    pub fn get_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    pub fn get_voting_options() -> Vec<String> {
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
}

pub fn create_router() -> Router {
    Router::new()
        .with_route_result("/", |_request: RouteRequest| {
            let content = app_layout();
            let container = content.into_iter().next().unwrap_or_default();
            async move {
                Ok::<View, anyhow::Error>(View {
                    immediate: container,
                    future: None,
                })
            }
        })
        .with_route_result("/home", |_request: RouteRequest| {
            let content = home_content();
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
pub fn page_layout(content: Containers) -> Containers {
    tracing::info!("page_layout called, wrapping content with main-content div");
    container! {
        div id="main-content" width=100% height=100% padding=20 overflow-y="auto" {
            (content)
        }
    }
}

#[must_use]
pub fn app_layout() -> Containers {
    page_layout(home_content())
}

#[must_use]
pub fn home_content() -> Containers {
    container! {
        h1 { "Planning Poker" }
        div { "Welcome to Planning Poker!" }

        div margin-top=20 {
            h2 { "Join a Game" }
            form hx-post="/join-game" {
                div margin-bottom=10 {
                    span { "Game ID:" }
                    input type="text" name="game-id" placeholder="Enter game ID" margin-left=10 required;
                }
                div margin-bottom=10 {
                    span { "Your Name:" }
                    input type="text" name="player-name" placeholder="Enter your name" margin-left=10 required;
                }
                button type="submit" margin-top=10 padding=10 background="#007bff" color="#fff" border="none" border-radius=5 {
                    "Join Game"
                }
            }
        }

        div margin-top=30 {
            h2 { "Create a New Game" }
            form hx-post="/api/games" {
                div margin-bottom=10 {
                    span { "Game Name:" }
                    input type="text" name="name" placeholder="Enter game name" margin-left=10 required;
                }
                div margin-bottom=10 {
                    span { "Voting System:" }
                    input type="text" name="voting_system" value="fibonacci" placeholder="fibonacci, tshirt, or powers_of_2" margin-left=10 required;
                }
                button type="submit" margin-top=10 padding=10 background="#28a745" color="#fff" border="none" border-radius=5 {
                    "Create Game"
                }
            }
        }
    }
}

// UI Component Functions

pub fn game_status_section(status: &str) -> Containers {
    container! {
        div id="game-status" margin-top=20 {
            div padding=10 background="#f0f0f0" border-radius=5 {
                span { "Status: " }
                span { (status) }
            }
        }
    }
}

pub fn players_section(players: &[Player]) -> Containers {
    container! {
        div margin-top=20 {
            h2 { "Players" }
            div id="players-list" {
                @if players.is_empty() {
                    div color="#666" { "No players yet" }
                } @else {
                    @for player in players {
                        div padding=5 border-bottom="1px solid #eee" {
                            span { (player.name) }
                            @if player.is_observer {
                                span margin-left=10 color="#666" { "(Observer)" }
                            }
                            span margin-left=10 color="#999" { (format!("joined {}", player.joined_at.format("%H:%M"))) }
                        }
                    }
                }
            }
        }
    }
}

pub fn voting_section(game_id: &str, voting_active: bool) -> Containers {
    let start_voting_url = format!("/api/games/{game_id}/start-voting");

    container! {
        div id="voting-section" margin-top=20 {
            h2 { "Voting" }

            // Story input section
            div id="story-input" margin-bottom=15 {
                span { "Story:" }
                input type="text" placeholder="Enter story to vote on" margin-left=10;
                button hx-post=(start_voting_url) margin-left=10 padding=5 background="#007bff" color="#fff" border="none" border-radius=3 {
                    "Start Voting"
                }
            }

            // Vote buttons section
            div id="vote-buttons" margin-top=15 {
                @if voting_active {
                    (vote_buttons(game_id))
                } @else {
                    div color="#666" {
                        "Voting not active. Click 'Start Voting' to begin."
                    }
                }
            }
        }
    }
}

pub fn vote_buttons(game_id: &str) -> Containers {
    let vote_values = ["1", "2", "3", "5", "8", "13", "?"];

    container! {
        span { "Your Vote:" }
        div margin-top=10 {
            @for value in vote_values {
                form hx-post=(format!("/api/games/{game_id}/vote")) {
                    input type="hidden" name="vote" value=(value);
                    button type="submit" margin=5 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 { (value) }
                }
            }
        }
    }
}

pub fn results_section(game_id: &str, votes: &[Vote], votes_revealed: bool) -> Containers {
    let reveal_url = format!("/api/games/{game_id}/reveal");
    let reset_url = format!("/api/games/{game_id}/reset");

    container! {
        div id="results-section" margin-top=20 {
            h2 { "Results" }
            div id="vote-results" {
                @if votes.is_empty() {
                    div color="#666" { "No votes cast yet" }
                } @else if votes_revealed {
                    div {
                        h3 { "Vote Results:" }
                        @for vote in votes {
                            div padding=5 border-bottom="1px solid #eee" {
                                span { (format!("{}: {}", vote.player_name, vote.value)) }
                                span margin-left=10 color="#999" { (format!("cast at {}", vote.cast_at.format("%H:%M:%S"))) }
                            }
                        }
                    }
                } @else {
                    div {
                        span { (format!("{} votes cast", votes.len())) }
                        span margin-left=10 color="#666" { "(hidden until revealed)" }
                    }
                }
            }

            // Game action buttons
            div id="game-actions" margin-top=15 {
                button hx-post=(reveal_url) margin=5 padding=10 background="#dc3545" color="#fff" border="none" border-radius=5 {
                    "Reveal Votes"
                }
                button hx-post=(reset_url) margin=5 padding=10 background="#ffc107" color="#000" border="none" border-radius=5 {
                    "Reset Voting"
                }
            }
        }
    }
}
// Partial update UI functions for SSE
pub fn players_list_content(players: &[Player]) -> Containers {
    container! {
        @if players.is_empty() {
            div color="#666" { "No players yet" }
        } @else {
            @for player in players {
                div padding=5 border-bottom="1px solid #eee" {
                    span { (player.name) }
                    @if player.is_observer {
                        span margin-left=10 color="#666" { "(Observer)" }
                    }
                    span margin-left=10 color="#999" { (format!("joined {}", player.joined_at.format("%H:%M"))) }
                }
            }
        }
    }
}

pub fn vote_results_content(votes: &[Vote], revealed: bool) -> Containers {
    container! {
        @if votes.is_empty() {
            div color="#666" { "No votes cast yet" }
        } @else if revealed {
            div {
                h3 { "Vote Results:" }
                @for vote in votes {
                    div padding=5 border-bottom="1px solid #eee" {
                        span { (format!("{}: {}", vote.player_name, vote.value)) }
                        span margin-left=10 color="#999" { (format!("cast at {}", vote.cast_at.format("%H:%M:%S"))) }
                    }
                }
            }
        } @else {
            div {
                span { (format!("{} votes cast", votes.len())) }
                span margin-left=10 color="#666" { "(hidden until revealed)" }
            }
        }
    }
}

pub fn game_status_content(status: &str) -> Containers {
    container! {
        div padding=10 background="#f0f0f0" border-radius=5 {
            span { "Status: " }
            span { (status) }
        }
    }
}

pub fn story_input_content(game_id: &str, voting_active: bool) -> Containers {
    let start_voting_url = format!("/api/games/{game_id}/start-voting");

    if voting_active {
        container! {
            span { "Story:" }
            input type="text" placeholder="Enter story to vote on" margin-left=10;
            button hx-post=(start_voting_url) margin-left=10 padding=5 background="#007bff" color="#fff" border="none" border-radius=3 disabled {
                "Voting Active"
            }
        }
    } else {
        container! {
            span { "Story:" }
            input type="text" placeholder="Enter story to vote on" margin-left=10;
            button hx-post=(start_voting_url) margin-left=10 padding=5 background="#007bff" color="#fff" border="none" border-radius=3 {
                "Start Voting"
            }
        }
    }
}

pub fn game_page_with_data(
    game_id: String,
    game: Game,
    players: Vec<Player>,
    votes: Vec<Vote>,
) -> Containers {
    tracing::info!("game_page_with_data called, wrapping with page_layout");
    let content = game_content_with_data(game_id, game, players, votes);
    page_layout(content)
}

pub fn game_content_with_data(
    game_id: String,
    game: Game,
    players: Vec<Player>,
    votes: Vec<Vote>,
) -> Containers {
    let game_id_display = format!("Game ID: {game_id}");
    let status_text = match game.state {
        GameState::Waiting => "Waiting for players",
        GameState::Voting => "Voting in progress",
        GameState::Revealed => "Votes revealed",
    };
    let voting_active = matches!(game.state, GameState::Voting);
    let votes_revealed = matches!(game.state, GameState::Revealed);

    container! {
        h1 { "Planning Poker Game" }
        div { (game_id_display) }
        div { (format!("Game: {}", game.name)) }

        (game_status_section(&status_text))
        (players_section(&players))
        (voting_section(&game_id, voting_active))
        (results_section(&game_id, &votes, votes_revealed))

        div margin-top=30 {
            anchor href="/" {
                "← Back to Home"
            }
        }
    }
}
