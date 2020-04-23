use crate::board::Board;
use crate::request;
use crate::response;
use crate::game::{Game, State};
use crate::error::{GameError};
use mio::Token;
use uuid::Uuid;
use tungstenite::Message;


pub type Responses = Vec<(Token, Message)>;

#[derive(Debug)]
pub struct Room {
    pub id: Uuid,
    pub game: Game
}

impl Room {

    pub fn new(id: Uuid, board: Board, admin: Token) -> Room {
        Room {
            id: id,
            game: Game::new(board, admin),
        }
    }

    pub fn is_alive(&self, token: Token) -> bool {
        token != self.game.admin && self.game.nb_players() > 0
    }

    pub fn remove_player(&mut self, token: Token) -> Responses {
        let mut responses = Vec::new();

        if let Some(name) = self.game.remove_player(token) {
            let mut r = self.broadcast(response::leave(&self.game, &name));
            responses.append(&mut r);

            match self.game.state {
                State::Start => {
                    let mut r = self.broadcast(response::restart(&self.game));
                    responses.append(&mut r);
                },
                _ => {}
            }
        }
        responses
    }

    pub fn broadcast_room(&self) -> Responses {
        vec![(self.game.admin, response::room(self))]
    }

    pub fn handle(&mut self, token: Token, request: &request::Request) -> Responses {
        let result = match request {
            request::Request::Team(t) => self.join_team(token, t),
            request::Request::Start(s) => self.start(token, s),
            request::Request::Hint(h) => self.hint(token, h),
            request::Request::Guess(g) => self.guess(token, g),
            request::Request::Pass(_) => self.pass(token),
            _ => {
                return vec![(token, response::error("unhandled request"))]
            }
        };

        match result {
            Ok(r) => r,
            Err(error) => {
                vec![(token, response::error(&error.to_string()))]
            }
        }
    }

    fn broadcast(&mut self, response: Message) -> Responses {
        self.game.tokens()
            .map(|t| {
                log::debug!("broadcasting to {}", t.0);
                (*t, response.clone())
            })
            .collect()
    }

    fn join_team(&mut self, token: Token, team: &request::Team) -> Result<Responses, GameError> {
        log::info!("{} - player {} joined team {:?}", self.id, team.name, team.team);

        self.game.add_player(token, team.team, &team.name);

        let response = response::join(&self.game, &team.name);
        Ok(self.broadcast(response))
    }

    fn start(&mut self, token: Token, start: &request::Start) -> Result<Responses, GameError> {
        self.game.start(token, start)?;
        log::info!("{} - game started", self.id);

        let mut responses = Vec::new();
        let response = response::tiles(&self.game);

        if let Some(master) = self.game.red.master {
            responses.push((master, response.clone()))
        }

        if let Some(master) = self.game.blue.master {
            responses.push((master, response.clone()))
        }

        for item in self.broadcast(response::turn(&self.game)) {
            responses.push(item);
        }
        Ok(responses)
    }

    fn hint(&mut self, token: Token, hint: &request::Hint) -> Result<Responses, GameError> {
        self.game.hint(token, &hint)?;
        log::info!("{} - hint word: {} guesses: {}", self.id, hint.hint, hint.guesses);

        Ok(self.broadcast(
            response::hint(&self.game)
        ))
    }

    fn guess(&mut self, token: Token, guess: &request::Guess) -> Result<Responses, GameError> {
        self.game.guess(token, &guess)?;
        log::info!("{} - guess {} {}", self.id, guess.x, guess.y);

        match self.game.state {
            State::End(winner) => {
                log::info!("{} - game ended. winner: {:?}", self.id, winner);
                 Ok(self.broadcast(response::end(&self.game)))
            },
            _ => Ok(self.broadcast(response::turn(&self.game)))
        }
    }

    fn pass(&mut self, token: Token) -> Result<Responses, GameError> {
        self.game.pass(token)?;
        log::info!("{} - pass", self.id);

        Ok(self.broadcast(
            response::turn(&self.game)
        ))
    }

}
