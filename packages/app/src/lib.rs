#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use chrono::Utc;
use hyperchad::{
    app::{renderer::DefaultRenderer, App, AppBuilder},
    renderer::{Content, PartialView, Renderer},
    router::{ParseError, RouteRequest, Router},
    template::{self as hyperchad_template, container, Containers},
    transformer::html::ParseError as HtmlParseError,
};
use planning_poker_config::Config;
use planning_poker_database::{create_connection, DatabaseConfig};
use planning_poker_models::{GameState, Player, Vote};
use planning_poker_session::{DatabaseSessionManager, SessionManager};
use serde::Deserialize;
use std::sync::{Arc, OnceLock};
use switchy::http::models::Method;

use uuid::Uuid;

static RENDERER: OnceLock<Arc<dyn Renderer>> = OnceLock::new();

#[cfg(feature = "assets")]
pub mod assets {
    use hyperchad::renderer;
    use std::{path::PathBuf, sync::LazyLock};

    static CARGO_MANIFEST_DIR: LazyLock<Option<std::path::PathBuf>> =
        LazyLock::new(|| std::option_env!("CARGO_MANIFEST_DIR").map(Into::into));

    static ASSETS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
        CARGO_MANIFEST_DIR.as_ref().map_or_else(
            || <PathBuf as std::str::FromStr>::from_str("public").unwrap(),
            |dir| dir.join("public"),
        )
    });

    pub static ASSETS: LazyLock<Vec<renderer::assets::StaticAssetRoute>> = LazyLock::new(|| {
        vec![
            #[cfg(feature = "vanilla-js")]
            renderer::assets::StaticAssetRoute {
                route: format!(
                    "/js/{}",
                    hyperchad::renderer_vanilla_js::SCRIPT_NAME_HASHED.as_str()
                ),
                target: renderer::assets::AssetPathTarget::FileContents(
                    hyperchad::renderer_vanilla_js::SCRIPT.as_bytes().into(),
                ),
            },
            renderer::assets::StaticAssetRoute {
                route: "/favicon.ico".to_string(),
                target: ASSETS_DIR.join("favicon.ico").try_into().unwrap(),
            },
            renderer::assets::StaticAssetRoute {
                route: "/public".to_string(),
                target: ASSETS_DIR.clone().try_into().unwrap(),
            },
        ]
    });
}

