use simvar::{utils::run_until_simulation_cancelled, Sim};

pub const HOST: &str = "planning_poker_server";
pub const PORT: u16 = 8080;

pub fn start(sim: &mut impl Sim) {
    let host = "127.0.0.1";
    let addr = format!("{host}:{PORT}");

    sim.host(HOST, move || {
        let addr = addr.clone();
        async move {
            log::debug!("starting Planning Poker server simulation");

            // Run the server simulation
            run_until_simulation_cancelled(run_server_simulation(&addr))
                .await
                .transpose()
                .map_err(|x| {
                    Box::new(std::io::Error::other(x.to_string()))
                        as Box<dyn std::error::Error + Send>
                })?;

            log::debug!("finished Planning Poker server simulation");
            Ok(())
        }
    });
}

async fn run_server_simulation(_addr: &str) -> Result<(), crate::Error> {
    use planning_poker_database::{create_connection, DatabaseConfig};
    use planning_poker_session::{DatabaseSessionManager, SessionManager};
    use simvar::switchy::unsync::time::sleep;
    use std::sync::Arc;
    use switchy::unsync::sync::RwLock;

    log::info!("Starting Planning Poker server simulation");

    // Initialize database and session manager
    let config = DatabaseConfig {
        database_url: "sqlite://:memory:".to_string(),
        ..Default::default()
    };

    let database = create_connection(config)
        .await
        .map_err(|e| crate::Error::Database(format!("Failed to create database: {e}")))?;

    let session_manager = DatabaseSessionManager::new(database);
    session_manager
        .init_schema()
        .await
        .map_err(|e| crate::Error::Database(format!("Failed to initialize schema: {e}")))?;

    let session_manager = Arc::new(RwLock::new(session_manager));

    // Main server loop
    loop {
        // Process any pending session updates
        let session_manager_guard = session_manager.read().await;
        session_manager_guard.cleanup_expired_sessions().await.ok();
        drop(session_manager_guard);

        // Simulate server processing time
        sleep(std::time::Duration::from_millis(10)).await;

        // Check if simulation should continue
        if simvar::utils::is_simulator_cancelled() {
            break;
        }
    }

    log::info!("Planning Poker server simulation completed");
    Ok(())
}
