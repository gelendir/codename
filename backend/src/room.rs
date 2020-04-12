use crate::board::Board;
use crate::request;
use crate::response;
use crate::game::{Game, State};
use crate::connection::Connection;
use crate::connection;
use anyhow::{Result, anyhow};
use mio::Token;
use uuid::Uuid;
use std::collections::HashMap;
use tungstenite::Message;

pub struct Room {
    pub id: Uuid,
    pub game: Game,
    pub connections: HashMap<Token, Connection>
}

impl Room {

    pub fn new(board: Board, admin: Token) -> Room {
        Room {
            id: Uuid::new_v4(),
            game: Game::new(board, admin),
            connections: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.connections.len()
    }

    pub fn add_conn(&mut self, token: Token, conn: Connection) {
        log::debug!("{} - adding {} {:?}", self.id, token.0, conn);
        self.connections.insert(token, conn);
    }

    pub fn remove_conn(&mut self, token: Token) -> Option<(String, Connection)> {
        if let Some(player) = self.game.remove_player(token) {
            if let Some(conn) = self.connections.remove(&token) {
                return Some((player, conn));
            }
        }

        log::warn!("{} - token {} not found", self.id, token.0);
        None
    }

    pub fn room_response(&mut self) -> Result<()> {
        self.broadcast(response::room(&self))
    }

    pub fn remove_response(&mut self, player: String) -> Result<()> {
        log::info!("{} - player {} removed", self.id, player);
        self.broadcast(response::leave(&self.game, &player))?;
        match self.game.state {
            State::Start => {
                log::info!("{} - reselect new masters", self.id);
                self.broadcast(response::restart(&self.game))
            }
            _ => Ok(())
        }
    }

    pub fn handle(&mut self, token: Token) -> Result<()> {
        loop {
            if let Some(request) = self.read_request(token)? {
                let result = match request {
                    request::Request::Team(t) => self.join_team(token, t),
                    request::Request::Start(s) => self.start(token, s),
                    request::Request::Hint(h) => self.hint(token, h),
                    request::Request::Guess(g) => self.guess(token, g),
                    request::Request::Pass(_) => self.pass(token),
                    _ => Err(anyhow!("request not handled during game")),
                };

                if let Err(error) = result {
                    if let Some(e) = error.downcast_ref::<connection::Error>() {
                        if let connection::Error::ParseError(ee) = e {
                            self.send_error(token, &ee)?;
                        }
                    }
                }
            } else {
                return Ok(());
            }
        }
    }

    fn read_request(&mut self, token: Token) -> Result<Option<request::Request>> {
        if let Some(conn) = self.connections.get_mut(&token) {
            return Ok(conn.read()?);
        }
        Ok(None)
    }

    fn broadcast(&mut self, response: Message) -> Result<()> {
        for conn in self.connections.values_mut() {
            conn.send(response.clone())?;
        }
        Ok(())
    }

    fn join_team(&mut self, token: Token, team: request::Team) -> Result<()> {
        let name = team.name.clone();
        log::info!("{} - player {} joined", self.id, name);

        self.game.add_player(token, team.team, team.name);
        let response = response::join(&self.game, &name);
        self.broadcast(response)
    }

    fn start(&mut self, token: Token, start: request::Start) -> Result<()> {
        self.game.start(token, start)?;
        log::info!("{} - game started", self.id);

        let response = response::tiles(&self.game);

        if let Some(master) = self.game.red.master {
            if let Some(conn) = self.connections.get_mut(&master) {
                conn.send(response.clone())?;
            }
        }

        if let Some(master) = self.game.blue.master {
            if let Some(conn) = self.connections.get_mut(&master) {
                conn.send(response.clone())?;
            }
        }

        self.broadcast(
            response::turn(&self.game)
        )
    }

    fn hint(&mut self, token: Token, hint: request::Hint) -> Result<()> {
        self.game.hint(token, &hint)?;
        log::info!("{} - hint word: {} guesses: {}", self.id, hint.hint, hint.guesses);

        self.broadcast(
            response::hint(&self.game)
        )
    }

    fn guess(&mut self, token: Token, guess: request::Guess) -> Result<()> {
        self.game.guess(token, &guess)?;
        log::info!("{} - guess {} {}", self.id, guess.x, guess.y);

        match self.game.state {
            State::End(winner) => {
                log::info!("{} - game ended. winner: {:?}", self.id, winner);
                 self.broadcast(response::end(&self.game))
            },
            _ => self.broadcast(response::turn(&self.game))
        }
    }

    fn pass(&mut self, token: Token) -> Result<()> {
        self.game.pass(token)?;
        log::info!("{} - pass", self.id);

        self.broadcast(
            response::turn(&self.game)
        )
    }

    fn send_error(&mut self, token: Token, error: &anyhow::Error) -> Result<()> {
        if let Some(conn) = self.connections.get_mut(&token) {
            let msg = format!("{}", error);
            let response = response::error(&msg);
            conn.send(response)?;
        }
        Ok(())
    }

}
