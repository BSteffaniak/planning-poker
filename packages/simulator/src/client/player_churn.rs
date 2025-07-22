use simvar::{
    switchy::{random::rng, unsync::time::sleep},
    Sim,
};

pub fn start(sim: &mut impl Sim) {
    let player_name = "PlayerChurnSimulator".to_string();

    sim.client(player_name.clone(), async move {
        log::info!("Starting player churn simulation for player: {player_name}");

        let churn_cycles = 5;

        for cycle in 0..churn_cycles {
            log::info!("{} starting churn cycle {}", player_name, cycle + 1);

            // Simulate joining
            sleep(std::time::Duration::from_millis(rng().gen_range(100..300))).await;
            log::info!("{} joined game (cycle {})", player_name, cycle + 1);

            // Participate briefly
            let participation_time = rng().gen_range(200..800);
            sleep(std::time::Duration::from_millis(participation_time)).await;

            // Leave (gracefully or abruptly)
            if rng().gen_bool(0.7) {
                log::info!("{} left game gracefully (cycle {})", player_name, cycle + 1);
            } else {
                log::info!(
                    "{} disconnected abruptly (cycle {})",
                    player_name,
                    cycle + 1
                );
            }

            // Wait before next cycle
            let wait_time = rng().gen_range(200..1000);
            sleep(std::time::Duration::from_millis(wait_time)).await;
        }

        log::info!("Player churn simulation completed for player: {player_name}");
        Ok::<(), Box<dyn std::error::Error + Send>>(())
    });
}
