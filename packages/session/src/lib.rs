use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use planning_poker_database::{Database, DatabaseValue, Row};
use planning_poker_models::{Game, GameState, Player, Session, Vote};
use switchy::database::query::FilterableQuery;
use uuid::Uuid;

#[async_trait]
pub trait SessionManager: Send + Sync {
    async fn create_game(
        &self,
        name: String,
        voting_system: String,
        owner_id: Uuid,
    ) -> Result<Game>;
    async fn get_game(&self, game_id: Uuid) -> Result<Option<Game>>;
    async fn update_game(&self, game: &Game) -> Result<()>;
    async fn delete_game(&self, game_id: Uuid) -> Result<()>;

    async fn add_player_to_game(&self, game_id: Uuid, player: Player) -> Result<()>;
    async fn remove_player_from_game(&self, game_id: Uuid, player_id: Uuid) -> Result<()>;
    async fn get_game_players(&self, game_id: Uuid) -> Result<Vec<Player>>;

    async fn cast_vote(&self, game_id: Uuid, vote: Vote) -> Result<()>;
    async fn get_game_votes(&self, game_id: Uuid) -> Result<Vec<Vote>>;
    async fn clear_game_votes(&self, game_id: Uuid) -> Result<()>;

    async fn start_voting(&self, game_id: Uuid, story: String) -> Result<()>;
    async fn reset_voting(&self, game_id: Uuid) -> Result<()>;

    async fn create_session(&self, session: Session) -> Result<()>;
    async fn get_session(&self, connection_id: &str) -> Result<Option<Session>>;
    async fn update_session_last_seen(&self, connection_id: &str) -> Result<()>;
    async fn delete_session(&self, connection_id: &str) -> Result<()>;
    async fn cleanup_expired_sessions(&self) -> Result<()>;
}

pub struct DatabaseSessionManager {
    #[allow(dead_code)]
    db: std::sync::Arc<Box<dyn Database>>,
}

impl DatabaseSessionManager {
    pub fn new(db: Box<dyn Database>) -> Self {
        Self {
            db: std::sync::Arc::new(db),
        }
    }

    pub async fn init_schema(&self) -> Result<()> {
        tracing::info!("Running database migrations...");

        planning_poker_schema::migrate(&**self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

        tracing::info!("Database migrations completed successfully");
        Ok(())
    }
}

// Database model conversion functions
fn get_string_from_row(row: &Row, column: &str) -> Result<String> {
    match row.get(column) {
        Some(DatabaseValue::String(s)) => Ok(s),
        Some(value) => Err(anyhow::anyhow!(
            "Expected string for column '{}', got {:?}",
            column,
            value
        )),
        None => Err(anyhow::anyhow!("Missing column '{}'", column)),
    }
}

fn get_optional_string_from_row(row: &Row, column: &str) -> Result<Option<String>> {
    match row.get(column) {
        Some(DatabaseValue::String(s)) => Ok(Some(s)),
        Some(DatabaseValue::Null) => Ok(None),
        Some(value) => Err(anyhow::anyhow!(
            "Expected string or null for column '{}', got {:?}",
            column,
            value
        )),
        None => Ok(None),
    }
}

fn get_i64_from_row(row: &Row, column: &str) -> Result<i64> {
    match row.get(column) {
        Some(DatabaseValue::Number(n)) => Ok(n),
        Some(value) => Err(anyhow::anyhow!(
            "Expected number for column '{}', got {:?}",
            column,
            value
        )),
        None => Err(anyhow::anyhow!("Missing column '{}'", column)),
    }
}

fn row_to_game(row: &Row) -> Result<Game> {
    let state_str = get_string_from_row(row, "state")?;
    let state = match state_str.as_str() {
        "Waiting" => GameState::Waiting,
        "Voting" => GameState::Voting,
        "Revealed" => GameState::Revealed,
        _ => return Err(anyhow::anyhow!("Invalid game state: {}", state_str)),
    };

    Ok(Game {
        id: Uuid::parse_str(&get_string_from_row(row, "id")?)?,
        name: get_string_from_row(row, "name")?,
        owner_id: Uuid::parse_str(&get_string_from_row(row, "owner_id")?)?,
        voting_system: get_string_from_row(row, "voting_system")?,
        state,
        current_story: get_optional_string_from_row(row, "current_story")?,
        created_at: DateTime::parse_from_rfc3339(&get_string_from_row(row, "created_at")?)?
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(&get_string_from_row(row, "updated_at")?)?
            .with_timezone(&Utc),
    })
}

fn row_to_player(row: &Row) -> Result<Player> {
    Ok(Player {
        id: Uuid::parse_str(&get_string_from_row(row, "id")?)?,
        name: get_string_from_row(row, "name")?,
        is_observer: get_i64_from_row(row, "is_observer")? != 0,
        joined_at: DateTime::parse_from_rfc3339(&get_string_from_row(row, "joined_at")?)?
            .with_timezone(&Utc),
    })
}

fn rows_to_players(rows: Vec<Row>) -> Result<Vec<Player>> {
    rows.iter().map(row_to_player).collect()
}

#[async_trait]
impl SessionManager for DatabaseSessionManager {
    async fn create_game(
        &self,
        name: String,
        voting_system: String,
        owner_id: Uuid,
    ) -> Result<Game> {
        let game_id = Uuid::new_v4();
        let now = Utc::now();

        self.db
            .insert("games")
            .value("id", DatabaseValue::String(game_id.to_string()))
            .value("name", DatabaseValue::String(name.clone()))
            .value("owner_id", DatabaseValue::String(owner_id.to_string()))
            .value(
                "voting_system",
                DatabaseValue::String(voting_system.clone()),
            )
            .value("state", DatabaseValue::String("Waiting".to_string()))
            .value("current_story", DatabaseValue::Null)
            .value("created_at", DatabaseValue::String(now.to_rfc3339()))
            .value("updated_at", DatabaseValue::String(now.to_rfc3339()))
            .execute(&**self.db)
            .await?;

        let game = Game {
            id: game_id,
            name,
            owner_id,
            voting_system,
            state: GameState::Waiting,
            current_story: None,
            created_at: now,
            updated_at: now,
        };

        tracing::info!("Created game: {:?}", game);
        Ok(game)
    }

