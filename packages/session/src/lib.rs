#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use moosicbox_json_utils::ToValueType;
use planning_poker_database::{Database, DatabaseValue};
use planning_poker_models::{Game, GameState, Player, Session, Vote};
use switchy::database::query::FilterableQuery;
use tracing::warn;
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
    async fn reveal_votes(&self, game_id: Uuid) -> Result<()>;
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
    #[must_use]
    pub fn new(db: Box<dyn Database>) -> Self {
        Self {
            db: std::sync::Arc::new(db),
        }
    }

    /// Initialize the database schema by running migrations
    ///
    /// # Errors
    ///
    /// Returns an error if the database migrations fail
    pub async fn init_schema(&self) -> Result<()> {
        tracing::info!("Running database migrations...");

        planning_poker_schema::migrate(&**self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

        tracing::info!("Database migrations completed successfully");
        Ok(())
    }
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
            .value("created_at", DatabaseValue::Now)
            .value("updated_at", DatabaseValue::Now)
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
                let game: Game = row
                    .to_value_type()
                    .map_err(|e| anyhow::anyhow!("Failed to convert row to Game: {}", e))?;
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    async fn update_game(&self, game: &Game) -> Result<()> {
        tracing::info!("Updating game: {:?}", game);

        let state_str = match game.state {
            GameState::Waiting => "Waiting",
            GameState::Voting => "Voting",
            GameState::Revealed => "Revealed",
        };

        self.db
            .update("games")
            .value("name", DatabaseValue::String(game.name.clone()))
            .value(
                "voting_system",
                DatabaseValue::String(game.voting_system.clone()),
            )
            .value("state", DatabaseValue::String(state_str.to_string()))
            .value(
                "current_story",
                game.current_story
                    .as_ref()
                    .map_or(DatabaseValue::Null, |story| {
                        DatabaseValue::String(story.clone())
                    }),
            )
            .value("updated_at", DatabaseValue::Now)
            .where_eq("id", DatabaseValue::String(game.id.to_string()))
            .execute(&**self.db)
            .await?;

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
            .value("is_observer", DatabaseValue::Bool(player.is_observer))
            .value("joined_at", DatabaseValue::Now)
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

        let players: Vec<Player> = rows
            .iter()
            .map(|row| {
                row.to_value_type()
                    .map_err(|e| anyhow::anyhow!("Failed to convert row to Player: {}", e))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(players)
    }

    async fn cast_vote(&self, game_id: Uuid, vote: Vote) -> Result<()> {
        tracing::info!("Casting vote for game {}: {:?}", game_id, vote);

        // First, delete any existing vote from this player for this game
        self.db
            .delete("votes")
            .where_eq("game_id", DatabaseValue::String(game_id.to_string()))
            .where_eq(
                "player_id",
                DatabaseValue::String(vote.player_id.to_string()),
            )
            .execute(&**self.db)
            .await?;

        // Insert the new vote
        self.db
            .insert("votes")
            .value("game_id", DatabaseValue::String(game_id.to_string()))
            .value(
                "player_id",
                DatabaseValue::String(vote.player_id.to_string()),
            )
            .value("player_name", DatabaseValue::String(vote.player_name))
            .value("value", DatabaseValue::String(vote.value))
            .value("cast_at", DatabaseValue::Now)
            .execute(&**self.db)
            .await?;

        Ok(())
    }

    async fn get_game_votes(&self, game_id: Uuid) -> Result<Vec<Vote>> {
        tracing::info!("Getting votes for game: {}", game_id);

        let rows = self
            .db
            .select("votes")
            .where_eq("game_id", DatabaseValue::String(game_id.to_string()))
            .execute(&**self.db)
            .await?;

        let votes: Vec<Vote> = rows
            .iter()
            .map(|row| {
                row.to_value_type()
                    .map_err(|e| anyhow::anyhow!("Failed to convert row to Vote: {}", e))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(votes)
    }

    async fn clear_game_votes(&self, game_id: Uuid) -> Result<()> {
        tracing::info!("Clearing votes for game: {}", game_id);

        self.db
            .delete("votes")
            .where_eq("game_id", DatabaseValue::String(game_id.to_string()))
            .execute(&**self.db)
            .await?;

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

        self.db
            .update("games")
            .value("state", DatabaseValue::String("Voting".to_string()))
            .value("current_story", DatabaseValue::String(story))
            .value("updated_at", DatabaseValue::Now)
            .where_eq("id", DatabaseValue::String(game_id.to_string()))
            .execute(&**self.db)
            .await?;

        Ok(())
    }

    async fn reveal_votes(&self, game_id: Uuid) -> Result<()> {
        tracing::info!("Revealing votes for game {}", game_id);

        self.db
            .update("games")
            .value("state", DatabaseValue::String("Revealed".to_string()))
            .value("updated_at", DatabaseValue::Now)
            .where_eq("id", DatabaseValue::String(game_id.to_string()))
            .execute(&**self.db)
            .await?;

        Ok(())
    }

    async fn reset_voting(&self, game_id: Uuid) -> Result<()> {
        tracing::info!("Resetting voting for game {}", game_id);

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
            .value("updated_at", DatabaseValue::Now)
            .where_eq("id", DatabaseValue::String(game_id.to_string()))
            .execute(&**self.db)
            .await?;

        Ok(())
    }
}
