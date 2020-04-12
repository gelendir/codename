use mio::net::TcpStream;
use mio::Token;
use tungstenite::protocol::WebSocket;
use tungstenite::server::accept;
use tungstenite::Message as WsMessage;
use tungstenite::error::Error as WsError;
use crate::request::Request;
use std::io;
use std::error;
use std::fmt;
use anyhow;

#[derive(Debug)]
pub struct Connection {
    pub token: Token,
    pub ws: WebSocket<TcpStream>,
}

#[derive(Debug)]
pub enum Error {
    Close(Token),
    ParseError(anyhow::Error)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Close(t) => write!(f, "Connection {} closed", t.0),
            Error::ParseError(e) => write!(f, "Request parse error : {}", e)
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl Connection {

    pub fn accept(token: Token, stream: TcpStream) -> anyhow::Result<Connection> {
        let ws = accept(stream)?;
        log::debug!("accept: {} {:?}", token.0, ws);

        Ok(Connection{
            token: token,
            ws: ws,
        })
    }

    pub fn read(&mut self) -> Result<Option<Request>, Error> {
        match self.ws.read_message() {
            Ok(result) => {
                match result {
                    WsMessage::Text(msg) => {
                        match Request::from_str(&msg) {
                            Ok(request) => Ok(Some(request)),
                            Err(e) => Err(Error::ParseError(e))
                        }
                    },
                    WsMessage::Close(_) => {
                        Err(Error::Close(self.token))
                    }
                    _ => Ok(None)
                }
            },
            Err(error) => {
                match error {
                    WsError::Io(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        Ok(None)
                    },
                    e => {
                        log::error!("websocket error: {}", e);
                        Err(Error::Close(self.token))
                    }
                }
            }
        }
    }

    pub fn send(&mut self, response: WsMessage) -> anyhow::Result<()> {
        log::debug!("response: {} {}", self.token.0, response);
        self.ws.write_message(response)?;
        Ok(())
    }

}
