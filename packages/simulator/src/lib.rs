#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use std::{
    collections::VecDeque,
    sync::{Arc, LazyLock, Mutex},
};

use simvar::Sim;

pub mod client;
pub mod host;
pub mod http;

static ACTIONS: LazyLock<Arc<Mutex<VecDeque<Action>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(VecDeque::new())));

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Game error: {0}")]
    Game(String),
}

#[derive(Debug, Clone)]
enum Action {
    DisconnectPlayer(uuid::Uuid),
    ReconnectPlayer(uuid::Uuid),
    NetworkPartition(Vec<uuid::Uuid>),
    RestoreNetwork,
}

/// Queues a player disconnection action for the next simulation step.
///
/// # Panics
///
/// Panics if the global actions mutex is poisoned.
pub fn queue_disconnect_player(player_id: uuid::Uuid) {
    ACTIONS
        .lock()
        .unwrap()
        .push_back(Action::DisconnectPlayer(player_id));
}

/// Queues a player reconnection action for the next simulation step.
///
/// # Panics
///
/// Panics if the global actions mutex is poisoned.
pub fn queue_reconnect_player(player_id: uuid::Uuid) {
    ACTIONS
        .lock()
        .unwrap()
        .push_back(Action::ReconnectPlayer(player_id));
}

/// Queues a network partition action that will disconnect the specified players.
///
/// # Panics
///
/// Panics if the global actions mutex is poisoned.
pub fn queue_network_partition(player_ids: Vec<uuid::Uuid>) {
    ACTIONS
        .lock()
        .unwrap()
        .push_back(Action::NetworkPartition(player_ids));
}

/// Queues a network restoration action to restore connectivity for all players.
///
/// # Panics
///
/// Panics if the global actions mutex is poisoned.
pub fn queue_restore_network() {
    ACTIONS.lock().unwrap().push_back(Action::RestoreNetwork);
}

/// Processes all queued actions and applies them to the simulation.
///
/// # Panics
///
/// Panics if the global actions mutex is poisoned.
pub fn handle_actions(sim: &mut impl Sim) {
    let actions = ACTIONS.lock().unwrap().drain(..).collect::<Vec<_>>();
    for action in actions {
        match action {
            Action::DisconnectPlayer(player_id) => {
                log::debug!("Disconnecting player {player_id}");
                sim.bounce(format!("player-{player_id}"));
            }
            Action::ReconnectPlayer(player_id) => {
                log::debug!("Reconnecting player {player_id}");
                // Reconnection is handled by client simulation plans
            }
            Action::NetworkPartition(player_ids) => {
                log::debug!("Creating network partition for players: {player_ids:?}");
                for player_id in player_ids {
                    sim.bounce(format!("player-{player_id}"));
                }
            }
            Action::RestoreNetwork => {
                log::debug!("Restoring network connectivity");
                // Network restoration is handled by reconnection logic
            }
        }
    }
}
