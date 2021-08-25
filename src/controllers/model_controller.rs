use std::{
    collections::HashMap,
    fs,
};

use cursive::Cursive;
use getset::Setters;
use uuid::Uuid;

use crate::game::{
    Deck,
    Event,
    Game,
    GameError,
    GameResult,
    History,
    RowIndex,
};

type OnEventListener = Box<dyn Fn(&mut Cursive, &Event)>;
type OnErrorListener = Box<dyn Fn(&GameError)>;
type OnNewGameListener = Box<dyn Fn(&mut Cursive)>;
type OnCheckDealPossible = Box<dyn Fn(&mut Cursive, bool)>;
type OnCheckPlacePossible = Box<dyn Fn(&mut Cursive, bool)>;
type OnDeckSizeChangedListener = Box<dyn Fn(&mut Cursive, usize)>;
type OnGameOverListener = Box<dyn Fn(&mut Cursive, GameResult)>;

#[derive(Setters)]
pub struct ModelController
{
    game: Option<Game>,

    #[getset(set = "pub")]
    on_event: Option<OnEventListener>,

    #[getset(set = "pub")]
    on_error: Option<OnErrorListener>,

    #[getset(set = "pub")]
    on_new_game: Option<OnNewGameListener>,

    #[getset(set = "pub")]
    on_deck_size_changed: Option<OnDeckSizeChangedListener>,

    #[getset(set = "pub")]
    on_check_deal_possible: Option<OnCheckDealPossible>,

    #[getset(set = "pub")]
    on_check_place_possible: Option<OnCheckPlacePossible>,

    #[getset(set = "pub")]
    on_game_over: Option<OnGameOverListener>,
}

impl ModelController
{
    pub fn new() -> ModelController
    {
        ModelController {
            game: None,
            on_event: None,
            on_error: None,
            on_new_game: None,
            on_deck_size_changed: None,
            on_check_deal_possible: None,
            on_check_place_possible: None,
            on_game_over: None,
        }
    }

    pub fn drop_game(&mut self)
    {
        self.game.take().map(Game::end).map(save_history);
    }

    pub fn new_game(&mut self, s: &mut Cursive)
    {
        self.drop_game();
        self.game = Some(Game::new(Deck::shuffled()));

        self.on_new_game.as_ref().map(|listener| listener(s));
        self.call_on_deck_size_changed(s);
        self.call_on_check_deal_possible(s);
        self.call_on_check_place_possible(s);
    }

    pub fn deal(&mut self, s: &mut Cursive)
    {
        if let Some(result) = self.game.as_mut().map(Game::deal) {
            match result {
                Ok(event) => {
                    self.on_event.as_ref().map(|listener| listener(s, event));
                    self.call_on_deck_size_changed(s);
                    self.call_on_check_deal_possible(s);
                    self.call_on_check_place_possible(s);
                    self.call_on_game_over_if_game_is_over(s);
                }
                Err(error) => {
                    self.on_error.as_ref().map(|listener| listener(&error));
                }
            };
        }
    }

    pub fn eliminate(&mut self, s: &mut Cursive, index: RowIndex)
    {
        if let Some(result) = self.game.as_mut().map(|game| game.eliminate(index)) {
            match result {
                Ok(event) => {
                    self.on_event.as_ref().map(|listener| listener(s, event));
                    self.call_on_check_deal_possible(s);
                    self.call_on_check_place_possible(s);
                    self.call_on_game_over_if_game_is_over(s);
                }
                Err(error) => {
                    self.on_error.as_ref().map(|listener| listener(&error));
                }
            };
        }
    }

    pub fn place(&mut self, s: &mut Cursive, from: RowIndex, to: RowIndex)
    {
        if let Some(result) = self.game.as_mut().map(|game| game.place(from, to)) {
            match result {
                Ok(event) => {
                    self.on_event.as_ref().map(|listener| listener(s, event));
                    self.call_on_check_deal_possible(s);
                    self.call_on_check_place_possible(s);
                    self.call_on_game_over_if_game_is_over(s);
                }
                Err(error) => {
                    self.on_error.as_ref().map(|listener| listener(&error));
                }
            };
        }
    }

    fn call_on_deck_size_changed(&self, s: &mut Cursive)
    {
        self.on_deck_size_changed.as_ref().map(|listener| {
            self.game
                .as_ref()
                .map(Game::deck)
                .map(Deck::len)
                .map(|len| listener(s, len));
        });
    }

    fn call_on_check_deal_possible(&self, s: &mut Cursive)
    {
        self.on_check_deal_possible.as_ref().map(|listener| {
            self.game
                .as_ref()
                .map(Game::check_deal)
                .map(|result| listener(s, result.is_ok()));
        });
    }

    fn call_on_check_place_possible(&self, s: &mut Cursive)
    {
        self.on_check_place_possible.as_ref().map(|listener| {
            self.game.as_ref().map(Game::table).map(|table| {
                listener(
                    s,
                    table.iter().map(Vec::len).any(|len| len > 1)
                        && table.iter().map(Vec::is_empty).any(|x| x),
                );
            });
        });
    }

    fn call_on_game_over_if_game_is_over(&self, s: &mut Cursive)
    {
        self.on_game_over.as_ref().map(|listener| {
            self.game
                .as_ref()
                .map(Game::game_result)
                .flatten()
                .map(|result| listener(s, result))
        });
    }
}

fn save_history(history: History) -> Result<(), std::io::Error>
{
    use std::io::ErrorKind::NotFound;

    let history_path = "history.json";

    match fs::read_to_string(history_path) {
        Err(error) => {
            if matches!(error.kind(), NotFound) {
                fs::write(history_path, "{}")?;
                save_history(history)
            } else {
                Err(error)
            }
        }
        Ok(data) => {
            let mut archive: HashMap<Uuid, History> = serde_json::from_str(&data).unwrap();
            let uuid = Uuid::new_v4();
            archive.insert(uuid, history);
            fs::write(history_path, serde_json::to_string(&archive).unwrap())
        }
    }
}
