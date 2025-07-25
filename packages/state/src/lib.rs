#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use std::sync::{Arc, OnceLock};

use anyhow::Result;
use planning_poker_config::Config;
use planning_poker_database::{create_connection, DatabaseConfig};
pub use planning_poker_session::{DatabaseSessionManager, SessionManager};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Database error: {0}")]
    Database(#[from] planning_poker_database::DatabaseError),
    #[error("Session error: {0}")]
    Session(#[from] anyhow::Error),
}

/// Planning Poker application state with lazy database initialization
pub struct PlanningPokerState {
    session_manager: OnceLock<Arc<dyn SessionManager>>,
}

impl PlanningPokerState {
    /// Create a new state instance with no database connection
    #[must_use]
    pub const fn new() -> Self {
        Self {
            session_manager: OnceLock::new(),
        }
    }

    /// Get the session manager, initializing the database connection on first access
    ///
    /// # Errors
    ///
    /// Returns `StateError` if database connection or initialization fails
    ///
    /// # Panics
    ///
    /// Panics if the `OnceLock` is in an inconsistent state after successful initialization.
    /// This should never happen in practice as the `OnceLock` guarantees thread-safe initialization.
    pub async fn get_session_manager(&self) -> Result<&Arc<dyn SessionManager>, StateError> {
        // Return existing session manager if already initialized
        if let Some(manager) = self.session_manager.get() {
            return Ok(manager);
        }

        // Lazy initialization - only happens on first access
        tracing::info!("Initializing database connection (lazy initialization)");

        let manager = Arc::new(self.setup_database().await?);

        // Store the session manager (this can only happen once due to OnceLock)
        match self.session_manager.set(manager) {
            Ok(()) => {
                tracing::info!("Database connection initialized successfully");
                Ok(self.session_manager.get().unwrap())
            }
            Err(_) => {
                // Another thread beat us to initialization, return the existing one
                Ok(self.session_manager.get().unwrap())
            }
        }
    }

    /// Set up database connection and initialize schema
    async fn setup_database(&self) -> Result<DatabaseSessionManager, StateError> {
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
        let db = create_connection(db_config).await?;
        let session_manager = DatabaseSessionManager::new(db);

        // Initialize database schema
        session_manager.init_schema().await?;

        Ok(session_manager)
    }
}

impl Default for PlanningPokerState {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export session manager types for convenience
// (Already re-exported above)
