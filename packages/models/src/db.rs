use std::str::FromStr;

use moosicbox_json_utils::{database::ToValue as _, ParseError, ToValueType};
use switchy::database::{DatabaseValue, Row};
use uuid::Uuid;

use crate::{Game, GameState, Player, Vote};

// ToValueType implementations following MoosicBox pattern

// Implement MissingValue for our local types
impl moosicbox_json_utils::MissingValue<GameState> for &Row {}
impl moosicbox_json_utils::MissingValue<Game> for &Row {}
impl moosicbox_json_utils::MissingValue<Player> for &Row {}
impl moosicbox_json_utils::MissingValue<Vote> for &Row {}

// ToValueType for GameState (local type, so orphan rule allows this)
impl ToValueType<GameState> for DatabaseValue {
    fn to_value_type(self) -> Result<GameState, ParseError> {
        let state_str: String = (&self).to_value_type()?;
        match state_str.as_str() {
            "Waiting" => Ok(GameState::Waiting),
            "Voting" => Ok(GameState::Voting),
            "Revealed" => Ok(GameState::Revealed),
            _ => Err(ParseError::ConvertType(format!(
                "Invalid GameState: {state_str}"
            ))),
        }
    }
}

// ToValueType for Game (local type, so orphan rule allows this)
impl ToValueType<Game> for &Row {
    fn to_value_type(self) -> Result<Game, ParseError> {
        Ok(Game {
            id: {
                let uuid_str: String = self.to_value("id")?;
                Uuid::from_str(&uuid_str)
                    .map_err(|e| ParseError::ConvertType(format!("Invalid Uuid in id: {e}")))?
            },
            name: self.to_value("name")?,
            owner_id: {
                let uuid_str: String = self.to_value("owner_id")?;
                Uuid::from_str(&uuid_str).map_err(|e| {
                    ParseError::ConvertType(format!("Invalid Uuid in owner_id: {e}"))
                })?
            },
            voting_system: self.to_value("voting_system")?,
            state: self.to_value("state")?,
            current_story: self.to_value("current_story")?,
            created_at: self.to_value("created_at")?,
            updated_at: self.to_value("updated_at")?,
        })
    }
}

// ToValueType for Player (local type, so orphan rule allows this)
impl ToValueType<Player> for &Row {
    fn to_value_type(self) -> Result<Player, ParseError> {
        Ok(Player {
            id: {
                let uuid_str: String = self.to_value("id")?;
                Uuid::from_str(&uuid_str)
                    .map_err(|e| ParseError::ConvertType(format!("Invalid Uuid in id: {e}")))?
            },
            name: self.to_value("name")?,
            is_observer: self.to_value("is_observer")?,
            joined_at: self.to_value("joined_at")?,
        })
    }
}

// ToValueType for Vote (local type, so orphan rule allows this)
impl ToValueType<Vote> for &Row {
    fn to_value_type(self) -> Result<Vote, ParseError> {
        Ok(Vote {
            player_id: {
                let uuid_str: String = self.to_value("player_id")?;
                Uuid::from_str(&uuid_str).map_err(|e| {
                    ParseError::ConvertType(format!("Invalid Uuid in player_id: {e}"))
                })?
            },
            player_name: self.to_value("player_name")?,
            value: self.to_value("value")?,
            cast_at: self.to_value("cast_at")?,
        })
    }
}
