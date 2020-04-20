use serde::Serialize as SerdeSerialize;
use serde::ser::{Serialize, Serializer, SerializeStruct};
use crate::request::Hint;
use crate::team::Team;
use crate::board::Tile;
use crate::request;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use mio::Token;
use std::result;

#[derive(Debug, SerdeSerialize)]
#[serde(rename_all="lowercase")]
pub enum State {
    Hint,
    Guess
}

#[derive(Debug)]
pub struct GameTeam {
    pub team: Team,
    pub guesses: u8,
    pub hint: String,
    pub previous: Option<String>,
    pub players: HashMap<Token, String>,
    pub state: State,
    pub master: Option<Token>,
}

impl Serialize for GameTeam {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let players: Vec<&String> = self.players.values().collect();
        let master = if let Some(token) = self.master {
            self.players.get(&token)
        } else {
            None
        };

        let mut s = serializer.serialize_struct("GameTeam", 5)?;
        s.serialize_field("master", &master)?;
        s.serialize_field("hint", &self.hint)?;
        s.serialize_field("guesses", &self.guesses)?;
        s.serialize_field("previous", &self.previous)?;
        s.serialize_field("players", &players)?;
        s.end()
    }
}

impl GameTeam {

    pub fn new(team: Team) -> GameTeam {
        GameTeam {
            team: team,
            master: None,
            hint: String::new(),
            guesses: 0,
            previous: None,
            state: State::Hint,
            players: HashMap::new()
        }
    }

    pub fn add_player(&mut self, token: Token, player: String) {
        self.players.insert(token, player);
    }

    pub fn remove_player(&mut self, token: Token) -> Option<String> {
        self.players.remove(&token)
    }

    pub fn nb_players(&self) -> usize {
        self.players.len()
    }

    pub fn has_master(&self) -> bool {
        self.master.is_some()
    }

    pub fn set_master(&mut self, start: &request::Start) -> Result<()> {
        let name = start.master(&self.team);

        self.master = self.players.iter()
            .filter(|(_, p)| *p == name)
            .map(|(t, _)| *t)
            .next();

        if self.master.is_none() {
            return Err(anyhow!("master not found"))
        }
        Ok(())
    }

    pub fn give_hint(&mut self, token: Token, hint: &Hint) -> Result<()> {
        self.validate_player(token, true)?;

        match self.state {
            State::Guess => Err(anyhow!("not time to give a hint")),
            State::Hint => {
                if self.guesses > 0 || self.previous.is_some() {
                    self.previous = Some(self.hint.clone());
                }

                self.hint = hint.hint.clone();
                self.guesses = hint.guesses;
                self.state = State::Guess;

                Ok(())
            }
        }
    }

    pub fn next_team(&mut self, token: Token, tile: Tile) -> Result<Team> {
        self.validate_player(token, false)?;

        match self.state {
            State::Hint => Err(anyhow!("not time to give a hint")),
            State::Guess => {
                match tile {
                    Tile::Red if self.team == Team::Red => {
                        Ok(self.decrease_guess())
                    },
                    Tile::Blue if self.team == Team::Blue => {
                        Ok(self.decrease_guess())
                    },
                    _ => {
                        self.state = State::Hint;
                        Ok(self.team.opposite())
                    }
                }
            }
        }
    }

    pub fn decrease_guess(&mut self) -> Team {
        if self.guesses > 0 {
            self.guesses -= 1;
        } else {
            self.previous = None;
        }

        if self.guesses > 0 || self.previous.is_some() {
            self.team
        } else {
            self.state = State::Hint;
            self.team.opposite()
        }
    }

    pub fn pass(&mut self, token: Token) -> Result<()> {
        self.validate_player(token, false)?;
        self.state = State::Hint;
        Ok(())
    }

    fn validate_player(&self, token: Token, master: bool) -> Result<()> {
        if master {
            match self.master {
                Some(t) if token == t => Ok(()),
                _ => Err(anyhow!("player not master"))
            }
        } else {
            match self.players.get(&token) {
                Some(_) => Ok(()),
                None => Err(anyhow!("player not found in team"))
            }
        }
    }

    pub fn can_guess(&self, token: Token) -> bool {
        if !self.players.contains_key(&token) {
            return false
        }

        if let Some(master) = self.master {
            if token == master {
                return false
            }
        }

        match self.state {
            State::Guess => self.guesses > 0 || self.previous.is_some(),
            State::Hint => false
        }
    }
}
