mod request;
mod response;
mod game; 
mod gameteam;
mod server;
mod team;
mod board;
mod room;
mod idgenerator;
mod stream;
mod error;

extern crate log;

use mio::net::TcpListener;
use std::error::Error;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let boardset = board::load_board_file(&args[1])?;


    let addr = "0.0.0.0:8080".parse()?;
    let listener = TcpListener::bind(addr)?;

    let stream = stream::Stream::new(listener)?;
    let mut server = server::Server::new(boardset, stream);

    if let Err(e) = server.run() {
        log::error!("server error: {}", e);
    }

    Ok(())
}
