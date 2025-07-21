use anyhow::Result;
use futures::{SinkExt, StreamExt};
use planning_poker_models::{ClientMessage, ServerMessage};
use planning_poker_session::SessionManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{error, info, warn};
use uuid::Uuid;

pub type WebSocket = WebSocketStream<tokio::net::TcpStream>;

#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    session_manager: Arc<dyn SessionManager>,
}

pub struct Connection {
    pub id: String,
    pub player_id: Option<Uuid>,
    pub game_id: Option<Uuid>,
    pub sender: mpsc::UnboundedSender<ServerMessage>,
}

impl ConnectionManager {
    pub fn new(session_manager: Arc<dyn SessionManager>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            session_manager,
        }
    }

    pub async fn add_connection(
        &self,
        connection_id: String,
        sender: mpsc::UnboundedSender<ServerMessage>,
    ) {
        let connection = Connection {
            id: connection_id.clone(),
            player_id: None,
            game_id: None,
            sender,
        };

        let mut connections = self.connections.write().await;
        connections.insert(connection_id, connection);
    }

    pub async fn remove_connection(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.remove(connection_id) {
            if let (Some(game_id), Some(player_id)) = (connection.game_id, connection.player_id) {
                // Notify other players that this player left
                self.broadcast_to_game(
                    game_id,
                    ServerMessage::PlayerLeft { player_id },
                    Some(connection_id),
                )
                .await;
            }
        }

        // Clean up session
        if let Err(e) = self.session_manager.delete_session(connection_id).await {
            error!("Failed to delete session {}: {}", connection_id, e);
        }
    }

    pub async fn handle_message(&self, connection_id: &str, message: ClientMessage) -> Result<()> {
        match message {
            ClientMessage::JoinGame {
                game_id,
                player_name,
            } => {
                self.handle_join_game(connection_id, game_id, player_name)
                    .await
            }
            ClientMessage::LeaveGame => self.handle_leave_game(connection_id).await,
            ClientMessage::CastVote { value } => self.handle_cast_vote(connection_id, value).await,
            ClientMessage::StartVoting { story } => {
                self.handle_start_voting(connection_id, story).await
            }
            ClientMessage::RevealVotes => self.handle_reveal_votes(connection_id).await,
            ClientMessage::ResetVoting => self.handle_reset_voting(connection_id).await,
        }
    }

    async fn handle_join_game(
        &self,
        connection_id: &str,
        game_id: Uuid,
        player_name: String,
    ) -> Result<()> {
        // TODO: Implement join game logic
        info!("Player {} joining game {}", player_name, game_id);

        let player = planning_poker_models::Player {
            id: Uuid::new_v4(),
            name: player_name,
            is_observer: false,
            joined_at: chrono::Utc::now(),
        };

        // Update connection
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(connection_id) {
                connection.player_id = Some(player.id);
                connection.game_id = Some(game_id);
            }
        }

        // Add player to game
        self.session_manager
            .add_player_to_game(game_id, player.clone())
            .await?;

        // Get game and players
        let game = self.session_manager.get_game(game_id).await?;
        let players = self.session_manager.get_game_players(game_id).await?;

        if let Some(game) = game {
            // Send game joined message to the new player
            self.send_to_connection(
                connection_id,
                ServerMessage::GameJoined {
                    game,
                    players: players.clone(),
                },
            )
            .await;

            // Notify other players
            self.broadcast_to_game(
                game_id,
                ServerMessage::PlayerJoined { player },
                Some(connection_id),
            )
            .await;
        } else {
            self.send_to_connection(
                connection_id,
                ServerMessage::Error {
                    message: "Game not found".to_string(),
                },
            )
            .await;
        }

        Ok(())
    }

    async fn handle_leave_game(&self, connection_id: &str) -> Result<()> {
        let (game_id, player_id) = {
            let connections = self.connections.read().await;
            if let Some(connection) = connections.get(connection_id) {
                (connection.game_id, connection.player_id)
            } else {
                return Ok(());
            }
        };

        if let (Some(game_id), Some(player_id)) = (game_id, player_id) {
            self.session_manager
                .remove_player_from_game(game_id, player_id)
                .await?;

            self.broadcast_to_game(
                game_id,
                ServerMessage::PlayerLeft { player_id },
                Some(connection_id),
            )
            .await;
        }

        Ok(())
    }

    async fn handle_cast_vote(&self, connection_id: &str, value: String) -> Result<()> {
        let (game_id, player_id) = {
            let connections = self.connections.read().await;
            if let Some(connection) = connections.get(connection_id) {
                (connection.game_id, connection.player_id)
            } else {
                return Ok(());
            }
        };

        if let (Some(game_id), Some(player_id)) = (game_id, player_id) {
            let vote = planning_poker_models::Vote {
                player_id,
                value,
                cast_at: chrono::Utc::now(),
            };

            self.session_manager.cast_vote(game_id, vote).await?;

            // Notify all players that a vote was cast (without revealing the value)
            self.broadcast_to_game(
                game_id,
                ServerMessage::VoteCast {
                    player_id,
                    has_voted: true,
                },
                None,
            )
            .await;
        }

        Ok(())
    }

    async fn handle_start_voting(&self, connection_id: &str, story: String) -> Result<()> {
        // TODO: Check if player is game owner
        let game_id = {
            let connections = self.connections.read().await;
            connections.get(connection_id).and_then(|c| c.game_id)
        };

        if let Some(game_id) = game_id {
            // Clear existing votes
            self.session_manager.clear_game_votes(game_id).await?;

            // Broadcast voting started
            self.broadcast_to_game(game_id, ServerMessage::VotingStarted { story }, None)
                .await;
        }

        Ok(())
    }

    async fn handle_reveal_votes(&self, connection_id: &str) -> Result<()> {
        // TODO: Check if player is game owner
        let game_id = {
            let connections = self.connections.read().await;
            connections.get(connection_id).and_then(|c| c.game_id)
        };

        if let Some(game_id) = game_id {
            let votes = self.session_manager.get_game_votes(game_id).await?;

            self.broadcast_to_game(game_id, ServerMessage::VotesRevealed { votes }, None)
                .await;
        }

        Ok(())
    }

    async fn handle_reset_voting(&self, connection_id: &str) -> Result<()> {
        // TODO: Check if player is game owner
        let game_id = {
            let connections = self.connections.read().await;
            connections.get(connection_id).and_then(|c| c.game_id)
        };

        if let Some(game_id) = game_id {
            self.session_manager.clear_game_votes(game_id).await?;

            self.broadcast_to_game(game_id, ServerMessage::VotingReset, None)
                .await;
        }

        Ok(())
    }

    async fn send_to_connection(&self, connection_id: &str, message: ServerMessage) {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(connection_id) {
            if let Err(e) = connection.sender.send(message) {
                warn!(
                    "Failed to send message to connection {}: {}",
                    connection_id, e
                );
            }
        }
    }

    async fn broadcast_to_game(
        &self,
        game_id: Uuid,
        message: ServerMessage,
        exclude_connection: Option<&str>,
    ) {
        let connections = self.connections.read().await;
        for (conn_id, connection) in connections.iter() {
            if connection.game_id == Some(game_id) && Some(conn_id.as_str()) != exclude_connection {
                if let Err(e) = connection.sender.send(message.clone()) {
                    warn!(
                        "Failed to broadcast message to connection {}: {}",
                        conn_id, e
                    );
                }
            }
        }
    }
}

pub async fn handle_websocket_connection(
    websocket: WebSocket,
    connection_id: String,
    connection_manager: Arc<ConnectionManager>,
) {
    let (mut ws_sender, mut ws_receiver) = websocket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Add connection to manager
    connection_manager
        .add_connection(connection_id.clone(), tx)
        .await;

    // Spawn task to handle outgoing messages
    let _connection_id_clone = connection_id.clone();
    let outgoing_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&message) {
                if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    let connection_id_clone2 = connection_id.clone();
    let connection_manager_clone = connection_manager.clone();
    let incoming_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(client_message) = serde_json::from_str::<ClientMessage>(&text) {
                        if let Err(e) = connection_manager_clone
                            .handle_message(&connection_id_clone2, client_message)
                            .await
                        {
                            error!("Failed to handle WebSocket message: {}", e);
                        }
                    } else {
                        warn!("Failed to parse WebSocket message: {}", text);
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed: {}", connection_id_clone2);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = outgoing_task => {},
        _ = incoming_task => {},
    }

    // Clean up connection
    connection_manager.remove_connection(&connection_id).await;
    info!("WebSocket connection handler finished: {}", connection_id);
}
