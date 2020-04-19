use mio::net::TcpStream;
use mio::Token;
use tungstenite::protocol::WebSocket;
use tungstenite::server::accept;
use tungstenite::Message as WsMessage;
use tungstenite::error::Error as WsError;
use crate::request::Request;
use anyhow;
use uuid::Uuid;
use std::io;

pub enum Stream {
    Tcp(TcpStream),
    Ws(WebSocket<TcpStream>)
}

pub struct Client {
    pub token: Token,
    stream: Stream,
    id: Option<Uuid>,
    responses: Vec<WsMessage>,
}

#[derive(Debug)]
pub struct Requests {
    pub id: Uuid,
    pub token: Token,
    pub requests: Vec<Request>
}

impl Client {

    pub fn new(token: Token, stream: TcpStream) -> Client {
        Client {
            token: token,
            stream: Stream::Tcp(stream),
            id: None,
            responses: Vec::new(),
        }
    }

    pub fn read(&mut self) -> anyhow::Result<Option<Requests>> {
        match &mut self.stream {
            Stream::Tcp(s) => {
                self.stream = Stream::Ws(accept(s)?);
                Ok(None)
            },
            Stream::Ws(mut ws) => {
                if let Some(id) = self.id {
                    let requests = Self::read_requests(&mut ws)?;
                    Ok(Some(Requests{
                        id: id.clone(),
                        token: self.token,
                        requests: requests
                    }))
                } else {
                    self.id = Some(Self::accept_websocket(&mut ws)?);
                    Ok(None)
                }
            }
        }
    }

    fn accept_socket(&self, stream: TcpStream) -> anyhow::Result<WebSocket<TcpStream>> {
        let ws = accept(stream)?;
        Ok(ws)
    }

    fn accept_websocket(ws: &mut WebSocket<TcpStream>) -> anyhow::Result<Uuid> {
        match Self::read_request(ws)? {
            Some(request) => match request {
                Request::Room(_) => {
                    let id = Uuid::new_v4();
                    Ok(id)
                },
                Request::Join(j) => {
                    Ok(j.id)
                },
                _ => {
                    Err(anyhow::Error::new(WsError::ConnectionClosed))
                }
            },
            None => Err(anyhow::Error::new(WsError::ConnectionClosed))
        }
    }

    fn read_requests(ws: &mut WebSocket<TcpStream>) -> anyhow::Result<Vec<Request>> {
        let mut requests = Vec::new();
        while let Some(request) = Self::read_request(ws)? {
            requests.push(request);
        }
        Ok(requests)
    }

    pub fn push(&mut self, response: WsMessage) {
        self.responses.push(response);
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        match &mut self.stream {
            Stream::Tcp(_) => {},
            Stream::Ws(ws) => {
                for message in self.responses.drain(..) {
                    ws.write_message(message)?;
                }
            }
        }
        Ok(())
    }

    pub fn mut_socket(&mut self) -> &mut TcpStream {
        match &mut self.stream {
            Stream::Tcp(s) => s,
            Stream::Ws(ws) => ws.get_mut()
        }
    }

}
