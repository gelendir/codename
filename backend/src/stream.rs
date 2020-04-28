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
pub struct Event {
    pub token: Token,
    pub kind: EventKind,
}

#[derive(Debug)]
pub enum EventKind {
    Request( Request),
    Error(RequestError),
    Close
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
                log::debug!("reregister write {}", token.0);
                self.poll.registry().reregister(ws.get_mut(), *token, Interest::READABLE | Interest::WRITABLE)?;
            }
        }
        Ok(())
    }

    fn read(&mut self, token: Token) {
        if let Some(stream) = self.sockets.remove(&token) {
            log::debug!("accepting socket {}", token.0);
            match accept(stream) {
                Ok(ws) => {
                     self.ws.insert(token, ws);
                },
                Err(error) => {
                    log::error!("accept error: {}", error);
                    self.remove(token);
                }
            }
            return
        }

        if let Some(ws) = self.ws.get_mut(&token) {
            loop {
                log::debug!("reading request on {}", token.0);
                match ws.read_message() {
                    Ok(message) => match message {
                        Message::Text(msg) => {
                            match Request::from_str(&msg) {
                                Ok(request) => self.events.push(Event{
                                    token: token,
                                    kind: EventKind::Request(request)
                                }),
                                Err(error) => self.events.push(Event{
                                    token: token,
                                    kind: EventKind::Error(error)
                                }),
                            }
                        },
                        Message::Close(_) => {
                            self.events.push(Event{
                                token: token,
                                kind: EventKind::Close
                            });
                            return
                        }
                        _ => {}
                    },
                    Err(error) => match error {
                        WsError::Io(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            log::debug!("finished reading requests on {}", token.0);
                            return
                        },
                        _ => {
                            log::error!("read error: {}", error);
                            self.events.push(Event{
                                token: token,
                                kind: EventKind::Close
                            });
                            return
                        }
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, token: Token) {
        if let Some(mut s) = self.sockets.remove(&token) {
            log::debug!("removing socket {}", token.0);
            self.poll.registry().deregister(&mut s).unwrap();
        } else if let Some(mut ws) = self.ws.remove(&token) {
            log::debug!("removing websocket {}", token.0);
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
                if let Err(error) = ws.write_message(response) {
                    log::error!("write error: {}", error);
                    self.events.push(Event{
                        token: token,
                        kind: EventKind::Close
                    });
                }
            }
            self.poll.registry().reregister(ws.get_mut(), token, Interest::READABLE)?;
        }

        Ok(())
    }

}
