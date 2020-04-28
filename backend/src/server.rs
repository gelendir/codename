use crate::board::BoardSet;
use crate::room::Room;
use crate::request;
use crate::response;
use crate::error::{GameError, RoomError};
use crate::stream::{Stream, EventKind};
use uuid::Uuid;
use mio::Token;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;


pub struct Server {
    stream: Stream,
    players: HashMap<Token, Uuid>,
    rooms: HashMap<Uuid, Room>,
    boardset: Rc<BoardSet>
}


impl Server {

    pub fn new(boardset: Rc<BoardSet>, stream: Stream) -> Server {
        Server {
            boardset: boardset,
            stream: stream,
            players: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            for event in self.stream.poll()? {
                log::debug!("handling event: {:?}", event);
                match event.kind {
                    EventKind::Request(request) => {
                        if let Err(error) = self.handle_request(event.token, request) {
                            self.stream.push(event.token, response::error(&error.to_string()))
                        }
                    }
                    EventKind::Error(error) => {
                        self.stream.push(event.token, response::error(&error.to_string()))
                    },
                    EventKind::Close => {
                        self.remove_player(event.token)
                    }
                }
            }
        }
    }

    fn remove_player(&mut self, token: Token) {
        if let Some(id) = self.players.remove(&token) {

            let mut remove = false;
            if let Some(room) = self.rooms.get_mut(&id) {
                self.stream.append(room.remove_player(token));
                remove = !room.is_alive(token) 
            }

            if remove {
                self.remove_room(id);
            }
        } 
    }
    
    fn remove_room(&mut self, id: Uuid) {
        if let Some(room) = self.rooms.remove(&id) {
            log::info!("{} - closing room", id);
            for token in room.game.tokens() {
                self.players.remove(token);
                self.stream.remove(*token);
            }
        }
    }
    
    fn handle_request(&mut self, token: Token, request: request::Request) -> Result<(), RoomError> {
        if self.players.contains_key(&token) {
            self.handle_room(token, &request)
        } else {
            self.handle_client(token, &request)
        }
    }

    fn handle_room(&mut self, token: Token, request: &request::Request) -> Result<(), RoomError> {
        let id = self.players.get(&token).ok_or(GameError::NotFound("player"))?;
        let room = self.rooms.get_mut(id).ok_or(RoomError::NotFound(id.clone()))?;

        log::debug!("{} - handle token {} request {:?}", room.id, token.0, request);
        let responses = room.handle(token, &request)?;
        self.stream.append(responses);
        
        Ok(())
    }

    fn handle_client(&mut self, token: Token, request: &request::Request) -> Result<(), RoomError> {
        match &request {
            request::Request::Room(r) => self.new_room(token, r),
            request::Request::Join(j) if self.rooms.contains_key(&j.id) => {
                log::debug!("{} - adding token {}", j.id, token.0);
                self.players.insert(token, j.id);
                self.handle_room(token, request)
            },
            _ => {
                Err(RoomError::Forbidden)
            }
        }
    }

    fn new_room(&mut self, token: Token, request: &request::Room) -> Result<(), RoomError> {
        let room = Room::new(self.boardset.clone(), token, request)?;
        log::info!("{} - new room created by {}", room.id, request.name);
        self.players.insert(token, room.id);
        self.stream.append(room.broadcast_room());
        self.rooms.insert(room.id, room);

        Ok(())
    }
}
