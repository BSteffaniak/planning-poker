use hyperchad::{
    renderer::View,
    router::{RouteRequest, Router},
    template::{self as hyperchad_template, container, Containers},
};
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
            let content = home_page();
            let container = content.into_iter().next().unwrap_or_default();
            async move {
                Ok::<View, anyhow::Error>(View {
                    immediate: container,
                    future: None,
                })
            }
        })
        .with_route_result(
            hyperchad::router::RoutePath::LiteralPrefix("/game/".to_string()),
            |request: RouteRequest| {
                // Extract game_id from path like "/game/uuid-here"
                let game_id = request
                    .path
                    .strip_prefix("/game/")
                    .unwrap_or("")
                    .to_string();
                let content = game_page(game_id);
                let container = content.into_iter().next().unwrap_or_default();
                async move {
                    Ok::<View, anyhow::Error>(View {
                        immediate: container,
                        future: None,
                    })
                }
            },
        )
}

#[must_use]
pub fn home_page() -> Containers {
    container! {
        div width=100% height=100% padding=20 {
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
}

#[must_use]
pub fn game_page(game_id: String) -> Containers {
    let game_id_display = format!("Game ID: {game_id}");
    let start_voting_url = format!("/api/games/{game_id}/start-voting");
    let vote_url = format!("/api/games/{game_id}/vote");
    let reveal_url = format!("/api/games/{game_id}/reveal");
    let reset_url = format!("/api/games/{game_id}/reset");

    container! {
        div width=100% height=100% padding=20 {
            h1 { "Planning Poker Game" }
            div { (game_id_display) }

            div margin-top=20 {
                h2 { "Players" }
                div id="players-list" {
                    "Loading players..."
                }
            }

            div margin-top=20 {
                h2 { "Voting" }
                div id="voting-section" {
                    div margin-bottom=10 {
                        span { "Story:" }
                        input type="text" id="story-input" placeholder="Enter story to vote on" margin-left=10;
                        button hx-post=(start_voting_url) margin-left=10 padding=5 background="#007bff" color="#fff" border="none" border-radius=3 {
                            "Start Voting"
                        }
                    }

                    div margin-top=15 {
                        span { "Your Vote:" }
                        div margin-top=10 {
                            button hx-post=(vote_url.clone()) margin=5 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 { "1" }
                            button hx-post=(vote_url.clone()) margin=5 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 { "2" }
                            button hx-post=(vote_url.clone()) margin=5 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 { "3" }
                            button hx-post=(vote_url.clone()) margin=5 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 { "5" }
                            button hx-post=(vote_url.clone()) margin=5 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 { "8" }
                            button hx-post=(vote_url.clone()) margin=5 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 { "13" }
                            button hx-post=(vote_url) margin=5 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 { "?" }
                        }
                    }

                    div margin-top=15 {
                        button hx-post=(reveal_url) margin=5 padding=10 background="#dc3545" color="#fff" border="none" border-radius=5 {
                            "Reveal Votes"
                        }
                        button hx-post=(reset_url) margin=5 padding=10 background="#ffc107" color="#000" border="none" border-radius=5 {
                            "Reset Voting"
                        }
                    }
                }
            }

            div margin-top=20 {
                h2 { "Results" }
                div id="results-section" {
                    "No votes yet"
                }
            }



            div margin-top=30 {
                anchor href="/" {
                    "← Back to Home"
                }
            }
        }
    }
}
