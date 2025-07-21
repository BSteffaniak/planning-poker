use anyhow::Result;
use hyperchad::app::AppBuilder;
use std::sync::Arc;
use tracing::{info, Level};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Planning Poker App");

    // Create runtime like MoosicBox does
    let runtime = switchy::unsync::runtime::Builder::new()
        .max_blocking_threads(64)
        .build()
        .unwrap();

    let runtime = Arc::new(runtime);

    // Create router with planning poker routes
    let router = planning_poker_ui::create_router();

    // Build hyperchad app with runtime handle - following MoosicBox pattern
    let app = AppBuilder::new()
        .with_title("Planning Poker".to_string())
        .with_description("A planning poker application".to_string())
        .with_router(router)
        .with_runtime_handle(runtime.handle().clone())
        .with_size(800.0, 600.0)
        .build_default()?;

    info!("Running hyperchad app with built-in CLI");
    app.run()?;

    Ok(())
}
