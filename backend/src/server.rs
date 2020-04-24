use crate::board::BoardSet;
use crate::room::Room;
use crate::request;
use crate::response;
use crate::stream::{Stream, Event};
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
                match event {
                    Event::Request(token, request) => self.handle_request(token, request),
                    Event::Close(token) => self.remove_player(token),
                    Event::Error(token, error) => {
                        let response = response::error(&error.to_string());
                        self.stream.push(token, response)
                    }
                }
            }
        }
    }

    fn remove_player(&mut self, token: Token) {
        if let Some(id) = self.players.remove(&token) {

            let remove = if let Some(room) = self.rooms.get_mut(&id) {
                self.stream.append(
                    room.remove_player(token)
                );
                !room.is_alive(token) 
            } else {
                false
            };

            if remove {
                log::info!("{} - closing room", id);
                if let Some(room) = self.rooms.remove(&id) {
                    for token in room.game.tokens() {
                        self.stream.remove(*token)
                    }
                }
            }
        }
    }
    
    fn handle_request(&mut self, token: Token, request: request::Request) {
        if self.players.contains_key(&token) {
            self.handle_room(token, &request);
        } else {
            self.handle_client(token, &request);
        }
    }

    fn handle_room(&mut self, token: Token, request: &request::Request) {
        if let Some(id) = self.players.get(&token) {
            if let Some(room) = self.rooms.get_mut(id) {
                log::debug!("{} - handle token {} request {:?}", room.id, token.0, request);
                let responses = room.handle(token, &request);
                self.stream.append(responses);
            }
        }
    }

    fn handle_client(&mut self, token: Token, request: &request::Request) {
        match &request {
            request::Request::Room(r) => self.new_room(token, r),
            request::Request::Join(j) if self.rooms.contains_key(&j.id) => {
                log::debug!("{} - adding token {}", j.id, token.0);
                self.players.insert(token, j.id);
                self.handle_room(token, request);
            },
            _ => {
                let msg = "invalid request. Expecting room or join";
                self.stream.push(token, response::error(msg));
            }
        }
    }

    fn new_room(&mut self, token: Token, request: &request::Room) {
        let room = Room::new(self.boardset.clone(), token, request);
        log::info!("{} - new room created by {}", room.id, request.name);

        self.players.insert(token, room.id);
        self.stream.append(room.broadcast_room());
        self.rooms.insert(room.id, room);

    }
}
