use actix_web::{middleware::Logger, web, App, HttpServer};
use anyhow::Result;
use clap::Parser;
use planning_poker_config::Config;
use planning_poker_database::{create_connection, DatabaseConfig};
use planning_poker_session::DatabaseSessionManager;
use planning_poker_websocket::ConnectionManager;
use std::sync::Arc;
use tracing::{info, Level};

#[derive(Parser)]
#[command(name = "planning-poker-server")]
#[command(about = "Planning Poker WebSocket Server")]
struct Args {
    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    #[arg(short, long)]
    database_url: Option<String>,

    #[arg(short, long)]
    config: Option<String>,
}

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args = Args::parse();

    info!("Starting Planning Poker Server");
    info!("Host: {}", args.host);
    info!("Port: {}", args.port);

    // Load configuration
    let config = if let Some(config_path) = args.config {
        Config::from_file(&config_path)?
    } else {
        Config::default()
    };

    // Set up database
    let database_url = args
        .database_url
        .or(config.database_url.clone())
        .unwrap_or_else(|| "sqlite://planning_poker.db".to_string());

    let db_config = DatabaseConfig {
        database_url,
        max_connections: 10,
        connection_timeout: std::time::Duration::from_secs(30),
    };

    let db = create_connection(db_config).await?;
    let session_manager = Arc::new(DatabaseSessionManager::new(db));
    let connection_manager = Arc::new(ConnectionManager::new(session_manager.clone()));

    info!("Starting HTTP server on {}:{}", args.host, args.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(connection_manager.clone()))
            .app_data(web::Data::new(session_manager.clone()))
            .wrap(Logger::default())
            .configure(planning_poker_api::configure)
    })
    .bind((args.host, args.port))?
    .run()
    .await?;

    Ok(())
}
