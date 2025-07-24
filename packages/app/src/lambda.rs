#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use planning_poker_app::{build_app, init, set_renderer};
use std::sync::{Arc, LazyLock};
use tracing::info;

static RUNTIME: LazyLock<Arc<switchy::unsync::runtime::Runtime>> = LazyLock::new(|| {
    let runtime = switchy::unsync::runtime::Builder::new()
        .max_blocking_threads(64)
        .build()
        .unwrap();
    Arc::new(runtime)
});

#[allow(clippy::cognitive_complexity)]
fn main() -> Result<(), hyperchad::app::Error> {
    // Initialize tracing - respect RUST_LOG environment variable
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Planning Poker Lambda");

    // Initialize app builder (synchronous like MoosicBox)
    let app_builder = init().with_runtime_handle(RUNTIME.handle().clone());

    // Build app (database will be initialized lazily when needed)
    let app = build_app(app_builder)?;

    let renderer = Arc::new(app.renderer.clone());
    info!("Setting renderer");
    set_renderer(renderer);
    info!("Renderer set successfully");

    // Lambda mode: call handle_serve() directly like MoosicBox does
    info!("Starting Lambda handler");
    app.handle_serve()?;

    Ok(())
}
