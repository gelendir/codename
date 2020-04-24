use serde::ser::{Serialize, Serializer, SerializeStruct};
use crate::request;
use crate::response;
use crate::game::{Game, State};
use crate::error::RoomError;
use crate::board::BoardSet;
use mio::Token;
use uuid::Uuid;
use tungstenite::Message;
use std::rc::Rc;
use std::collections::HashMap;
use std::result;


pub type Responses = Vec<(Token, Message)>;

#[derive(Debug)]
pub struct Room {
    pub id: Uuid,
    pub game: Game,
    pub players: HashMap<Token, String>,
    boards: Rc<BoardSet>,
    admin: Token
}

impl Serialize for Room {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let players: Vec<&String> = self.players.values().collect();

        let state = match self.game.state {
            State::Start if self.players.len() >= 4 => "team",
            State::Start => "join",
            State::Play(_) => "play",
            State::End(_) => "end"
        };

        let mut s = serializer.serialize_struct("Room", 4)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("game", &self.game)?;
        s.serialize_field("players", &players)?;
        s.serialize_field("state", state)?;
        s.end()
    }
}

impl Room {

    pub fn new(boards: Rc<BoardSet>, admin: Token, request: &request::Room) -> Result<Room, RoomError> {
        let board = boards.new_board(&request.language)?;

        let mut players = HashMap::new();
        players.insert(admin, request.name.clone());

        Ok(Room {
            id: Uuid::new_v4(),
            game: Game::new(board, admin),
            boards: boards,
            players: players,
            admin: admin,
        })
    }

    pub fn is_alive(&self, token: Token) -> bool {
        token != self.admin && self.players.len() > 0
    }

    pub fn remove_player(&mut self, token: Token) -> Responses {
        if let Some(name) = self.players.remove(&token) {
            log::info!("{} - removing player {}", self.id, name);
            self.game.remove_player(token);
            self.broadcast(response::room(&self))
        } else {
            Vec::new()
        }
    }

    pub fn broadcast_room(&self) -> Responses {
        vec![(self.admin, response::room(self))]
    }

    pub fn handle(&mut self, token: Token, request: &request::Request) -> Responses {
        let result = match request {
            request::Request::Join(j) => self.join(token, j),
            request::Request::Team(t) => self.team(token, t),
            request::Request::Start(s) => self.start(token, s),
            request::Request::Hint(h) => self.hint(token, h),
            request::Request::Guess(g) => self.guess(token, g),
            request::Request::Pass(_) => self.pass(token),
            request::Request::Reset(r) => self.reset(token, r),
            _ => {
                return vec![(token, response::error("unhandled request"))]
            }
        };

        match result {
            Ok(r) => r,
            Err(error) => {
                vec![(token, response::error(&error.to_string()))]
            }
        }
    }

    fn broadcast(&mut self, response: Message) -> Responses {
        self.players.keys()
            .map(|t| {
                log::debug!("broadcasting to {}", t.0);
                (*t, response.clone())
            })
            .collect()
    }

    fn reset(&mut self, token: Token, reset: &request::Reset) -> Result<Responses, RoomError> {
        log::info!("{} - game reset", self.id);

        if token == self.admin {
            let board = self.boards.new_board(&reset.language)?;
            self.game = Game::new(board, self.admin);
            let response = response::room(&self);
            Ok(self.broadcast(response))
        } else {
            let response = response::error("player not admin");
            Ok(vec![(token, response)])
        }
    }

    fn join(&mut self, token: Token, join: &request::Join) -> Result<Responses, RoomError> {
        log::info!("{} - {} has joined", self.id, join.name);

        self.players.insert(token, join.name.clone());
        let response = response::room(&self);
        Ok(self.broadcast(response))
    }

    fn team(&mut self, token: Token, team: &request::Team) -> Result<Responses, RoomError> {
        if let Some(name) = self.players.get(&token) {
            log::info!("{} - player {:?} joined team {:?}", self.id, name, team.team);

            self.game.add_player(token, team.team, &name);
            let response = response::room(&self);
            Ok(self.broadcast(response))
        } else {
            let response = response::error("player not found in game");
            Ok(vec![(token, response)])
        }
    }

    fn start(&mut self, token: Token, start: &request::Start) -> Result<Responses, RoomError> {
        self.game.start(token, start)?;
        log::info!("{} - game started", self.id);

        let mut responses = self.broadcast(response::room(&self));

        let response = response::tiles(&self.game);

        if let Some(master) = self.game.red.master {
            responses.push((master, response.clone()))
        }

        if let Some(master) = self.game.blue.master {
            responses.push((master, response.clone()))
        }

        Ok(responses)
    }

    fn hint(&mut self, token: Token, hint: &request::Hint) -> Result<Responses, RoomError> {
        self.game.hint(token, &hint)?;
        log::info!("{} - hint {:?}", self.id, hint);

        Ok(self.broadcast(response::room(&self)))
    }

    fn guess(&mut self, token: Token, guess: &request::Guess) -> Result<Responses, RoomError> {
        self.game.guess(token, &guess)?;
        log::info!("{} - guess {} {}", self.id, guess.x, guess.y);

        Ok(self.broadcast(response::room(&self)))
    }

    fn pass(&mut self, token: Token) -> Result<Responses, RoomError> {
        self.game.pass(token)?;
        log::info!("{} - pass", self.id);

        Ok(self.broadcast(response::room(&self)))
    }

}
