use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="lowercase")]
pub enum Team {
    Blue,
    Red
}


impl Team {

    pub fn opposite(&self) -> Team {
        match self {
            Team::Blue => Team::Red,
            Team::Red => Team::Blue
        }
    }

}
