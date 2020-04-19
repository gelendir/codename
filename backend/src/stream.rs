use crate::idgenerator::IdGenerator;
use crate::request::Request;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use anyhow::{Result, Error};
use std::collections::HashMap;
use tungstenite::{WebSocket, Message, accept};
use tungstenite::Error as WsError;
use std::io;

const LISTENER: Token = Token(0);

pub struct Stream {
    sockets: HashMap<Token, TcpStream>,
    ws: HashMap<Token, WebSocket<TcpStream>>,
    generator: IdGenerator,
    poll: Poll,
    listener: TcpListener,
    events: Vec<Event>,
    responses: Vec<(Token, Message)>
}

#[derive(Debug)]
pub enum Event {
    Close(Token),
    Request(Token, Request),
    Error(Token, Error)
}

impl Stream {

    pub fn new(listener: TcpListener) -> Result<Stream> {
        let mut stream = Stream {
            listener: listener,
            sockets: HashMap::new(),
            ws: HashMap::new(),
            generator: IdGenerator::new(),
            poll: Poll::new()?,
            responses: Vec::new(),
            events: Vec::new(),
        };

        stream.init()?;
        Ok(stream)
    }

    fn init(&mut self) -> Result<()> {
        self.poll.registry()
            .register(&mut self.listener, LISTENER, Interest::READABLE)?;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Vec<Event>> {
        self.reregister()?;

        let mut events = Events::with_capacity(128);

        log::debug!("polling");
        self.poll.poll(&mut events, None)?;

        for event in events.iter() {
            let token = event.token();
            log::debug!("event: {:?}", event);

            if token == LISTENER {
                self.register()?;
            } else if event.is_readable() {
                self.read(token)?;
            } else if event.is_writable() {
                self.write(token)?;
            }
        }

        Ok(self.events.drain(..).collect())
    }

    pub fn register(&mut self) -> Result<()> {
        let (mut sock, _) = self.listener.accept()?;
        let token = Token(self.generator.next());

        self.poll.registry().register(&mut sock, token, Interest::READABLE)?; 
        self.sockets.insert(token, sock);

        Ok(())
    }

    pub fn reregister(&mut self) -> Result<()> {
        for (token, _) in self.responses.iter() {
            if let Some(ws) = self.ws.get_mut(&token) {
                self.poll.registry().reregister(ws.get_mut(), *token, Interest::READABLE | Interest::WRITABLE)?;
            }
        }
        Ok(())
    }

    fn read(&mut self, token: Token) -> Result<()> {
        loop {
            match self.read_event(token) {
                Ok(None) => return Ok(()),
                Ok(Some(event)) => self.events.push(event),
                Err(error) => {
                    if let Some(_) = error.downcast_ref::<WsError>() {
                        self.remove(token);
                        self.events.push(Event::Close(token))
                    } else {
                        return Err(error)
                    }
                }
            }
        }
    }

    fn read_event(&mut self, token: Token) -> Result<Option<Event>> {
        if let Some(stream) = self.sockets.remove(&token) {
            let ws = accept(stream)?;
            self.ws.insert(token, ws);
            return Ok(None);
        }

        if let Some(ws) = self.ws.get_mut(&token) {
            match Self::read_request(ws) {
                Ok(Some(request)) => return Ok(Some(Event::Request(token, request))),
                Err(error) => return Ok(Some(Event::Error(token, error))),
                _ => {}
            }
        }

        Ok(None)
    }

    pub fn remove(&mut self, token: Token) {
        if let Some(mut s) = self.sockets.remove(&token) {
            self.poll.registry().deregister(&mut s).unwrap();
        } else if let Some(mut ws) = self.ws.remove(&token) {
            self.poll.registry().deregister(ws.get_mut()).unwrap();
        }
        self.generator.recycle(token.0);
    }

    fn read_request(ws: &mut WebSocket<TcpStream>) -> Result<Option<Request>> {
        match ws.read_message() {
            Ok(message) => match message {
                Message::Text(msg) => {
                    Ok(Some(Request::from_str(&msg)?))
                },
                Message::Close(_) => {
                    Err(anyhow::Error::new(WsError::ConnectionClosed))
                }
                _ => Ok(None)
            },
            Err(error) => match error {
                WsError::Io(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    Ok(None)
                },
                _ => Err(anyhow::Error::new(error))
            }
        }
    }

    pub fn push(&mut self, token: Token, response: Message) {
        log::debug!("push: {:?}", response);
        self.responses.push((token, response))
    }

    pub fn append(&mut self, mut responses: Vec<(Token, Message)>) {
        log::debug!("append: {:?}", responses);
        self.responses.append(&mut responses)
    }

    fn write(&mut self, token: Token) -> Result<()> {
        let (filtered, responses) = self.responses
            .drain(..)
            .partition(|(t, _)| *t == token);

        self.responses = responses;

        if let Some(ws) = self.ws.get_mut(&token) {
            for (_, response) in filtered {
                if let Err(_) = ws.write_message(response) {
                    self.events.push(Event::Close(token))
                }
            }
            self.poll.registry().reregister(ws.get_mut(), token, Interest::READABLE)?;
        }

        Ok(())
    }

}