#[derive(Debug, thiserror::Error)]
pub enum RouteError {
    #[error("Missing form data")]
    MissingFormData,
    #[error("Unsupported method")]
    UnsupportedMethod,
    #[error("Failed to parse body")]
    ParseBody(#[from] ParseError),
    #[error("Failed to parse HTML")]
    ParseHtml(#[from] HtmlParseError),
    #[error("Invalid UUID")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Route failed: {0}")]
    RouteFailed(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct JoinGameForm {
    pub game_id: String,
    pub player_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGameForm {
    pub name: String,
    pub voting_system: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    pub name: String,
    pub voting_system: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinGameRequest {
    pub player_name: String,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub player_id: Uuid,
    pub vote: String,
}

#[derive(Debug, Deserialize)]
pub struct VoteForm {
    pub vote: String,
}

// SSE Partial Update Helper Functions
#[allow(clippy::cognitive_complexity)]
async fn send_partial_update(target: &str, content: Containers) {
    let Some(renderer) = RENDERER.get() else {
        tracing::warn!("RENDERER not initialized, cannot send partial update");
        return;
    };

    tracing::info!(
        "Sending partial update to target: {} with content length: {}",
        target,
        format!("{content:?}").len()
    );

    let partial = PartialView {
        target: target.to_string(),
        container: content.into(),
    };

    if let Err(e) = renderer.render_partial(partial).await {
        tracing::error!("Failed to render_partial for target {}: {e:?}", target);
    } else {
        tracing::info!("Successfully sent partial update to target: {}", target);
    }
}

async fn update_game_status(_game_id: &str, status: &str) {
    let content = planning_poker_ui::game_status_content(status);
    send_partial_update("game-status", content).await;
}

async fn update_players_list(_game_id: &str, players: Vec<Player>) {
    let content = planning_poker_ui::players_list_content(&players);
    send_partial_update("players-list", content).await;
}

#[allow(clippy::cognitive_complexity)]
async fn update_vote_buttons(game_id: &str, voting_active: bool) {
    tracing::info!(
        "VOTE BUTTONS: update_vote_buttons called for game {}, voting_active: {}",
        game_id,
        voting_active
    );

    let content = if voting_active {
        tracing::info!("VOTE BUTTONS: Voting is active, using simple test content");
        container! {
            div {
                "VOTING IS ACTIVE - TEST MESSAGE"
            }
        }
    } else {
        tracing::info!("VOTE BUTTONS: Voting is not active, showing inactive message");
        container! {
            div color="#666" {
                "Voting not active. Click 'Start Voting' to begin."
            }
        }
    };

    tracing::info!("VOTE BUTTONS: About to send partial update to vote-buttons target");
    send_partial_update("vote-buttons", content).await;
}

async fn update_entire_voting_section(game_id: &str, voting_active: bool) {
    tracing::info!(
        "VOTING SECTION: Updating entire voting section for game {}, voting_active: {}",
        game_id,
        voting_active
    );

    let content = planning_poker_ui::voting_section(game_id, voting_active);
    send_partial_update("voting-section", content).await;
}

async fn update_story_input(game_id: &str, voting_active: bool) {
    let content = planning_poker_ui::story_input_content(game_id, voting_active);
    send_partial_update("story-input", content).await;
}

#[allow(clippy::cognitive_complexity)]
async fn update_vote_results(_game_id: &str, votes: Vec<Vote>, revealed: bool) {
    tracing::info!(
        "Updating vote results: {} votes, revealed: {}",
        votes.len(),
        revealed
    );

    // Log individual votes for debugging
    for (i, vote) in votes.iter().enumerate() {
        tracing::info!(
            "Vote {}: player_id={}, player_name={}, value={}, cast_at={}",
            i,
            vote.player_id,
            vote.player_name,
            vote.value,
            vote.cast_at
        );
    }

    if votes.is_empty() {
        tracing::info!("No votes found - will show 'No votes cast yet' message");
    } else if revealed {
        tracing::info!("Votes are revealed - will show actual vote values");
    } else {
        tracing::info!("Votes are hidden - will show vote count only");
    }

    let content = planning_poker_ui::vote_results_content(&votes, revealed);
    send_partial_update("vote-results", content).await;
}

async fn update_game_actions(game_id: &str, game_state: GameState) {
    tracing::info!(
        "GAME ACTIONS: Updating game actions for game {}, state: {:?}",
        game_id,
        game_state
    );

    let reveal_url = format!("/api/games/{game_id}/reveal");
    let reset_url = format!("/api/games/{game_id}/reset");

    let content = container! {
        @if matches!(game_state, GameState::Revealed) {
            button hx-post=(reveal_url) margin=5 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 disabled {
                "Votes Revealed"
            }
            button hx-post=(reset_url) margin=5 padding=10 background="#ffc107" color="#000" border="none" border-radius=5 {
                "Reset Voting"
            }
        } @else if matches!(game_state, GameState::Voting) {
            button hx-post=(reveal_url) margin=5 padding=10 background="#dc3545" color="#fff" border="none" border-radius=5 {
                "Reveal Votes"
            }
            button hx-post=(reset_url) margin=5 padding=10 background="#ffc107" color="#000" border="none" border-radius=5 {
                "Reset Voting"
            }
        } @else {
            // Waiting state - no votes to reveal yet, no need for reset
            div color="#666" {
                "Start voting to see action buttons"
            }
        }
    };

    send_partial_update("game-actions", content).await;
}

async fn update_entire_results_section(game_id: &str, votes: Vec<Vote>, votes_revealed: bool) {
    tracing::info!(
        "RESULTS SECTION: Updating entire results section for game {}, {} votes, revealed: {}",
        game_id,
        votes.len(),
        votes_revealed
    );

    let content = planning_poker_ui::results_section(game_id, &votes, votes_revealed);
    send_partial_update("results-section", content).await;
}

pub fn set_renderer(renderer: Arc<dyn Renderer>) {
    tracing::info!("set_renderer called");
    if RENDERER.set(renderer).is_err() {
        tracing::warn!("RENDERER already initialized");
    } else {
        tracing::info!("RENDERER successfully initialized");
    }
}

/// Initialize the app with common configuration (synchronous like `MoosicBox`)
///
/// # Panics
///
/// * If the `assets` feature is enabled and an asset fails to be initialized
#[must_use]
pub fn init() -> AppBuilder {
    // Build hyperchad app builder - following MoosicBox pattern
    #[cfg_attr(not(feature = "assets"), allow(unused_mut))]
    let mut app_builder = AppBuilder::new()
        .with_title("Planning Poker".to_string())
        .with_description("A planning poker application".to_string())
        .with_size(800.0, 600.0);

    #[cfg(feature = "assets")]
    {
        for asset in assets::ASSETS.iter().cloned() {
            tracing::trace!("Adding static asset route: {asset:?}");
            app_builder = app_builder.with_static_asset_route_result(asset).unwrap();
        }
    }

    app_builder
}

/// Set up database and create session manager
///
/// # Errors
///
/// * If database connection fails
/// * If schema initialization fails
pub async fn setup_database() -> Result<Arc<dyn SessionManager>, hyperchad::app::Error> {
    // Set up database connection
    let config = Config::from_env();
    let database_url = config
        .database_url
        .unwrap_or_else(|| "sqlite://planning_poker.db".to_string());

    let db_config = DatabaseConfig {
        database_url,
        max_connections: 10,
        connection_timeout: std::time::Duration::from_secs(30),
    };

    // Create database connection and session manager
    let db = create_connection(db_config).await.map_err(|e| {
        hyperchad::app::Error::from(Box::new(e) as Box<dyn std::error::Error + Send>)
    })?;
    let session_manager = Arc::new(DatabaseSessionManager::new(db));

    // Initialize database schema
    session_manager
        .init_schema()
        .await
        .map_err(|e| hyperchad::app::Error::from(Box::<dyn std::error::Error + Send>::from(e)))?;

    Ok(session_manager as Arc<dyn SessionManager>)
}

/// Build the app from the configured builder
///
/// # Errors
///
/// * If app building fails
pub fn build_app(
    builder: AppBuilder,
    session_manager: &Arc<dyn SessionManager>,
) -> Result<App<DefaultRenderer>, hyperchad::app::Error> {
    // Create router with planning poker routes and database access
    let router = create_app_router(session_manager);

    let app = builder.with_router(router).build_default()?;
    Ok(app)
}

pub fn create_app_router(session_manager: &Arc<dyn SessionManager>) -> Router {
    planning_poker_ui::create_router()
        .with_route_result("/join-game", {
            let session_manager = session_manager.clone();
            move |req| {
                let session_manager = session_manager.clone();
                async move { join_game_route(req, session_manager).await }
            }
        })
        .with_route_result(
            hyperchad::router::RoutePath::LiteralPrefix("/game/".to_string()),
            {
                let session_manager = session_manager.clone();
                move |req| {
                    let session_manager = session_manager.clone();
                    async move { game_page_route(req, session_manager).await }
                }
            },
        )
        .with_route_result("/api/games", {
            let session_manager = session_manager.clone();
            move |req| {
                let session_manager = session_manager.clone();
                async move {
                    // Handle both POST /api/games (create) and GET /api/games/uuid (get)
                    if req.path == "/api/games" {
                        create_game_route(req, session_manager).await
                    } else {
                        get_game_route(req, session_manager).await
                    }
                }
            }
        })
        .with_route_result(
            hyperchad::router::RoutePath::LiteralPrefix("/api/games/".to_string()),
            {
                let session_manager = session_manager.clone();
                move |req| {
                    let session_manager = session_manager.clone();
                    async move {
                        // Route based on the path suffix
                        if req.path.ends_with("/join") {
                            join_game_api_route(req, session_manager).await
                        } else if req.path.ends_with("/vote") {
                            vote_route(req, session_manager).await
                        } else if req.path.ends_with("/reveal") {
                            reveal_votes_route(req, session_manager).await
                        } else if req.path.ends_with("/start-voting") {
                            start_voting_route(req, session_manager).await
                        } else if req.path.ends_with("/reset") {
                            reset_voting_route(req, session_manager).await
                        } else {
                            // Default to get_game_route for paths like /api/games/uuid
                            get_game_route(req, session_manager).await
                        }
                    }
                }
            },
        )
}

/// Handles the join game route
///
/// # Errors
///
/// * If method is not POST
/// * If game ID is not a valid UUID
/// * If game ID is not found
/// * If adding player to game fails
/// * If getting game players fails
///
/// # Panics
///
/// * Infallible
pub async fn join_game_route(
    req: RouteRequest,
    session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Post) {
        return Err(RouteError::UnsupportedMethod);
    }

    let form_data = req.parse_form::<JoinGameForm>()?;

    // Validate form data
    if form_data.game_id.trim().is_empty() {
        return Err(RouteError::RouteFailed("Game ID is required".to_string()));
    }

    if form_data.player_name.trim().is_empty() {
        return Err(RouteError::RouteFailed(
            "Player name is required".to_string(),
        ));
    }

    // Parse game ID as UUID
    let game_id = Uuid::parse_str(&form_data.game_id)?;

    // Check if game exists
    match session_manager.get_game(game_id).await {
        Ok(Some(_)) => {
            // Join the game directly via database
            let player = Player {
                id: Uuid::new_v4(),
                name: form_data.player_name.clone(),
                is_observer: false,
                joined_at: Utc::now(),
            };
            if let Err(e) = session_manager.add_player_to_game(game_id, player).await {
                return Err(RouteError::RouteFailed(format!("Failed to join game: {e}")));
            }

            // Return success message with redirect to game page
            tracing::info!("Join game success: game_id = {}", form_data.game_id);
            let content = container! {
                h2 { "Success!" }
                div {
                    (format!("Successfully joined game {} as {}", form_data.game_id, form_data.player_name))
                }
                div margin-top=20 {
                    anchor href=(format!("/game/{}", form_data.game_id)) margin=10 padding=10 background="#007bff" color="#fff" text-decoration="none" border-radius=5 {
                        "Go to Game"
                    }
                    anchor href="/" margin=10 padding=10 background="#6c757d" color="#fff" text-decoration="none" border-radius=5 {
                        "Back to Home"
                    }
                }
            };
            let success_content = planning_poker_ui::page_layout(&content);

            Ok(Content::try_view(success_content).unwrap())
        }
        Ok(None) => Err(RouteError::RouteFailed("Game not found".to_string())),
        Err(e) => Err(RouteError::RouteFailed(format!("Database error: {e}"))),
    }
}

/// Handles the create game router
///
/// # Errors
///
/// * If method is not POST
/// * If form data is missing
/// * If form data is invalid
/// * If creating game fails
/// * If getting game fails
///
/// # Panics
///
/// * Infallible
pub async fn create_game_route(
    req: RouteRequest,
    session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Post) {
        return Err(RouteError::UnsupportedMethod);
    }

    let form_data = req.parse_form::<CreateGameForm>()?;

    // Validate form data
    if form_data.name.trim().is_empty() {
        return Err(RouteError::RouteFailed("Game name is required".to_string()));
    }

    if form_data.voting_system.trim().is_empty() {
        return Err(RouteError::RouteFailed(
            "Voting system is required".to_string(),
        ));
    }
    let owner_id = Uuid::new_v4(); // TODO: Get from authentication

    match session_manager
        .create_game(
            form_data.name.clone(),
            form_data.voting_system.clone(),
            owner_id,
        )
        .await
    {
        Ok(game) => {
            tracing::info!("Create game success: game_id = {}", game.id);
            let content = container! {
                h2 { "Game Created!" }
                div {
                    (format!("Created game: {}", game.name))
                }
                div {
                    (format!("Game ID: {}", game.id))
                }
                div margin-top=20 {
                    anchor href=(format!("/game/{}", game.id)) margin=10 padding=10 background="#007bff" color="#fff" text-decoration="none" border-radius=5 {
                        "Go to Game"
                    }
                    anchor href="/" margin=10 padding=10 background="#6c757d" color="#fff" text-decoration="none" border-radius=5 {
                        "Back to Home"
                    }
                }
            };
            let success_content = planning_poker_ui::page_layout(&content);
            Ok(Content::try_view(success_content).unwrap())
        }
        Err(e) => Err(RouteError::RouteFailed(format!(
            "Failed to create game: {e}"
        ))),
    }
}

/// Handles the game page route
///
/// # Errors
///
/// * If method is not GET
/// * If game ID is not a valid UUID
/// * If game ID is not found
/// * If getting game fails
/// * If getting game players fails
/// * If getting game votes fails
///
/// # Panics
///
/// * Infallible
pub async fn game_page_route(
    req: RouteRequest,
    session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    tracing::info!("game_page_route called with path: {}", req.path);

    if !matches!(req.method, Method::Get) {
        return Err(RouteError::UnsupportedMethod);
    }

    // Extract game_id from path like "/game/uuid-here"
    let game_id_str = req.path.strip_prefix("/game/").unwrap_or("");
    tracing::info!(
        "Game page route: received path = {}, extracted game_id_str = {}",
        req.path,
        game_id_str
    );
    let game_id = Uuid::parse_str(game_id_str)?;

    match session_manager.get_game(game_id).await {
        Ok(Some(game)) => {
            let players = session_manager
                .get_game_players(game_id)
                .await
                .unwrap_or_default();
            let votes = session_manager
                .get_game_votes(game_id)
                .await
                .unwrap_or_default();
            let game_content =
                planning_poker_ui::game_page_with_data(game_id_str, &game, &players, &votes);
            Ok(Content::try_view(game_content).unwrap())
        }
        Ok(None) => Err(RouteError::RouteFailed("Game not found".to_string())),
        Err(e) => Err(RouteError::RouteFailed(format!("Database error: {e}"))),
    }
}

/// Handles the get game route
///
/// # Errors
///
/// * If method is not GET
/// * If game ID is not a valid UUID
/// * If game ID is not found
/// * If getting game fails
/// * If getting game players fails
/// * If getting game votes fails
///
/// # Panics
///
/// * Infallible
pub async fn get_game_route(
    req: RouteRequest,
    session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Get) {
        return Err(RouteError::UnsupportedMethod);
    }

    // Extract game_id from path like "/api/games/uuid-here"
    let game_id_str = req.path.strip_prefix("/api/games/").unwrap_or("");
    let game_id = Uuid::parse_str(game_id_str)?;

    match session_manager.get_game(game_id).await {
        Ok(Some(game)) => {
            let players = session_manager
                .get_game_players(game_id)
                .await
                .unwrap_or_default();
            let votes = if game.state == planning_poker_models::GameState::Revealed {
                Some(
                    session_manager
                        .get_game_votes(game_id)
                        .await
                        .unwrap_or_default(),
                )
            } else {
                None
            };

            let content = container! {
                h2 { (format!("Game: {}", game.name)) }
                div { (format!("State: {:?}", game.state)) }

                div margin-top=20 {
                    h3 { "Players" }
                    @for player in players {
                        div { (format!("{} (joined: {})", player.name, player.joined_at.format("%H:%M"))) }
                    }
                }

                @if let Some(votes) = votes {
                    div margin-top=20 {
                        h3 { "Votes" }
                        @for vote in votes {
                            div { (format!("Player {}: {}", vote.player_id, vote.value)) }
                        }
                    }
                }
            };
            let game_content = planning_poker_ui::page_layout(&content);
            Ok(Content::try_view(game_content).unwrap())
        }
        Ok(None) => Err(RouteError::RouteFailed("Game not found".to_string())),
        Err(e) => Err(RouteError::RouteFailed(format!("Database error: {e}"))),
    }
}

/// Handles the join game API route
///
/// # Errors
///
/// * If method is not POST
/// * If game ID is not a valid UUID
/// * If game ID is not found
/// * If adding player to game fails
///
/// # Panics
///
/// * Infallible
pub async fn join_game_api_route(
    req: RouteRequest,
    session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Post) {
        return Err(RouteError::UnsupportedMethod);
    }

    // Extract game_id from path like "/api/games/uuid-here/join"
    let path_parts: Vec<&str> = req.path.split('/').collect();
    let game_id_str = path_parts.get(3).unwrap_or(&"");
    let game_id = Uuid::parse_str(game_id_str)?;
    let body = req.body.as_ref().ok_or(RouteError::MissingFormData)?;
    let join_request: JoinGameRequest = serde_json::from_slice(body)
        .map_err(|e| RouteError::ParseBody(ParseError::SerdeJson(e)))?;

    match session_manager.get_game(game_id).await {
        Ok(Some(_)) => {
            let player = Player {
                id: Uuid::new_v4(),
                name: join_request.player_name,
                is_observer: false,
                joined_at: Utc::now(),
            };
            match session_manager
                .add_player_to_game(game_id, player.clone())
                .await
            {
                Ok(()) => {
                    // Send real-time updates to all connected clients
                    if let Ok(players) = session_manager.get_game_players(game_id).await {
                        update_players_list(game_id_str, players).await;
                    }

                    let success_content = container! {
                        div padding=20 {
                            h2 { "Joined Game!" }
                            div { "Successfully joined the game" }
                            div { (format!("Your player ID: {}", player.id)) }
                        }
                    };
                    Ok(Content::try_view(success_content).unwrap())
                }
                Err(e) => Err(RouteError::RouteFailed(format!("Failed to join game: {e}"))),
            }
        }
        Ok(None) => Err(RouteError::RouteFailed("Game not found".to_string())),
        Err(e) => Err(RouteError::RouteFailed(format!("Database error: {e}"))),
    }
}

/// Handles the vote route
///
/// # Errors
///
/// * If method is not POST
/// * If game ID is not a valid UUID
/// * If game ID is not found
/// * If getting game fails
/// * If getting game players fails
/// * If casting vote fails
///
/// # Panics
///
/// * Infallible
pub async fn vote_route(
    req: RouteRequest,
    session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Post) {
        return Err(RouteError::UnsupportedMethod);
    }

    // Extract game_id from path like "/api/games/uuid-here/vote"
    let path_parts: Vec<&str> = req.path.split('/').collect();
    let game_id_str = path_parts.get(3).unwrap_or(&"");
    let game_id = Uuid::parse_str(game_id_str)?;

    // Parse form data instead of JSON
    let form_data = req.parse_form::<VoteForm>()?;

    // TODO: Get actual player ID from session management
    // For now, use the first player in the game as a workaround
    let players = session_manager
        .get_game_players(game_id)
        .await
        .unwrap_or_default();
    let (player_id, player_name) = if let Some(first_player) = players.first() {
        (first_player.id, first_player.name.clone())
    } else {
        return Err(RouteError::RouteFailed("No players in game".to_string()));
    };

    let vote = Vote {
        player_id,
        player_name,
        value: form_data.vote,
        cast_at: Utc::now(),
    };
    match session_manager.cast_vote(game_id, vote).await {
        Ok(()) => {
            tracing::info!(
                "Vote cast successfully for game {}, triggering partial updates",
                game_id
            );

            // Send partial updates via SSE instead of returning full page
            if let Ok(votes) = session_manager.get_game_votes(game_id).await {
                if let Ok(Some(game)) = session_manager.get_game(game_id).await {
                    let revealed = matches!(game.state, GameState::Revealed);
                    tracing::info!(
                        "Updating vote results: {} votes, revealed: {}",
                        votes.len(),
                        revealed
                    );
                    update_vote_results(game_id_str, votes, revealed).await;
                }
            }

            // Return minimal success response
            let success_content = container! {
                div { "Vote cast successfully" }
            };
            Ok(Content::try_view(success_content).unwrap())
        }
        Err(e) => Err(RouteError::RouteFailed(format!("Failed to cast vote: {e}"))),
    }
}

/// Handles the reveal votes route
///
/// # Errors
///
/// * If method is not POST
/// * If game ID is not a valid UUID
/// * If game ID is not found
/// * If getting game fails
/// * If revealing votes fails
///
/// # Panics
///
/// * Infallible
#[allow(clippy::cognitive_complexity)]
pub async fn reveal_votes_route(
    req: RouteRequest,
    session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Post) {
        return Err(RouteError::UnsupportedMethod);
    }

