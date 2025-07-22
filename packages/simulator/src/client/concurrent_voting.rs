use simvar::{
    switchy::{random::rng, unsync::time::sleep},
    Sim,
};

pub fn start(sim: &mut impl Sim) {
    let player_name = "ConcurrentVotingPlayer".to_string();

    sim.client(player_name.clone(), async move {
        // Simplified concurrent voting simulation
        log::info!("Starting concurrent voting simulation for player: {player_name}");

        // Simulate concurrent voting behavior
        for round in 0..3 {
            sleep(std::time::Duration::from_millis(rng().gen_range(100..300))).await;
            log::info!(
                "{} simulating concurrent vote in round {}",
                player_name,
                round + 1
            );
        }

        log::info!("Concurrent voting simulation completed for player: {player_name}");
        Ok::<(), Box<dyn std::error::Error + Send>>(())
    });
}
