use serde::ser::{Serialize, Serializer, SerializeStruct};
use crate::request;
use crate::board::Board;
use crate::board::Tile;
use crate::team::Team;
use crate::gameteam::GameTeam;
use crate::error::GameError;
use mio::Token;
use std::result;

type Result<T> = result::Result<T, GameError>;


#[derive(Debug)]
pub enum State {
    Start,
    Play(Team),
    End(Team)
}

#[derive(Debug)]
pub struct Game {
    pub admin: Token,
    pub board: Board,
    pub state: State,
    pub red: GameTeam,
    pub blue: GameTeam,
}

impl Serialize for Game {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let turn = match self.state {
            State::Start => self.board.start_team(),
            State::Play(team) => team.clone(),
            State::End(team) => team.clone(),
        };

        let action = match turn {
            Team::Blue => &self.blue.state,
            Team::Red => &self.red.state
        };

        let mut s = serializer.serialize_struct("Game", 4)?;
        s.serialize_field("board", &self.board)?;
        s.serialize_field("red", &self.red)?;
        s.serialize_field("blue", &self.blue)?;
        s.serialize_field("turn", &turn)?;
        s.serialize_field("action", action)?;
        s.end()
    }
}


impl Game {

    pub fn new(board: Board, admin: Token) -> Game {
        Game {
            admin: admin,
            board: board,
            state: State::Start,
            red: GameTeam::new(Team::Red),
            blue: GameTeam::new(Team::Blue),
        }
    }

    pub fn team_mut(&mut self, team: &Team) -> &mut GameTeam {
        match team {
            Team::Red => &mut self.red,
            Team::Blue => &mut self.blue
        }
    }

    pub fn team(&self, team: &Team) -> &GameTeam {
        match team {
            Team::Red => &self.red,
            Team::Blue => &self.blue
        }
    }

    pub fn add_player(&mut self, token: Token, team: Team, player: &str) {
        self.team_mut(&team).add_player(token, player.to_string());
    }

    pub fn remove_player(&mut self, token: Token) -> Option<String> {
        let result = self.red.remove_player(token).or(self.blue.remove_player(token));
        if result.is_some() {
            for team in [&self.red, &self.blue].iter() {
                if !(team.has_master() && team.nb_players() >= 2) {
                    self.state = State::Start
                }
            }
        }
        result
    }

    pub fn nb_players(&self) -> usize {
        self.red.nb_players() + self.blue.nb_players()
    }

    pub fn tokens(&self) -> impl Iterator<Item = &Token> {
        self.red.players.keys().chain(self.blue.players.keys())
    }

    pub fn start(&mut self, token: Token, start: &request::Start) -> Result<()> {
        if token != self.admin {
            return Err(GameError::NotAdmin)
        }

        if self.blue.nb_players() < 2 {
            return Err(GameError::MissingPlayers("blue"))
        }

        if self.red.nb_players() < 2 {
            return Err(GameError::MissingPlayers("red"))
        }

        match self.state {
            State::Start => {
                self.red.set_master(&start)?;
                self.blue.set_master(&start)?;
                self.state = State::Play(self.board.start_team());
            }
            _ => return Err(GameError::AlreadyStarted)
        }

        Ok(())
    }

    pub fn hint(&mut self, token: Token, hint: &request::Hint) -> Result<()> {
        match self.state {
            State::Play(team) => {
                let gameteam = self.team_mut(&team);
                gameteam.give_hint(token, &hint)?;
                log::debug!("gave hint: {:?}", gameteam);
            },
            _ => {
                return Err(GameError::Turn("hint"))
            }
        }

        Ok(())
    }

    pub fn guess(&mut self, token: Token, guess: &request::Guess) -> Result<()> {
        match self.state {
            State::Play(team) => {
                if !self.team(&team).can_guess(token) {
                    return Err(GameError::Turn("guess"))
                }

                let tile = self.board.put_card(guess.x, guess.y);
                let gameteam = self.team_mut(&team);
                log::debug!("tile: {:?} team: {:?} gameteam: {:?}", tile, team, gameteam);

                match tile {
                    Tile::Blue | Tile::Red | Tile::Neutral => {
                        let next_team = gameteam.next_team(token, tile)?;
                        self.state = State::Play(next_team);
                    },
                    Tile::Death => {
                        self.state = State::End(team.opposite())
                    }
                }
            },
            _ => return Err(GameError::NotStarted)
        }

        if let Some(winner) = self.board.winner() {
            self.state = State::End(winner);
        }

        Ok(())
    }

    pub fn pass(&mut self, token: Token) -> Result<()> {
        match self.state {
            State::Play(team) => {
                self.team_mut(&team).pass(token)?;
                self.state = State::Play(team.opposite())
            },
            _ => return Err(GameError::NotStarted)
        }
        Ok(())
    }
}
