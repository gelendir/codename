use serde_json::Value;
use serde_json::json;
use crate::game::Game;
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
        "room": room,
    }))
}

pub fn tiles(game: &Game) -> Message {
    convert(json!({
        "response": "tiles",
        "tiles": game.board.tiles
    }))
}