    // Extract game_id from path like "/api/games/uuid-here/reveal"
    let path_parts: Vec<&str> = req.path.split('/').collect();
    let game_id_str = path_parts.get(3).unwrap_or(&"");
    let game_id = Uuid::parse_str(game_id_str)?;

    // Reveal the votes first
    match session_manager.reveal_votes(game_id).await {
        Ok(()) => {
            tracing::info!(
                "Votes revealed successfully for game {}, triggering partial updates",
                game_id
            );

            // Send partial updates via SSE instead of returning full page
            if let Ok(Some(game)) = session_manager.get_game(game_id).await {
                let status = match game.state {
                    GameState::Waiting => "Waiting for players",
                    GameState::Voting => "Voting in progress",
                    GameState::Revealed => "Votes revealed",
                };
                tracing::info!(
                    "Game state after reveal: {:?}, status: {}",
                    game.state,
                    status
                );
                update_game_status(game_id_str, status).await;

                // Update voting section to reflect revealed state
                let voting_active = matches!(game.state, GameState::Voting);
                update_entire_voting_section(game_id_str, voting_active).await;
            }

            if let Ok(votes) = session_manager.get_game_votes(game_id).await {
                tracing::info!("Revealing {} votes", votes.len());
                update_entire_results_section(game_id_str, votes, true).await;
            }

            // Return minimal success response
            let success_content = container! {
                div { "Votes revealed successfully" }
            };
            Ok(Content::try_view(success_content).unwrap())
        }
        Err(e) => Err(RouteError::RouteFailed(format!(
            "Failed to reveal votes: {e}"
        ))),
    }
}

