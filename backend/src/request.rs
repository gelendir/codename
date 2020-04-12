use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;
use crate::team::Team as TeamColor;

#[derive(Deserialize, Debug)]
pub struct Room {
}

#[derive(Deserialize, Debug)]
pub struct Join {
    pub id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct Team {
    pub name: String,
    pub team: TeamColor,
}

#[derive(Deserialize, Debug)]
pub struct Start {
    pub blue: String,
    pub red: String
}

#[derive(Deserialize, Debug)]
pub struct Hint {
    pub hint: String,
    pub guesses: u8,
}

#[derive(Deserialize, Debug)]
pub struct Guess {
    pub x: usize,
    pub y: usize
}

#[derive(Deserialize, Debug)]
pub struct Pass {
}

#[derive(Debug)]
pub enum Request {
    Room(Room),
    Join(Join),
    Team(Team),
    Start(Start),
    Hint(Hint),
    Guess(Guess),
    Pass(Pass),
}

impl Request {

    pub fn from_str(text: &str) -> Result<Request> {
        log::debug!("request parse: {}", text);
        let data: Value = serde_json::from_str(text)?;

        let request = match data.get("request") {
            Some(Value::String(s)) => s,
            _ => return Err(anyhow!("field request missing"))
        };

        match request.as_str() {
            "room" => Room::parse(data),
            "join" => Join::parse(data),
            "team" => Team::parse(data),
            "start" => Start::parse(data),
            "hint" => Hint::parse(data),
            "guess" => Guess::parse(data),
            "pass" => Pass::parse(data),
            e => Err(anyhow!("unknown request: {}", e))
        }
    }

}

impl Room {

    pub fn parse(data: Value) -> Result<Request> {
        let room: Room = serde_json::from_value(data)?;
        Ok(Request::Room(room))
    }

}

impl Join {

    pub fn parse(data: Value) -> Result<Request> {
        let join: Join = serde_json::from_value(data)?;
        Ok(Request::Join(join))
    }

}


impl Team {

    pub fn parse(data: Value) -> Result<Request> {
        let team: Team = serde_json::from_value(data)?;
        if team.name == "" {
            return Err(anyhow!("name empty"));
        }
        Ok(Request::Team(team))
    }

}

impl Start {

    pub fn parse(data: Value) -> Result<Request> {
        let start: Start = serde_json::from_value(data)?;

        if start.blue == "" {
            return Err(anyhow!("blue empty"));
        }
        if start.red == "" {
            return Err(anyhow!("red empty"));
        }

        Ok(Request::Start(start))
    }

    pub fn master(&self, team: &TeamColor) -> &str {
        match team {
            TeamColor::Red => &self.red,
            TeamColor::Blue => &self.blue,
        }
    }

}

impl Hint {

    pub fn parse(data: Value) -> Result<Request> {
        let hint: Hint = serde_json::from_value(data)?;

        if hint.hint == "" {
            return Err(anyhow!("hint empty"));
        }

        if !(hint.guesses >= 1 && hint.guesses <= 9) {
            return Err(anyhow!("guesses must be between 1 and 9"));
        }

        Ok(Request::Hint(hint))
    }

}

impl Guess {

    pub fn parse(data: Value) -> Result<Request> {
        let guess: Guess = serde_json::from_value(data)?;

        if guess.x > 4 {
            return Err(anyhow!("x must be between 0 and 4"));
        }

        if guess.x > 4 {
            return Err(anyhow!("x must be between 0 and 4"));
        }

        Ok(Request::Guess(guess))
    }

}

impl Pass {

    pub fn parse(data: Value) -> Result<Request> {
        Ok(Request::Pass(serde_json::from_value(data)?))
    }

}
