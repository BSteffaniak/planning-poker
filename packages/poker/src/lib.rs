use anyhow::Result;
use planning_poker_models::{GameState, Player, Vote};
use std::collections::HashMap;
use uuid::Uuid;

pub struct PlanningPokerGame {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub state: GameState,
    pub players: HashMap<Uuid, Player>,
    pub votes: HashMap<Uuid, Vote>,
    pub current_story: Option<String>,
    pub voting_system: VotingSystem,
}

#[derive(Debug, Clone)]
pub enum VotingSystem {
    Fibonacci,
    TShirtSizes,
    PowersOfTwo,
    Custom(Vec<String>),
}

impl PlanningPokerGame {
    pub fn new(name: String, owner_id: Uuid, voting_system: VotingSystem) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            owner_id,
            state: GameState::Waiting,
            players: HashMap::new(),
            votes: HashMap::new(),
            current_story: None,
            voting_system,
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<()> {
        self.players.insert(player.id, player);
        Ok(())
    }

    pub fn remove_player(&mut self, player_id: Uuid) -> Result<()> {
        self.players.remove(&player_id);
        self.votes.remove(&player_id);
        Ok(())
    }

    pub fn start_voting(&mut self, story: String) -> Result<()> {
        if self.state != GameState::Waiting {
            return Err(anyhow::anyhow!("Cannot start voting in current state"));
        }

        self.current_story = Some(story);
        self.state = GameState::Voting;
        self.votes.clear();
        Ok(())
    }

    pub fn cast_vote(&mut self, player_id: Uuid, vote: Vote) -> Result<()> {
        if self.state != GameState::Voting {
            return Err(anyhow::anyhow!("Not in voting state"));
        }

        if !self.players.contains_key(&player_id) {
            return Err(anyhow::anyhow!("Player not in game"));
        }

        self.votes.insert(player_id, vote);
        Ok(())
    }

    pub fn reveal_votes(&mut self) -> Result<()> {
        if self.state != GameState::Voting {
            return Err(anyhow::anyhow!("Not in voting state"));
        }

        self.state = GameState::Revealed;
        Ok(())
    }

    pub fn reset_voting(&mut self) -> Result<()> {
        self.state = GameState::Waiting;
        self.votes.clear();
        self.current_story = None;
        Ok(())
    }

    pub fn get_voting_options(&self) -> Vec<String> {
        match &self.voting_system {
            VotingSystem::Fibonacci => vec![
                "0".to_string(),
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "5".to_string(),
                "8".to_string(),
                "13".to_string(),
                "21".to_string(),
                "34".to_string(),
                "55".to_string(),
                "89".to_string(),
                "?".to_string(),
            ],
            VotingSystem::TShirtSizes => vec![
                "XS".to_string(),
                "S".to_string(),
                "M".to_string(),
                "L".to_string(),
                "XL".to_string(),
                "XXL".to_string(),
                "?".to_string(),
            ],
            VotingSystem::PowersOfTwo => vec![
                "1".to_string(),
                "2".to_string(),
                "4".to_string(),
                "8".to_string(),
                "16".to_string(),
                "32".to_string(),
                "64".to_string(),
                "?".to_string(),
            ],
            VotingSystem::Custom(options) => options.clone(),
        }
    }

    pub fn is_owner(&self, player_id: Uuid) -> bool {
        self.owner_id == player_id
    }

    pub fn all_players_voted(&self) -> bool {
        self.players.len() == self.votes.len()
    }
}
