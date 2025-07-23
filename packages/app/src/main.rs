#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use anyhow::Result;
use hyperchad::app::AppBuilder;
use planning_poker_app::set_renderer;
use planning_poker_config::Config;
use planning_poker_database::{create_connection, DatabaseConfig};
use planning_poker_session::{DatabaseSessionManager, SessionManager};
use std::sync::Arc;
use tracing::info;

#[allow(clippy::cognitive_complexity)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing - respect RUST_LOG environment variable
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Planning Poker App");

    // Create runtime like MoosicBox does
    let runtime = switchy::unsync::runtime::Builder::new()
        .max_blocking_threads(64)
        .build()
        .unwrap();

    let runtime = Arc::new(runtime);

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

    // Create database connection and session manager
    let db = runtime.block_on(async { create_connection(db_config).await })?;
    let session_manager = Arc::new(DatabaseSessionManager::new(db));

    // Initialize database schema
    {
        let session_manager_clone = session_manager.clone();
        runtime.block_on(async move { session_manager_clone.init_schema().await })?;
    }

    // Create router with planning poker routes and database access
    let router =
        planning_poker_app::create_app_router(&(session_manager as Arc<dyn SessionManager>));

    // Build hyperchad app with runtime handle - following MoosicBox pattern
    #[cfg_attr(not(feature = "assets"), allow(unused_mut))]
    let mut app_builder = AppBuilder::new()
        .with_title("Planning Poker".to_string())
        .with_description("A planning poker application".to_string())
        .with_router(router)
        .with_runtime_handle(runtime.handle().clone())
        .with_size(800.0, 600.0);

    #[cfg(feature = "assets")]
    {
        use planning_poker_app::assets::ASSETS;
        for asset in ASSETS.iter().cloned() {
            tracing::trace!("Adding static asset route: {asset:?}");
            app_builder = app_builder.with_static_asset_route_result(asset)?;
        }
    }

    let app = app_builder.build_default()?;

    let renderer = Arc::new(app.renderer.clone());
    info!("Setting renderer");
    set_renderer(renderer);
    info!("Renderer set successfully");

    info!("Running hyperchad app with built-in CLI");
    app.run()?;

    Ok(())
}