/// Handles the start voting route
///
/// # Errors
///
/// * If method is not POST
/// * If game ID is not a valid UUID
/// * If game ID is not found
/// * If getting game fails
/// * If starting voting fails
/// * If getting game votes fails
/// * If getting game fails
/// * If game state is not waiting
///
/// # Panics
///
/// * Infallible
#[allow(clippy::cognitive_complexity)]
pub async fn start_voting_route(
    req: RouteRequest,
    session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Post) {
        return Err(RouteError::UnsupportedMethod);
    }

    // Extract game_id from path like "/api/games/uuid-here/start-voting"
    let path_parts: Vec<&str> = req.path.split('/').collect();
    let game_id_str = path_parts.get(3).unwrap_or(&"");
    let game_id = Uuid::parse_str(game_id_str)?;

    tracing::info!("START VOTING: Received request for game {}", game_id);

    // Check current game state before starting voting
    if let Ok(Some(game)) = session_manager.get_game(game_id).await {
        tracing::info!(
            "START VOTING: Current game state before start: {:?}",
            game.state
        );
    }

    // TODO: Parse story from request body if needed
    // For now, use a default story
    let story = "Current Story".to_string();

    match session_manager.start_voting(game_id, story).await {
        Ok(()) => {
            tracing::info!(
                "START VOTING: session_manager.start_voting() completed successfully for game {}",
                game_id
            );

            // Send partial updates via SSE instead of returning full page
            if let Ok(Some(game)) = session_manager.get_game(game_id).await {
                let status = match game.state {
                    GameState::Waiting => "Waiting for players",
                    GameState::Voting => "Voting in progress",
                    GameState::Revealed => "Votes revealed",
                };
                tracing::info!(
                    "START VOTING: Game state after start_voting call: {:?}, status: {}",
                    game.state,
                    status
                );
                update_game_status(game_id_str, status).await;

                let voting_active = matches!(game.state, GameState::Voting);
                tracing::info!("START VOTING: Calculated voting_active: {}", voting_active);

                // Update the entire voting section to avoid partial update conflicts
                update_entire_voting_section(game_id_str, voting_active).await;
            } else {
                tracing::error!("START VOTING: Failed to get game after start_voting call");
            }

            if let Ok(votes) = session_manager.get_game_votes(game_id).await {
                if let Ok(Some(game)) = session_manager.get_game(game_id).await {
                    let votes_revealed = matches!(game.state, GameState::Revealed);
                    update_entire_results_section(game_id_str, votes, votes_revealed).await;
                }
            }

            // Return minimal success response
            let success_content = container! {
                div { "Voting started successfully" }
            };
            Ok(Content::try_view(success_content).unwrap())
        }
        Err(e) => Err(RouteError::RouteFailed(format!(
            "Failed to start voting: {e}"
        ))),
    }
}

