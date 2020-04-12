mod request;
mod response;
mod game; 
mod gameteam;
mod server;
mod team;
mod connection;
mod board;
mod room;

extern crate log;

use anyhow::Result;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::error::Error;
use std::env;

const LISTENER: Token = Token(0);

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let boardset = board::load_board_file(&args[1])?;

    let mut server = server::Server::new(boardset);
    let mut next_index = 1;

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);

    let addr = "0.0.0.0:8080".parse()?;
    let mut listener = TcpListener::bind(addr)?;
    
    poll.registry()
        .register(&mut listener, LISTENER, Interest::READABLE)?;

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {

            let token = event.token();
            log::debug!("event: {:?}", event);

            if token == LISTENER {
                let (mut sock, _) = listener.accept()?;

                let token = Token(next_index);
                next_index += 1;

                poll.registry().register(&mut sock, token, Interest::READABLE)?; 
                server.new_conn(token, sock);
            } else {
                if let Err(e) = server.process(token, &mut poll) {
                    log::error!("error: {}", e);
                }
            }
        }
    }
}
