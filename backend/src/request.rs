use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;
use crate::team::Team as TeamColor;
use crate::error::RequestError;

#[derive(Deserialize, Debug)]
pub struct Room {
    pub name: String,
    pub language: String,
}

#[derive(Deserialize, Debug)]
pub struct Reset {
    pub language: String,
}

#[derive(Deserialize, Debug)]
pub struct Join {
    pub id: Uuid,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Team {
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
    Reset(Reset),
}

impl Request {

    pub fn from_str(text: &str) -> Result<Request, RequestError> {
        log::debug!("request parse: {}", text);
        let data: Value = serde_json::from_str(text)?;

        let result = data.get("request").ok_or(RequestError::Missing("request"))?;

        if let Value::String(request) = result {
            match request.as_str() {
                "reset" => Reset::parse(data),
                "room" => Room::parse(data),
                "join" => Join::parse(data),
                "team" => Team::parse(data),
                "start" => Start::parse(data),
                "hint" => Hint::parse(data),
                "guess" => Guess::parse(data),
                "pass" => Pass::parse(data),
                _ => Err(RequestError::Unknown(request.clone()))
            }
        } else {
            Err(RequestError::Missing("request"))
        }
    }
}

impl Reset {

    pub fn parse(data: Value) -> Result<Request, RequestError> {
        let reset: Reset = serde_json::from_value(data)?;
        Ok(Request::Reset(reset))
    }

}

impl Room {

    pub fn parse(data: Value) -> Result<Request, RequestError> {
        let room: Room = serde_json::from_value(data)?;
        Ok(Request::Room(room))
    }

}

impl Join {

    pub fn parse(data: Value) -> Result<Request, RequestError> {
        let join: Join = serde_json::from_value(data)?;
        Ok(Request::Join(join))
    }

}


impl Team {

    pub fn parse(data: Value) -> Result<Request, RequestError> {
        let team: Team = serde_json::from_value(data)?;
        Ok(Request::Team(team))
    }

}

impl Start {

    pub fn parse(data: Value) -> Result<Request, RequestError> {
        let start: Start = serde_json::from_value(data)?;

        if start.blue == "" {
            return Err(RequestError::Missing("blue"));
        }
        if start.red == "" {
            return Err(RequestError::Missing("red"));
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

    pub fn parse(data: Value) -> Result<Request, RequestError> {
        let hint: Hint = serde_json::from_value(data)?;

        if hint.hint == "" {
            return Err(RequestError::Missing("hint"));
        }

        if !(hint.guesses >= 1 && hint.guesses <= 9) {
            return Err(RequestError::Invalid("guesses must be between 1 and 9"));
        }

        Ok(Request::Hint(hint))
    }

}

impl Guess {

    pub fn parse(data: Value) -> Result<Request, RequestError> {
        let guess: Guess = serde_json::from_value(data)?;

        if guess.x > 4 {
            return Err(RequestError::Invalid("x must be between 0 and 4"));
        }

        if guess.y > 4 {
            return Err(RequestError::Invalid("y must be between 0 and 4"));
        }

        Ok(Request::Guess(guess))
    }

}

impl Pass {

    pub fn parse(data: Value) -> Result<Request, RequestError> {
        Ok(Request::Pass(serde_json::from_value(data)?))
    }

}