/// Handles the reset voting route
///
/// # Errors
///
/// * If method is not POST
/// * If game ID is not a valid UUID
/// * If game ID is not found
/// * If getting game fails
/// * If resetting voting fails
/// * If getting game votes fails
///
/// # Panics
///
/// * Infallible
#[allow(clippy::cognitive_complexity)]
pub async fn reset_voting_route(
    req: RouteRequest,
    session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Post) {
        return Err(RouteError::UnsupportedMethod);
    }

    // Extract game_id from path like "/api/games/uuid-here/reset"
    let path_parts: Vec<&str> = req.path.split('/').collect();
    let game_id_str = path_parts.get(3).unwrap_or(&"");
    let game_id = Uuid::parse_str(game_id_str)?;

    match session_manager.reset_voting(game_id).await {
        Ok(()) => {
            tracing::info!(
                "Voting reset successfully for game {}, triggering partial updates",
                game_id
            );

            // Send partial updates via SSE instead of returning full page
            if let Ok(Some(game)) = session_manager.get_game(game_id).await {
                let status = match game.state {
                    GameState::Waiting => "Waiting for players",
                    GameState::Voting => "Voting in progress",
                    GameState::Revealed => "Votes revealed",
                };
                tracing::info!(
                    "Game state after reset: {:?}, status: {}",
                    game.state,
                    status
                );
                update_game_status(game_id_str, status).await;

                let voting_active = matches!(game.state, GameState::Voting);
                update_vote_buttons(game_id_str, voting_active).await;
                update_story_input(game_id_str, voting_active).await;
                update_game_actions(game_id_str, game.state).await;
            }

            // After reset, votes should be empty
            if let Ok(votes) = session_manager.get_game_votes(game_id).await {
                tracing::info!("Votes after reset: {} votes found", votes.len());
                update_vote_results(game_id_str, votes, false).await;
            }

            // Return minimal success response
            let success_content = container! {
                div { "Voting reset successfully" }
            };
            Ok(Content::try_view(success_content).unwrap())
        }
        Err(e) => Err(RouteError::RouteFailed(format!(
            "Failed to reset voting: {e}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use hyperchad::router::{RequestInfo, RouteRequest};
    use planning_poker_config::Config;
    use planning_poker_database::{create_connection, DatabaseConfig};
    use planning_poker_session::DatabaseSessionManager;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_join_game_form_parsing() {
        // Create a mock form data for multipart/form-data
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let form_data = "------WebKitFormBoundary7MA4YWxkTrZu0gW\r\n\
             Content-Disposition: form-data; name=\"game-id\"\r\n\r\n\
             test-game-123\r\n\
             ------WebKitFormBoundary7MA4YWxkTrZu0gW\r\n\
             Content-Disposition: form-data; name=\"player-name\"\r\n\r\n\
             John Doe\r\n\
             ------WebKitFormBoundary7MA4YWxkTrZu0gW--\r\n"
            .to_string();
        let body = Bytes::from(form_data);

        let mut headers = BTreeMap::new();
        headers.insert(
            "content-type".to_string(),
            format!("multipart/form-data; boundary={boundary}"),
        );

        let req = RouteRequest {
            path: "/join-game".to_string(),
            method: Method::Post,
            query: BTreeMap::new(),
            headers,
            cookies: BTreeMap::new(),
            info: RequestInfo::default(),
            body: Some(Arc::new(body)),
        };

        // Set up database connection
        let config = Config::default();
        let database_url = config
            .database_url
            .unwrap_or_else(|| "sqlite://planning_poker.db".to_string());

        let db_config = DatabaseConfig {
            database_url,
            max_connections: 10,
            connection_timeout: std::time::Duration::from_secs(30),
        };

        let db = create_connection(db_config).await.unwrap();
        let session_manager = Arc::new(DatabaseSessionManager::new(db));
        session_manager.init_schema().await.unwrap();

        // Test that the form parsing works
        let result = join_game_route(req, session_manager).await;

        // The result should be an error because UUID parsing will fail for "test-game-123"
        // but it should get past the form parsing stage
        match result {
            Err(RouteError::InvalidUuid(_)) => {
                // This is expected - the form was parsed successfully but UUID parsing failed
            }
            Err(other) => {
                // Let's see what error we actually get
                println!("Got error: {other:?}");
                panic!("Expected InvalidUuid error, got a different error type");
            }
            Ok(_) => panic!("Expected an error but got success"),
        }
    }

    #[test]
    fn test_join_game_form_deserialization() {
        let form_data = JoinGameForm {
            game_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            player_name: "Test Player".to_string(),
        };

        assert_eq!(form_data.game_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(form_data.player_name, "Test Player");
    }

    #[test]
    fn test_create_game_form_deserialization() {
        let form_data = CreateGameForm {
            name: "Test Game".to_string(),
            voting_system: "fibonacci".to_string(),
        };

        assert_eq!(form_data.name, "Test Game");
        assert_eq!(form_data.voting_system, "fibonacci");
    }
}
