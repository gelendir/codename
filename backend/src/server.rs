use mio::Token;
use mio::Poll;
use crate::request::Request;
use crate::response;
use crate::connection::Connection;
use crate::connection;
use crate::board::BoardSet;
use crate::room::Room;
use mio::net::TcpStream;
use anyhow::Result;
use uuid::Uuid;
use std::collections::HashMap;


pub struct Server {
    sockets: HashMap<Token, TcpStream>,
    connections: HashMap<Token, Connection>,
    sessions: HashMap<Token, Uuid>,
    rooms: HashMap<Uuid, Room>,
    boardset: BoardSet,
}


impl Server {

    pub fn new(boardset: BoardSet) -> Server {
        Server {
            sockets: HashMap::new(),
            connections: HashMap::new(),
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            boardset: boardset,
        }
    }

    pub fn new_conn(&mut self, token: Token, stream: TcpStream) {
        self.sockets.insert(token, stream);
    }

    pub fn process(&mut self, token: Token, poll: &mut Poll) -> Result<()> {
        if let Err(error) = self.handle(token) {
            if let Some(e) = error.downcast_ref::<connection::Error>() {
                if let connection::Error::Close(t) = e {
                    self.remove_conn(*t, poll)?;
                }
            } else {
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn handle(&mut self, token: Token) -> Result<()> {
        if let Some(socket) = self.sockets.remove(&token) {
            self.handle_socket(token, socket)?;
        } else if let Some(conn) = self.connections.remove(&token) {
            self.handle_connection(token, conn)?;
        } else if let Some(id) = self.sessions.get(&token) {
            if let Some(room) = self.rooms.get_mut(id) {
                room.handle(token)?;
            }
        }
        Ok(())
    }

    fn handle_socket(&mut self, token: Token, socket: TcpStream) -> Result<()> {
        let conn = Connection::accept(token, socket)?;
        self.connections.insert(token, conn);
        Ok(())
    }

    fn handle_connection(&mut self, token: Token, mut conn: Connection) -> Result<()> {
        match conn.read() {
            Ok(Some(request)) => self.handle_request(token, conn, request)?,
            Ok(None) => {},
            Err(e) => {
                let msg = format!("{}", e);
                let response = response::error(&msg);
                conn.send(response)?;
            }
        }
        Ok(())
    }

    pub fn handle_request(&mut self, token: Token, mut conn: Connection, req: Request) -> Result<()> {
        match &req {
            Request::Room(_) => {
                let board = self.boardset.new_board();

                let mut room = Room::new(board, token);
                let id = room.id;
                log::info!("{} - room created", room.id);

                room.add_conn(token, conn);
                self.sessions.insert(token, room.id);
                self.rooms.insert(room.id, room);

                if let Some(room) = self.rooms.get_mut(&id) {
                    room.room_response()?;
                    room.handle(token)?;
                }
            },
            Request::Join(j) => {
                if let Some(room) = self.rooms.get_mut(&j.id) {
                    log::info!("{} - new connection", room.id);
                    self.sessions.insert(token, room.id);
                    room.add_conn(token, conn);
                    room.handle(token)?;
                }
            },
            _ => {
                let response = response::error("expected room or join request");
                conn.send(response)?;
            }
        }
        Ok(())
    }

    pub fn remove_conn(&mut self, token: Token, poll: &mut Poll) -> Result<()> {
        if let Some(mut stream) = self.sockets.remove(&token) {

            log::debug!("removing socket {:?}", token);
            poll.registry().deregister(&mut stream)?;

        } else if let Some(mut conn) = self.connections.remove(&token) {

            log::debug!("removing ws {:?}", conn);
            poll.registry().deregister(conn.ws.get_mut())?;

        } else if let Some(id) = self.sessions.remove(&token) {
            if let Some(mut room) = self.remove_room(poll, id, token)? {
                log::info!("{} - room closed due to master leaving", room.id);
                for conn in room.connections.values_mut() {
                    log::debug!("removing conn {:?}", conn);
                    poll.registry().deregister(conn.ws.get_mut())?;
                }
            }
        }
        Ok(())
    }

    pub fn remove_room(&mut self, poll: &mut Poll, id: Uuid, token: Token) -> Result<Option<Room>> {
        let remove = if let Some(room) = self.rooms.get_mut(&id) {
            if let Some((player, mut conn)) = room.remove_conn(token) {
                poll.registry().deregister(conn.ws.get_mut())?;
                room.remove_response(player)?;
            }
            room.len() == 0 || token == room.game.admin
        } else {
            false
        };

        if remove {
            log::info!("{} - room closed due to being empty", id);
            Ok(self.rooms.remove(&id))
        } else {
            Ok(None)
        }
    }
}
