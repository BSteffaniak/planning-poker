use simvar::{
    switchy::{random::rng, unsync::time::sleep},
    Sim,
};

use crate::{queue_disconnect_player, queue_reconnect_player};

pub fn start(sim: &mut impl Sim) {
    let player_name = "NetworkPartitionPlayer".to_string();

    sim.client(player_name.clone(), async move {
        let player_id = uuid::Uuid::new_v4();

        log::info!("Starting network partition simulation for player: {player_name}");

        // Simulate normal operation
        for i in 0..3 {
            sleep(std::time::Duration::from_millis(rng().gen_range(100..300))).await;
            log::info!("{} normal operation round {}", player_name, i + 1);
        }

        // Simulate network partition
        log::warn!("{player_name} experiencing network partition");
        queue_disconnect_player(player_id);

        // Wait during partition
        sleep(std::time::Duration::from_millis(1000)).await;

        // Reconnect
        log::info!("{player_name} attempting to reconnect after partition");
        queue_reconnect_player(player_id);

        // Continue operation
        sleep(std::time::Duration::from_millis(500)).await;
        log::info!("{player_name} resumed normal operation");

        log::info!("Network partition simulation completed for player: {player_name}");
        Ok::<(), Box<dyn std::error::Error + Send>>(())
    });
}
