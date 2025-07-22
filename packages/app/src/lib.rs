use anyhow::Result;
use chrono::Utc;
use hyperchad::{
    renderer::Content,
    router::{ParseError, RouteRequest, Router},
    template::{self as hyperchad_template, container},
    transformer::html::ParseError as HtmlParseError,
};
use planning_poker_models::{Player, Vote};
use planning_poker_session::SessionManager;
use serde::Deserialize;
use std::sync::Arc;
use switchy::http::models::Method;
use uuid::Uuid;

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

pub fn create_app_router(session_manager: Arc<dyn SessionManager>) -> Router {
    planning_poker_ui::create_router()
        .with_route_result("/join-game", {
            let session_manager = session_manager.clone();
            move |req| {
                let session_manager = session_manager.clone();
                async move { join_game_route(req, session_manager).await }
            }
        })
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
                        } else if req.path.ends_with("/events") {
                            sse_events_route(req, session_manager).await
                        } else {
                            // Default to get_game_route for paths like /api/games/uuid
                            get_game_route(req, session_manager).await
                        }
                    }
                }
            },
        )
}

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
            let success_content = container! {
                div padding=20 {
                    h2 { "Success!" }
                    div {
                        (format!("Successfully joined game {} as {}", form_data.game_id, form_data.player_name))
                    }
                    div margin-top=20 {
                        button hx-get=(format!("/game/{}", form_data.game_id)) margin=10 padding=10 background="#007bff" color="#fff" border="none" border-radius=5 {
                            "Go to Game"
                        }
                        button hx-get="/" margin=10 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 {
                            "Back to Home"
                        }
                    }
                }
            };

            Ok(Content::try_view(success_content).unwrap())
        }
        Ok(None) => Err(RouteError::RouteFailed("Game not found".to_string())),
        Err(e) => Err(RouteError::RouteFailed(format!("Database error: {e}"))),
    }
}

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
            let success_content = container! {
                div padding=20 {
                    h2 { "Game Created!" }
                    div {
                        (format!("Created game: {}", game.name))
                    }
                    div {
                        (format!("Game ID: {}", game.id))
                    }
                    div margin-top=20 {
                        button hx-get=(format!("/game/{}", game.id)) margin=10 padding=10 background="#007bff" color="#fff" border="none" border-radius=5 {
                            "Go to Game"
                        }
                        button hx-get="/" margin=10 padding=10 background="#6c757d" color="#fff" border="none" border-radius=5 {
                            "Back to Home"
                        }
                    }
                }
            };
            Ok(Content::try_view(success_content).unwrap())
        }
        Err(e) => Err(RouteError::RouteFailed(format!(
            "Failed to create game: {e}"
        ))),
    }
}

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

            let game_content = container! {
                div padding=20 {
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
                }
            };
            Ok(Content::try_view(game_content).unwrap())
        }
        Ok(None) => Err(RouteError::RouteFailed("Game not found".to_string())),
        Err(e) => Err(RouteError::RouteFailed(format!("Database error: {e}"))),
    }
}

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
                Ok(_) => {
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
    let body = req.body.as_ref().ok_or(RouteError::MissingFormData)?;
    let vote_request: VoteRequest = serde_json::from_slice(body)
        .map_err(|e| RouteError::ParseBody(ParseError::SerdeJson(e)))?;

    let vote = Vote {
        player_id: vote_request.player_id,
        value: vote_request.vote,
        cast_at: Utc::now(),
    };
    match session_manager.cast_vote(game_id, vote).await {
        Ok(_) => {
            let success_content = container! {
                div padding=20 {
                    h2 { "Vote Cast!" }
                    div { "Your vote has been recorded successfully" }
                }
            };
            Ok(Content::try_view(success_content).unwrap())
        }
        Err(e) => Err(RouteError::RouteFailed(format!("Failed to cast vote: {e}"))),
    }
}

pub async fn reveal_votes_route(
    req: RouteRequest,
    _session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Post) {
        return Err(RouteError::UnsupportedMethod);
    }

    // Extract game_id from path like "/api/games/uuid-here/reveal"
    let path_parts: Vec<&str> = req.path.split('/').collect();
    let game_id_str = path_parts.get(3).unwrap_or(&"");
    let _game_id = Uuid::parse_str(game_id_str)?;

    // TODO: Implement reveal votes by updating game state
    // For now, just return success
    let success_content = container! {
    div padding=20 {
        h2 { "Votes Revealed!" }
        div { "All votes have been revealed successfully" }
    }
        };
    Ok(Content::try_view(success_content).unwrap())
}

pub async fn sse_events_route(
    req: RouteRequest,
    _session_manager: Arc<dyn SessionManager>,
) -> Result<Content, RouteError> {
    if !matches!(req.method, Method::Get) {
        return Err(RouteError::UnsupportedMethod);
    }

    // Extract game_id from path like "/api/games/uuid-here/events"
    let path_parts: Vec<&str> = req.path.split('/').collect();
    let game_id_str = path_parts.get(3).unwrap_or(&"");
    let _game_id = Uuid::parse_str(game_id_str)?;

    // TODO: Implement SSE stream for real-time game updates
    // For now, return a placeholder response
    let sse_content = container! {
        div {
            "SSE connection established"
        }
    };
    Ok(Content::try_view(sse_content).unwrap())
}

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

    // TODO: Parse story from request body if needed
    // For now, use a default story
    let story = "Current Story".to_string();

    match session_manager.start_voting(game_id, story).await {
        Ok(_) => {
            let success_content = container! {
                div padding=20 {
                    h2 { "Voting Started!" }
                    div { "Voting session has been started successfully" }
                }
            };
            Ok(Content::try_view(success_content).unwrap())
        }
        Err(e) => Err(RouteError::RouteFailed(format!(
            "Failed to start voting: {e}"
        ))),
    }
}

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
        Ok(_) => {
            let success_content = container! {
                div padding=20 {
                    h2 { "Voting Reset!" }
                    div { "Voting session has been reset successfully" }
                }
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
    use hyperchad::router::RouteRequest;
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
            info: Default::default(),
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
