use crate::board::BoardSet;
use crate::room::Room;
use crate::request::Request;
use crate::response;
use crate::stream::{Stream, Event};
use anyhow::Result;
use uuid::Uuid;
use mio::Token;
use std::collections::HashMap;



pub struct Server {
    stream: Stream,
    players: HashMap<Token, Uuid>,
    rooms: HashMap<Uuid, Room>,
    boardset: BoardSet,
}


impl Server {

    pub fn new(boardset: BoardSet, stream: Stream) -> Server {
        Server {
            boardset: boardset,
            stream: stream,
            players: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            for event in self.stream.poll()? {
                log::debug!("handling event: {:?}", event);
                match event {
                    Event::Close(token) => self.remove_player(token),
                    Event::Request(token, request) => self.handle_request(token, request),
                    Event::Error(token, error) => {
                        let response = response::error(&error);
                        self.stream.push(token, response)
                    }
                }
            }
        }
    }

    fn remove_player(&mut self, token: Token) {
        if let Some(id) = self.players.remove(&token) {

            let mut alive = true;
            if let Some(room) = self.rooms.get_mut(&id) {
                self.stream.append(
                    room.remove_player(token)
                );
                alive = room.is_alive(token)
            }

            if !alive {
                if let Some(room) = self.rooms.remove(&id) {
                    for token in room.game.tokens() {
                        self.stream.remove(*token)
                    }
                }
            }
        }
    }
    
    fn handle_request(&mut self, token: Token, request: Request) {
        if self.players.contains_key(&token) {
            self.handle_room(token, &request);
        } else {
            self.handle_client(token, &request);
        }
    }

    fn handle_room(&mut self, token: Token, request: &Request) {
        if let Some(id) = self.players.get(&token) {
            if let Some(room) = self.rooms.get_mut(id) {
                log::debug!("handle room: {:?} {:?}", token, request);
                let responses = room.handle(token, &request);
                self.stream.append(responses);
            }
        }
    }

    fn handle_client(&mut self, token: Token, request: &Request) {
        log::debug!("handle client: {:?} {:?}", token, request);
        match &request {
            Request::Room(_) => self.new_room(token, &request),
            Request::Join(j) if self.rooms.contains_key(&j.id) => {
                self.players.insert(token, j.id);
            },
            _ => {
                let msg = format!("invalid request. Expecting room or join");
                self.stream.push(token, response::error(&msg));
            }
        }
    }

    fn new_room(&mut self, token: Token, request: &Request) {
        log::debug!("new room: {:?} {:?}", token, request);
        let id = Uuid::new_v4();
        self.players.insert(token, id);

        let room = Room::new(id, self.boardset.new_board(), token);
        self.stream.append(room.broadcast_room());

        self.rooms.insert(id, room);
    }
}
