use crate::request;
use mio::net::TcpStream;
use tungstenite::protocol::WebSocket;
use uuid::Uuid;
use crate::response;
use crate::color::Color;
use anyhow::Result;

pub struct Session {
    ws: WebSocket<TcpStream>,
    pub name: String,
    pub color: Color,
    pub game: Uuid,
}

impl Session {

    pub fn game(ws: WebSocket<TcpStream>, request: request::Game, game: Uuid) -> Session {
        Session {
            ws: ws,
            name: request.name,
            color: request.color,
            game: game,
        }
    }

    pub fn join(ws: WebSocket<TcpStream>, request: request::Join, game: Uuid) -> Session {
        Session {
            ws: ws,
            name: request.name,
            color: request.color,
            game: game,
        }
    }

    pub fn send(&mut self, response: response::Response) -> Result<()> {
        self.ws.write_message(response.as_message())?;
        Ok(())
    }
}

