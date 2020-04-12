use serde_json::Value;
use serde_json::json;
use crate::game::Game;
use crate::game::State;
use crate::room::Room;
use tungstenite::Message;


fn convert(data: Value) -> Message {
    Message::Text(data.to_string())
}

pub fn error(msg: &str) -> Message {
    convert(json!({
        "response": "error",
        "error": msg,
    }))
}

pub fn room(room: &Room) -> Message {
    convert(json!({
        "response": "room",
        "id": room.id
    }))
}

pub fn join(game: &Game, name: &str) -> Message {
    convert(json!({
        "response": "join",
        "player": name,
        "game": game
    }))
}

pub fn leave(game: &Game, name: &str) -> Message {
    convert(json!({
        "response": "leave",
        "player": name,
        "game": game
    }))
}

pub fn tiles(game: &Game) -> Message {
    convert(json!({
        "response": "tiles",
        "tiles": game.board.tiles
    }))
}

pub fn turn(game: &Game) -> Message {
    convert(json!({
        "response": "turn",
        "game": game
    }))
}

pub fn end(game: &Game) -> Message {
    let winner = match game.state {
        State::End(winner) => winner,
        _ => panic!("no winner in end response")
    };

    convert(json!({
        "response": "end",
        "winner": winner,
        "game": game
    }))
}

pub fn restart(game: &Game) -> Message {
    convert(json!({
        "response": "restart",
        "game": game
    }))
}

pub fn hint(game: &Game) -> Message {
    convert(json!({
        "response": "hint",
        "game": game
    }))
}
