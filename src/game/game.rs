use std::{
    collections::HashSet,
    error::Error,
    fmt::{
        self,
        Display,
    },
};

use getset::Getters;

use crate::game::{
    card::{
        Card,
        Deck,
        Rank,
    },
    Event,
    History,
};

pub type RowIndex = usize;

pub type Table = [Vec<Card>; 4];
pub type DiscardPile = Vec<Card>;

/// Errors that can occure during a game.
#[derive(Debug)]
pub enum GameError
{
    DealFromDeckWithInsufficientCards,
    DealWithSameColeredCardsOnTable,
    EliminateEmptyRow,
    EliminateNoGreaterCard,
    PlaceFromSingleCardRow,
    PlaceToNonEmptyRow,
}

impl Display for GameError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{:?}", self)
    }
}

impl Error for GameError {}

#[derive(Debug, Eq, PartialEq)]
pub enum GameResult
{
    Win,
    Lose,
}

#[derive(Getters)]
pub struct Game
{
    #[getset(get = "pub")]
    deck: Deck,

    #[getset(get = "pub")]
    table: Table,

    discard_pile: DiscardPile,
    history: History,
}

impl Game
{
    /// Creates a new game.
    pub fn new(deck: Deck) -> Game
    {
        Game {
            deck,
            table: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            discard_pile: DiscardPile::new(),
            history: History::new(),
        }
    }

    /// Checks if it is possible to deal cards on the current table.
    pub fn check_deal(&self) -> Result<(), GameError>
    {
        use GameError::{
            DealFromDeckWithInsufficientCards,
            DealWithSameColeredCardsOnTable,
        };

        // Check that there are enough cards in the deck.
        (self.deck.len() > 3)
            .then(|| ())
            .ok_or(DealFromDeckWithInsufficientCards)?;

        // Check that there are no two cards of the same color on the table.
        let mut suits = HashSet::new();
        for row in self.table.iter() {
            if let Some(suit) = row.last().map(Card::suit) {
                (suits.insert(suit))
                    .then(|| ())
                    .ok_or(DealWithSameColeredCardsOnTable)?;
            }
        }

        Ok(())
    }

    /// Draws cards from the deck and places them on the table.
    pub fn deal(&mut self) -> Result<&Event, GameError>
    {
        self.check_deal()?;

        let table = &mut self.table;
        let deck = &mut self.deck;

        // Draw cards from deck and place on table.
        for i in 0..4 {
            let card = deck.draw().unwrap();
            table[i].push(card);
        }

        // Add the event to the history.
        self.history
            .push(Event::Deal([
                table[0].last().unwrap().clone(),
                table[1].last().unwrap().clone(),
                table[2].last().unwrap().clone(),
                table[3].last().unwrap().clone(),
            ]))
            .unwrap();
        Ok(self.history.last().unwrap())
    }

    /// Checks if it is possible to eliminate the card at `index`.
    pub fn check_eliminate(&self, index: RowIndex) -> Result<(), GameError>
    {
        use GameError::{
            EliminateEmptyRow,
            EliminateNoGreaterCard,
        };

        let table = &self.table;

        // Get the card that is supposed to be eliminated.
        let card = table[index].last().ok_or(EliminateEmptyRow)?;

        // Check that there is a greater card.
        table
            .iter()
            .map(|row| row.last().map(|other| other.greater(card)))
            .any(|compare| compare == Some(Some(true)))
            .then(|| ())
            .ok_or(EliminateNoGreaterCard)?;

        Ok(())
    }

    /// Eliminates the card at `index` if there are a greater card on the table.
    pub fn eliminate(&mut self, index: RowIndex) -> Result<&Event, GameError>
    {
        self.check_eliminate(index)?;

        // Remove the card at the row at `index` and add it to the discard pile.
        let table = &mut self.table;
        self.discard_pile.push(table[index].pop().unwrap());

        // Add the event to the history.
        self.history.push(Event::Eliminate(index)).unwrap();
        Ok(self.history.last().unwrap())
    }

    /// Checks if it is possible to place a card from the row with index `from` to the row with
    /// index `to`.
    pub fn check_place(&self, from: RowIndex, to: RowIndex) -> Result<(), GameError>
    {
        use GameError::{
            PlaceFromSingleCardRow,
            PlaceToNonEmptyRow,
        };

        let table = &self.table;

        // The row the card is moved from must contain at least two cards.
        (table[from].len() > 1)
            .then(|| ())
            .ok_or(PlaceFromSingleCardRow)?;

        // The row the card is placed at must be empty.
        table[to]
            .last()
            .is_none()
            .then(|| ())
            .ok_or(PlaceToNonEmptyRow)?;

        Ok(())
    }

    /// Places a card to an empty row.
    pub fn place(&mut self, from: RowIndex, to: RowIndex) -> Result<&Event, GameError>
    {
        self.check_place(from, to)?;

        let table = &mut self.table;

        // Place the card.
        let card = table[from].pop().unwrap();
        table[to].push(card);

        // Add event to the history.
        self.history.push(Event::Place(from, to)).unwrap();
        Ok(self.history.last().unwrap())
    }

    pub fn end(self) -> History
    {
        self.history.end()
    }

    pub fn game_result(&self) -> Option<GameResult>
    {
        //
        // Exit early if the game is not over.
        //

        // Continue if the deck is empty.
        self.deck.is_empty().then(|| ())?;

        // Continue if no elimination is possible.
        (0..4)
            .map(|i| self.check_eliminate(i))
            .all(|result| result.is_err())
            .then(|| ())?;

        // Continue if it is not possible to place any card.
        (!((self.table.iter().map(Vec::is_empty).any(|x| x))
            && (self.table.iter().map(Vec::len).any(|len| len > 1))))
        .then(|| ())?;

        //
        // There is no possible moves so check if the game was a win or lose.
        //
        Some(
            if self
                .table
                .iter()
                .map(|row| (row.len() == 1).then(|| row.first()).flatten())
                .map(|card| card.map(Card::rank))
                .all(|rank| rank == Some(&Rank::King))
            {
                GameResult::Win
            } else {
                GameResult::Lose
            },
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::game::card::{
        Card,
        Rank,
        Suit,
    };

    fn empty_deck() -> Deck
    {
        let mut deck = Deck::new();

        while deck.draw().is_some() {}
        deck
    }

    #[test]
    fn game_result_win()
    {
        let mut game = Game::new(empty_deck());

        game.table = [
            vec![Card::new(Suit::Spades, Rank::King)],
            vec![Card::new(Suit::Hearts, Rank::King)],
            vec![Card::new(Suit::Clubs, Rank::King)],
            vec![Card::new(Suit::Diamonds, Rank::King)],
        ];

        assert_eq!(game.game_result(), Some(GameResult::Win));
    }
}
