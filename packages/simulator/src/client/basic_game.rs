use simvar::{
    switchy::{
        random::rng,
        tcp::TcpStream,
        unsync::{io::AsyncWriteExt, time::sleep},
    },
    Sim,
};
use uuid::Uuid;

use crate::{
    host::server::PORT,
    http::{parse_http_response, read_http_response},
};

pub fn start(sim: &mut impl Sim) {
    let server_addr = format!("127.0.0.1:{PORT}");
    let player_name = "BasicGamePlayer".to_string();

    sim.client(player_name.clone(), async move {
        run_basic_game_simulation(&server_addr, &player_name).await
    });
}

async fn run_basic_game_simulation(
    server_addr: &str,
    player_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    let player_id = Uuid::new_v4();
    let mut game_id: Option<Uuid> = None;
    log::info!("Starting basic game simulation for player: {player_name}");

    // Create a new game via HTTP POST
    let create_game_request = serde_json::json!({
        "name": format!("{}'s Game", player_name),
        "voting_system": "fibonacci"
    });

    let (status, body) = make_http_request(
        server_addr,
        "POST",
        "/api/v1/games",
        Some(&create_game_request.to_string()),
        Some("application/json"),
    )
    .await?;

    if status != 200 {
        return Err(Box::new(std::io::Error::other(format!(
            "Failed to create game: HTTP {status}"
        ))));
    }

    let game_response: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

    if let Some(game_obj) = game_response.get("game") {
        if let Some(game_id_str) = game_obj.get("id").and_then(|v| v.as_str()) {
            game_id = Some(Uuid::parse_str(game_id_str).unwrap());
            log::info!("Game created with ID: {game_id_str}");
        }
    }

    log::debug!("HTTP connection established for {player_name}");

    let Some(game_id) = game_id else {
        return Err(Box::new(std::io::Error::other(
            "Failed to get game ID from response",
        )));
    };

    // Simulate other players joining
    sleep(std::time::Duration::from_millis(100)).await;

    // Join the game as a player (simulate form submission)
    let join_request = format!("game-id={game_id}&player-name={player_name}");
    let (status, _body) = make_http_request(
        server_addr,
        "POST",
        &format!("/games/{game_id}/join"),
        Some(&join_request),
        Some("application/x-www-form-urlencoded"),
    )
    .await?;

    if status != 200 && status != 302 {
        return Err(Box::new(std::io::Error::other(format!(
            "Failed to join game: HTTP {status}"
        ))));
    }

    log::info!("{player_name} joined game {game_id}");

    // Wait a bit then cast a vote
    sleep(std::time::Duration::from_millis(rng().gen_range(100..500))).await;

    let vote_values = ["1", "2", "3", "5", "8", "13", "21"];
    let vote_value = vote_values[rng().gen_range(0..vote_values.len())];

    // Cast vote via HTTP POST
    let vote_request = serde_json::json!({
        "player_id": player_id,
        "vote": vote_value
    });

    let (status, _body) = make_http_request(
        server_addr,
        "POST",
        &format!("/api/v1/games/{game_id}/vote"),
        Some(&vote_request.to_string()),
        Some("application/json"),
    )
    .await?;

    if status == 200 || status == 201 {
        log::info!("{player_name} cast vote: {vote_value}");
    } else {
        log::warn!("{player_name} failed to cast vote: HTTP {status}");
    }

    // Wait for other votes and reveal
    sleep(std::time::Duration::from_millis(1000)).await;

    // Get final game state
    let (status, body) = make_http_request(
        server_addr,
        "GET",
        &format!("/api/v1/games/{game_id}"),
        None,
        None,
    )
    .await?;

    if status == 200 {
        log::info!("Final game state: {body}");
    }

    log::info!("Basic game simulation completed for player: {player_name}");
    Ok(())
}

async fn make_http_request(
    server_addr: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
    content_type: Option<&str>,
) -> Result<(u16, String), Box<dyn std::error::Error + Send>> {
    let mut connection = TcpStream::connect(server_addr)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

    let content_length = body.map_or(0, str::len);
    let content_type_header =
        content_type.map_or(String::new(), |ct| format!("Content-Type: {ct}\r\n"));

    let request = format!(
        "{method} {path} HTTP/1.1\r\n\
         Host: {server_addr}\r\n\
         {content_type_header}Content-Length: {content_length}\r\n\
         Connection: close\r\n\
         \r\n{body}",
        body = body.unwrap_or("")
    );

    connection
        .write_all(request.as_bytes())
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

    let mut response = String::new();
    if let Some(response_data) = read_http_response(&mut response, Box::pin(&mut connection))
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?
    {
        let (status, body) = parse_http_response(&response_data)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        Ok((status, body))
    } else {
        Err(Box::new(std::io::Error::other("No HTTP response received")))
    }
}
