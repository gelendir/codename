use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use rand::prelude::*;
use anyhow::Result;
use anyhow::anyhow;


type WordMap = Vec<Vec<String>>;
type TileMap = Vec<Vec<Tile>>;
type CharMap = Vec<Vec<char>>;

#[derive(Debug)]
pub enum Tile {
    Neutral,
    Blue,
    Red,
    Death
}


#[derive(Deserialize, Debug)]
pub struct BoardFile {
    words: Vec<String>,
    tiles: Vec<CharMap>,
}

pub struct BoardSet {
    words: Vec<String>,
    tiles: Vec<TileMap>,
}

#[derive(Debug)]
pub struct Board {
    words: WordMap,
    tiles: TileMap,
}

pub fn load_board_file(path: &str) -> Result<BoardSet> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents);

    let boardfile: BoardFile = serde_json::from_str(&contents)?;
    let mut tilemaps: Vec<TileMap> = Vec::new();

    for tilemap in boardfile.tiles {

        let mut new_tilemap: TileMap = Vec::new();
        for row in tilemap {

            let mut new_row: Vec<Tile> = Vec::new();
            for c in row {
                let tile = match c {
                    'n' => Tile::Neutral,
                    'r' => Tile::Red,
                    'b' => Tile::Blue,
                    'd' => Tile::Death,
                    e => return Err(anyhow!("unexpected tile letter: {}", e))
                };
                new_row.push(tile);
            }

            new_tilemap.push(new_row);
        }
        tilemaps.push(new_tilemap);
    }

    Ok(BoardSet{
        words: boardfile.words,
        tiles: tilemaps,
    })
}

impl BoardSet {

    pub fn new_board(&self) -> Result<Board> {
        let mut rng = rand::thread_rng();

        let mut words: Vec<&String> = self.words.iter().collect();
        words.shuffle(&mut rng);

        let mut words = words.into_iter().take(25);

        let wordmap = (0..5)
            .map(|_| {
                words.take(5)
                    .map(|s| s.clone())
                    .collect()
            })
            .collect();

        let tilemap = self.tiles
            .choose(&mut rng)
            .expect("rng didn't produce a tilemap");

        Ok(Board{
            words: wordmap,
            tiles: *tilemap
        })
    }

}
