use std::fmt::{
    self,
    Display,
};

use getset::Getters;
use rand::Rng;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Suit
{
    Diamonds,
    Clubs,
    Hearts,
    Spades,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Rank
{
    Ace,
    Num(u32),
    Knight,
    Queen,
    King,
}

#[derive(Clone, Debug, Deserialize, Eq, Getters, Hash, PartialEq, Serialize)]
pub struct Card
{
    #[getset(get = "pub")]
    suit: Suit,

    #[getset(get = "pub")]
    rank: Rank,
}

impl Card
{
    pub fn new(suit: Suit, rank: Rank) -> Card
    {
        Card { suit, rank }
    }

    pub fn greater(&self, other: &Card) -> Option<bool>
    {
        (self.suit() == other.suit()).then(|| self.rank() > other.rank())
    }
}

impl Display for Card
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        use Rank::*;
        use Suit::*;

        match self.rank() {
            Ace => write!(f, "A"),
            Num(x) => write!(f, "{}", x),
            Knight => write!(f, "Kn"),
            Queen => write!(f, "Q"),
            King => write!(f, "K"),
        }?;
        match self.suit() {
            Spades => write!(f, "<<"),
            Hearts => write!(f, "<3"),
            Clubs => write!(f, "cc"),
            Diamonds => write!(f, "<>"),
            // TODO:
            //   I would like to use Unicode characters for the suit but it doesn't render well
            //   when the focus is changed. My guess is that uses amount of bytes and not amount of
            //   characters when redrawing.
            //
            //   Maybe I have just missed some setting... Otherwise it would be neat addition for
            //   cursive.
            /*
            Spades => write!(f, "♠️"),
            Hearts => write!(f, "♥️"),
            Clubs => write!(f, "♣️"),
            Diamonds => write!(f, "♦️"),
            */
        }
    }
}

#[derive(Debug)]
pub struct Deck(Vec<Card>);

impl Deck
{
    /// Creates a new deck.
    pub fn new() -> Deck
    {
        use {
            Rank::*,
            Suit::*,
        };

        Deck(
            vec![Spades, Hearts, Clubs, Diamonds]
                .into_iter()
                .map(|suit| {
                    vec![
                        Ace,
                        Num(2),
                        Num(3),
                        Num(4),
                        Num(5),
                        Num(6),
                        Num(7),
                        Num(8),
                        Num(9),
                        Num(10),
                        Knight,
                        Queen,
                        King,
                    ]
                    .into_iter()
                    .map(move |rank| Card::new(suit.clone(), rank))
                })
                .flatten()
                .collect(),
        )
    }

    /// Shuffles the deck in place.
    pub fn shuffle(&mut self)
    {
        let Deck(cards) = self;

        let mut rng = rand::thread_rng();
        let len = cards.len();

        for i in 0..cards.len() {
            let j = rng.gen_range(i..len);
            cards.swap(i, j);
        }
    }

    /// Creates a new shuffled deck.
    pub fn shuffled() -> Deck
    {
        let mut deck = Deck::new();
        deck.shuffle();
        deck
    }

    /// Draws the top card, if any.
    ///
    /// # NOTE:
    /// It's really the last element of the array so the order from drawing cards and iterating is
    /// reversed.
    pub fn draw(&mut self) -> Option<Card>
    {
        let Deck(cards) = self;
        cards.pop()
    }

    /// Amount of cards remaining in the deck.
    pub fn len(&self) -> usize
    {
        let Deck(cards) = self;
        cards.len()
    }

    /// True if there are no more cards to draw in the deck.
    pub fn is_empty(&self) -> bool
    {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests
{
    use std::collections::HashSet;

    use super::*;

    // Just making sure `PartialOrd` and `Ord` works as indended.
    #[test]
    fn compare_ranks()
    {
        use Rank::*;

        assert!(Ace < Num(2));
        assert!(Num(2) < Num(10));
        assert!(Num(10) < Knight);
        assert!(Knight < Queen);
        assert!(Queen < King);

        assert!(!(Ace > Num(2)));
        assert!(!(Num(2) > Num(10)));
        assert!(!(Num(10) > Knight));
        assert!(!(Knight > Queen));
        assert!(!(Queen > King));
    }

    #[test]
    fn greater_works()
    {
        use {
            Rank::*,
            Suit::*,
        };

        assert!(Card::new(Hearts, Num(3))
            .greater(&Card::new(Hearts, Num(2)))
            .unwrap());

        assert!(
            !(Card::new(Spades, Num(3))
                .greater(&Card::new(Spades, Num(4)))
                .unwrap())
        );

        assert!(Card::new(Diamonds, Queen)
            .greater(&Card::new(Diamonds, Ace))
            .unwrap());

        assert!(Card::new(Spades, Num(5))
            .greater(&Card::new(Clubs, Num(4)))
            .is_none());
    }

    #[test]
    fn deck_has_52_unique_cards()
    {
        let Deck(cards) = Deck::new();
        let cards: HashSet<Card> = cards.into_iter().collect();
        assert_eq!(cards.len(), 52);
    }
}
