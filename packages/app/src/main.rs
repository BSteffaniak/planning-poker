#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use planning_poker_app::{build_app, init, set_renderer};
use std::sync::Arc;
use tracing::info;

#[allow(clippy::cognitive_complexity)]
fn main() -> Result<(), hyperchad::app::Error> {
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

    // Initialize app builder (synchronous like MoosicBox)
    let app_builder = init().with_runtime_handle(runtime.handle().clone());

    // Build app (database will be initialized lazily when needed)
    let app = build_app(app_builder)?;

    let renderer = Arc::new(app.renderer.clone());
    info!("Setting renderer");
    set_renderer(renderer);
    info!("Renderer set successfully");

    info!("Running hyperchad app with built-in CLI");
    app.run()?;

    Ok(())
}
