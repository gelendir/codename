use serde::Serialize as SerdeSerialize;
use serde::Deserialize as SerdeDeserialize;
use serde::ser::{Serialize, Serializer, SerializeStruct};
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use rand::prelude::*;
use crate::team::Team;
use crate::error::BoardError;
use std::collections::HashMap;


pub type WordMap = [ [String; 5]; 5];
pub type TileMap = [ [Tile; 5]; 5];
pub type CardMap = [ [bool; 5]; 5];
pub type Dictionnary = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, SerdeSerialize, SerdeDeserialize, PartialEq)]
#[serde(rename_all="lowercase")]
pub enum Tile {
    Neutral,
    Blue,
    Red,
    Death
}

#[derive(Debug, SerdeSerialize, SerdeDeserialize)]
pub struct BoardSet {
    words: Dictionnary,
    tiles: Vec<TileMap>,
}

#[derive(Debug)]
pub struct Board {
    pub words: WordMap,
    pub cards: CardMap,
    pub tiles: TileMap,
}

impl Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut cards = Vec::new();
        for (x, r) in self.cards.iter().enumerate() {
            let mut row = Vec::new();
            for (y, card) in r.iter().enumerate() {
                if *card {
                    row.push(Some(&self.tiles[x][y]));
                } else {
                    row.push(None)
                }
            }
            cards.push(row);
        }

        let mut s = serializer.serialize_struct("Board", 2)?;
        s.serialize_field("words", &self.words)?;
        s.serialize_field("cards", &cards)?;
        s.end()
    }
}

pub fn load_board_file(path: &str) -> Result<BoardSet, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let boardset: BoardSet = serde_json::from_str(&contents)?;
    Ok(boardset)
}

impl BoardSet {

    pub fn new_board(&self, language: &str) -> Result<Board, BoardError> {
        let mut rng = rand::thread_rng();

        let words = self.words
            .get(language)
            .ok_or(BoardError::Language(language.to_string()))?;

        let mut words: Vec<&String> = words.iter().collect();

        words.shuffle(&mut rng);

        let wordmap = [
            [words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone()],
            [words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone()],
            [words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone()],
            [words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone()],
            [words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone(), words.pop().unwrap().clone()],
        ];

        let tilemap = self.tiles.choose(&mut rng)
            .expect("rng unable to pick tile")
            .clone();

        let cardmap = [
            [false; 5],
            [false; 5],
            [false; 5],
            [false; 5],
            [false; 5],
        ];

        Ok(Board{
            words: wordmap,
            tiles: tilemap,
            cards: cardmap,
        })
    }

}

impl Board {

    pub fn start_team(&self) -> Team {
        let red_tiles: usize = self.tiles.iter()
            .map(|row| {
                row.iter()
                    .filter(|t| *t == &Tile::Red)
                    .count()
            })
            .sum();

        let blue_tiles: usize = self.tiles.iter()
            .map(|row| {
                row.iter().
                    filter(|t| *t == &Tile::Blue)
                    .count()
            })
            .sum();

        if red_tiles > blue_tiles {
            Team::Red
        } else {
            Team::Blue
        }
    }

    pub fn put_card(&mut self, x: usize, y: usize) -> Tile {
        self.cards[x][y] = true;
        self.tiles[x][y].clone()
    }

    pub fn winner(&self) -> Option<Team> {
        let (blue_tiles, blue_cards) = self.count_cards(&Team::Blue);
        if blue_tiles == blue_cards {
            return Some(Team::Blue);
        }

        let (red_tiles, red_cards) = self.count_cards(&Team::Red);
        if red_tiles == red_cards {
            return Some(Team::Red);
        }
        None
    }

    fn count_cards(&self, team: &Team) -> (u8, u8) {
        let mut tiles = 0;
        let mut cards = 0;

        let search = match team {
            Team::Red => Tile::Red,
            Team::Blue => Tile::Blue
        };

        for (x, row) in self.tiles.iter().enumerate() {
            for (y, tile) in row.iter().enumerate() {
                if *tile == search {
                    tiles += 1;
                    if self.cards[x][y] {
                        cards += 1;
                    }
                }
            }
        }

        return (tiles, cards)
    }

}
