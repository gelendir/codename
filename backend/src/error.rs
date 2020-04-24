use thiserror::Error;
use serde_json::error::Error as SerdeError;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("parse error: {0}")]
    Parse(#[from] SerdeError),
    #[error("field missing: {0}")]
    Missing(&'static str),
    #[error("unknown request: {0}")]
    Unknown(String),
    #[error("invalid value: {0}")]
    Invalid(&'static str),
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("not your turn to give a {0}")]
    Turn(&'static str),
    #[error("player is not a master")]
    NotMaster,
    #[error("player is not the admin")]
    NotAdmin,
    #[error("team {0} does not have enough players")]
    MissingPlayers(&'static str),
    #[error("game has not started")]
    NotStarted,
    #[error("game has already started")]
    AlreadyStarted,
}

#[derive(Error, Debug)]
pub enum BoardError {
    #[error("language '{0}' not found")]
    Language(String)
}

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("board error: {0}")]
    Board(#[from] BoardError),
    #[error("game error: {0}")]
    Game(#[from] GameError),
}
