use crate::idgenerator::IdGenerator;
use crate::request::Request;
use crate::error::RequestError;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
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
    Request(Token, Request),
    Error(Token, RequestError),
    Close(Token),
}

impl Stream {

    pub fn new(listener: TcpListener) -> io::Result<Stream> {
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

    fn init(&mut self) -> io::Result<()> {
        self.poll.registry()
            .register(&mut self.listener, LISTENER, Interest::READABLE)?;
        Ok(())
    }

    pub fn poll(&mut self) -> io::Result<Vec<Event>> {
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
                self.read(token);
            } else if event.is_writable() {
                self.write(token)?;
            }
        }

        Ok(self.events.drain(..).collect())
    }

    pub fn register(&mut self) -> io::Result<()> {
        let (mut sock, _) = self.listener.accept()?;
        let token = Token(self.generator.next());

        self.poll.registry().register(&mut sock, token, Interest::READABLE)?; 
        self.sockets.insert(token, sock);

        Ok(())
    }

    pub fn reregister(&mut self) -> io::Result<()> {
        for (token, _) in self.responses.iter() {
            if let Some(ws) = self.ws.get_mut(&token) {
                self.poll.registry().reregister(ws.get_mut(), *token, Interest::READABLE | Interest::WRITABLE)?;
            }
        }
        Ok(())
    }

    fn read(&mut self, token: Token) {
        loop {
            self.read_event(token)
        }
    }

    fn read_event(&mut self, token: Token) {
        if let Some(stream) = self.sockets.remove(&token) {
            if let Ok(ws) = accept(stream) {
                self.ws.insert(token, ws);
            } else {
                self.remove(token)
            }
        }

        if let Some(ws) = self.ws.get_mut(&token) {
            match ws.read_message() {
                Ok(message) => match message {
                    Message::Text(msg) => {
                        match Request::from_str(&msg) {
                            Ok(request) => self.events.push(Event::Request(token, request)),
                            Err(error) => self.events.push(Event::Error(token, error))
                        }
                    },
                    Message::Close(_) => self.events.push(Event::Close(token)),
                    _ => {}
                },
                Err(error) => match error {
                    WsError::Io(e) if e.kind() != io::ErrorKind::WouldBlock => {
                        self.events.push(Event::Close(token))
                    },
                    _ => {
                        self.events.push(Event::Close(token))
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, token: Token) {
        if let Some(mut s) = self.sockets.remove(&token) {
            self.poll.registry().deregister(&mut s).unwrap();
        } else if let Some(mut ws) = self.ws.remove(&token) {
            self.poll.registry().deregister(ws.get_mut()).unwrap();
        }
        self.generator.recycle(token.0);
    }

    pub fn push(&mut self, token: Token, response: Message) {
        log::debug!("push: {:?}", response);
        self.responses.push((token, response))
    }

    pub fn append(&mut self, mut responses: Vec<(Token, Message)>) {
        log::debug!("append: {:?}", responses);
        self.responses.append(&mut responses)
    }

    fn write(&mut self, token: Token) -> io::Result<()> {
        let (filtered, responses) = self.responses
            .drain(..)
            .partition(|(t, _)| *t == token);

        self.responses = responses;

        if let Some(ws) = self.ws.get_mut(&token) {
            for (_, response) in filtered {
                if let Err(_) = ws.write_message(response) {
                    self.events.push(Event::Close(token));
                }
            }
            self.poll.registry().reregister(ws.get_mut(), token, Interest::READABLE)?;
        }

        Ok(())
    }

}