    async fn get_game(&self, game_id: Uuid) -> Result<Option<Game>> {
        tracing::info!("Getting game: {}", game_id);

        let result = self
            .db
            .select("games")
            .where_eq("id", DatabaseValue::String(game_id.to_string()))
            .execute_first(&**self.db)
            .await?;

        match result {
            Some(row) => {
                let game = row_to_game(&row)?;
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    async fn update_game(&self, game: &Game) -> Result<()> {
        // TODO: Implement database update
        tracing::info!("Updating game: {:?}", game);
        Ok(())
    }

    async fn delete_game(&self, game_id: Uuid) -> Result<()> {
        // TODO: Implement database deletion
        tracing::info!("Deleting game: {}", game_id);
        Ok(())
    }

    async fn add_player_to_game(&self, game_id: Uuid, player: Player) -> Result<()> {
        tracing::info!("Adding player {} to game {}", player.id, game_id);

        self.db
            .insert("players")
            .value("id", DatabaseValue::String(player.id.to_string()))
            .value("game_id", DatabaseValue::String(game_id.to_string()))
            .value("name", DatabaseValue::String(player.name))
            .value(
                "is_observer",
                DatabaseValue::Number(if player.is_observer { 1 } else { 0 }),
            )
            .value(
                "joined_at",
                DatabaseValue::String(player.joined_at.to_rfc3339()),
            )
            .execute(&**self.db)
            .await?;

        Ok(())
    }

    async fn remove_player_from_game(&self, game_id: Uuid, player_id: Uuid) -> Result<()> {
        // TODO: Implement database deletion
        tracing::info!("Removing player {} from game {}", player_id, game_id);
        Ok(())
    }

    async fn get_game_players(&self, game_id: Uuid) -> Result<Vec<Player>> {
        tracing::info!("Getting players for game: {}", game_id);

        let rows = self
            .db
            .select("players")
            .where_eq("game_id", DatabaseValue::String(game_id.to_string()))
            .execute(&**self.db)
            .await?;

        let players = rows_to_players(rows)?;
        Ok(players)
    }

    async fn cast_vote(&self, game_id: Uuid, vote: Vote) -> Result<()> {
        // TODO: Implement database insertion/update
        tracing::info!("Casting vote for game {}: {:?}", game_id, vote);
        Ok(())
    }

    async fn get_game_votes(&self, game_id: Uuid) -> Result<Vec<Vote>> {
        // TODO: Implement database query
        tracing::info!("Getting votes for game: {}", game_id);
        Ok(vec![])
    }

    async fn clear_game_votes(&self, game_id: Uuid) -> Result<()> {
        // TODO: Implement database deletion
        tracing::info!("Clearing votes for game: {}", game_id);
        Ok(())
    }

    async fn create_session(&self, session: Session) -> Result<()> {
        // TODO: Implement database insertion
        tracing::info!("Creating session: {:?}", session);
        Ok(())
    }

    async fn get_session(&self, connection_id: &str) -> Result<Option<Session>> {
        // TODO: Implement database query
        tracing::info!("Getting session: {}", connection_id);
        Ok(None)
    }

    async fn update_session_last_seen(&self, connection_id: &str) -> Result<()> {
        // TODO: Implement database update
        tracing::info!("Updating session last seen: {}", connection_id);
        Ok(())
    }

    async fn delete_session(&self, connection_id: &str) -> Result<()> {
        // TODO: Implement database deletion
        tracing::info!("Deleting session: {}", connection_id);
        Ok(())
    }

    async fn cleanup_expired_sessions(&self) -> Result<()> {
        // TODO: Implement cleanup logic
        tracing::info!("Cleaning up expired sessions");
        Ok(())
    }

    async fn start_voting(&self, game_id: Uuid, story: String) -> Result<()> {
        tracing::info!("Starting voting for game {} with story: {}", game_id, story);

        let now = Utc::now();
        self.db
            .update("games")
            .value("state", DatabaseValue::String("Voting".to_string()))
            .value("current_story", DatabaseValue::String(story))
            .value("updated_at", DatabaseValue::String(now.to_rfc3339()))
            .where_eq("id", DatabaseValue::String(game_id.to_string()))
            .execute(&**self.db)
            .await?;

        Ok(())
    }

    async fn reset_voting(&self, game_id: Uuid) -> Result<()> {
        tracing::info!("Resetting voting for game {}", game_id);

        let now = Utc::now();

        // Clear all votes for this game
        self.db
            .delete("votes")
            .where_eq("game_id", DatabaseValue::String(game_id.to_string()))
            .execute(&**self.db)
            .await?;

        // Reset game state to Waiting
        self.db
            .update("games")
            .value("state", DatabaseValue::String("Waiting".to_string()))
            .value("current_story", DatabaseValue::Null)
            .value("updated_at", DatabaseValue::String(now.to_rfc3339()))
            .where_eq("id", DatabaseValue::String(game_id.to_string()))
            .execute(&**self.db)
            .await?;

        Ok(())
    }
}
