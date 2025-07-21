use anyhow::Result;
use clap::Parser;
use planning_poker_ui::PlanningPokerApp;
use tracing::{info, Level};

#[derive(Parser)]
#[command(name = "planning-poker-app")]
#[command(about = "Planning Poker Cross-Platform Application")]
struct Args {
    #[arg(short, long, default_value = "desktop")]
    renderer: String,

    #[arg(short, long, default_value = "ws://localhost:8080/api/v1/ws")]
    server_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args = Args::parse();

    info!("Starting Planning Poker App");
    info!("Renderer: {}", args.renderer);
    info!("Server URL: {}", args.server_url);

    let app = PlanningPokerApp::new(args.server_url);

    match args.renderer.as_str() {
        #[cfg(feature = "desktop")]
        "desktop" => {
            info!("Starting desktop app with egui renderer");
            run_desktop_app(app).await?;
        }
        #[cfg(feature = "web")]
        "web" => {
            info!("Starting web app with HTML renderer");
            run_web_app(app).await?;
        }
        _ => {
            eprintln!("Unknown renderer: {}", args.renderer);
            eprintln!("Available renderers: desktop, web");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(feature = "desktop")]
async fn run_desktop_app(app: PlanningPokerApp) -> Result<()> {
    use eframe::egui;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Planning Poker"),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native("Planning Poker", options, Box::new(|_cc| Ok(Box::new(app))))
    {
        eprintln!("Failed to run desktop app: {e}");
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(feature = "web")]
async fn run_web_app(_app: PlanningPokerApp) -> Result<()> {
    // TODO: Implement web app using hyperchad HTML renderer
    info!("Web app not yet implemented");
    Ok(())
}
