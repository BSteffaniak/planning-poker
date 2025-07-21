use actix_web::{web, HttpRequest, HttpResponse, Result};
use planning_poker_models::{CreateGameRequest, CreateGameResponse, GetGameResponse};
use planning_poker_session::SessionManager;
use planning_poker_websocket::ConnectionManager;
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/games", web::post().to(create_game))
            .route("/games/{game_id}", web::get().to(get_game))
            .route("/ws", web::get().to(websocket_handler)),
    );
}

async fn create_game(
    req: web::Json<CreateGameRequest>,
    session_manager: web::Data<Arc<dyn SessionManager>>,
) -> Result<HttpResponse> {
    // TODO: Get owner_id from authentication
    let owner_id = Uuid::new_v4();

    match session_manager
        .create_game(req.name.clone(), req.voting_system.clone(), owner_id)
        .await
    {
        Ok(game) => Ok(HttpResponse::Ok().json(CreateGameResponse { game })),
        Err(e) => {
            tracing::error!("Failed to create game: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create game"
            })))
        }
    }
}

async fn get_game(
    path: web::Path<Uuid>,
    session_manager: web::Data<Arc<dyn SessionManager>>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();

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

            Ok(HttpResponse::Ok().json(GetGameResponse {
                game,
                players,
                votes,
            }))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Game not found"
        }))),
        Err(e) => {
            tracing::error!("Failed to get game {}: {}", game_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get game"
            })))
        }
    }
}

async fn websocket_handler(
    req: HttpRequest,
    body: web::Payload,
    _connection_manager: web::Data<Arc<ConnectionManager>>,
) -> Result<HttpResponse> {
    let connection_id = Uuid::new_v4().to_string();

    let (response, _session, _msg_stream) = actix_ws::handle(&req, body)?;

    let _msg_stream = _msg_stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    // Convert actix-ws stream to tokio-tungstenite compatible format
    // This is a simplified approach - in a real implementation, you'd need proper conversion
    tracing::info!("WebSocket connection established: {}", connection_id);

    // TODO: Implement proper WebSocket handling with tokio-tungstenite compatibility
    // For now, just log the connection
    tokio::spawn(async move {
        tracing::info!(
            "WebSocket handler spawned for connection: {}",
            connection_id
        );
        // Handle the WebSocket connection here
    });

    Ok(response)
}
