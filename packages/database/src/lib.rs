#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use anyhow::Result;
pub use switchy::database::Database;
use switchy::database_connection::InitDbError;
use thiserror::Error;

// Re-export switchy::database types for convenience
pub use switchy::database::{
    query::{DeleteStatement, InsertStatement, SelectQuery, UpdateStatement, UpsertStatement},
    DatabaseValue, Row, TryFromDb, TryFromError,
};

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Initialization error: {0}")]
    Init(#[from] InitDbError),
    #[error("Database error: {0}")]
    Database(#[from] switchy::database::DatabaseError),
}

pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub connection_timeout: std::time::Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite://planning_poker.db".to_string(),
            max_connections: 10,
            connection_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Create a database connection using `switchy::database`
///
/// # Errors
///
/// Returns `DatabaseError::Connection` if the database URL is invalid or unsupported
/// Returns `DatabaseError::Init` if database initialization fails
///
/// # Panics
///
/// Panics if the `SQLite` URL prefix cannot be stripped (this should never happen if the URL starts with "sqlite://")
#[allow(clippy::unused_async)]
pub async fn create_connection(config: DatabaseConfig) -> Result<Box<dyn Database>, DatabaseError> {
    tracing::info!(
        "Creating database connection with URL: {}",
        config.database_url
    );

    if config.database_url.starts_with("sqlite://") {
        #[cfg(feature = "sqlite")]
        {
            let path_str = config.database_url.strip_prefix("sqlite://").unwrap();
            let path = if path_str.is_empty() || path_str == ":memory:" {
                None
            } else {
                Some(std::path::Path::new(path_str))
            };

            let db = switchy::database_connection::init(path, None).await?;
            Ok(db)
        }
        #[cfg(not(feature = "sqlite"))]
        {
            Err(DatabaseError::Connection(
                "SQLite support not enabled".to_string(),
            ))
        }
    } else if config.database_url.starts_with("postgres://")
        || config.database_url.starts_with("postgresql://")
    {
        #[cfg(feature = "postgres")]
        {
            // Parse PostgreSQL URL to extract credentials
            let url = url::Url::parse(&config.database_url)
                .map_err(|e| DatabaseError::Connection(format!("Invalid PostgreSQL URL: {e}")))?;

            let host = url
                .host_str()
                .ok_or_else(|| {
                    DatabaseError::Connection("Missing host in PostgreSQL URL".to_string())
                })?
                .to_string();

            let database_name = url.path().trim_start_matches('/').to_string();
            if database_name.is_empty() {
                return Err(DatabaseError::Connection(
                    "Missing database name in PostgreSQL URL".to_string(),
                ));
            }

            let username = url.username().to_string();
            if username.is_empty() {
                return Err(DatabaseError::Connection(
                    "Missing username in PostgreSQL URL".to_string(),
                ));
            }

            let password = url.password().map(ToString::to_string);

            let creds = switchy::database_connection::Credentials::new(
                host,
                database_name,
                username,
                password,
            );
            let db = switchy::database_connection::init(
                #[cfg(feature = "sqlite")]
                None,
                Some(creds),
            )
            .await?;
            Ok(db)
        }
        #[cfg(not(feature = "postgres"))]
        {
            Err(DatabaseError::Connection(
                "PostgreSQL support not enabled".to_string(),
            ))
        }
    } else {
        Err(DatabaseError::Connection(format!(
            "Unsupported database URL: {}",
            config.database_url
        )))
    }
}
