#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use std::process::ExitCode;

use planning_poker_simulator::{client, handle_actions, host};
use simvar::{run_simulation, Sim, SimBootstrap, SimConfig};

pub struct PlanningPokerSimulator;

impl SimBootstrap for PlanningPokerSimulator {
    fn build_sim(&self, mut config: SimConfig) -> SimConfig {
        // Configure simulation parameters for WebSocket connections
        let tcp_capacity = 64; // Support multiple concurrent connections
        config.tcp_capacity(tcp_capacity);
        config
    }

    fn props(&self) -> Vec<(String, String)> {
        vec![("simulation_type".to_string(), "planning_poker".to_string())]
    }

    fn on_start(&self, sim: &mut impl Sim) {
        // Start the planning poker server
        host::server::start(sim);

        // Start client simulations
        client::basic_game::start(sim);
        client::concurrent_voting::start(sim);
        client::network_partition::start(sim);
        client::player_churn::start(sim);
    }

    fn on_step(&self, sim: &mut impl Sim) {
        handle_actions(sim);
    }
}

fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let results = run_simulation(PlanningPokerSimulator)?;

    if results.iter().any(|x| !x.is_success()) {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}
