use std::rc::Rc;

use cursive::{
    direction::Direction,
    event::{
        Callback,
        Event,
        EventResult,
        Key,
    },
    theme::{
        BaseColor,
        Color,
        ColorStyle,
    },
    traits::With,
    view::View,
    Cursive,
    Printer,
    Vec2,
};
use getset::{
    Getters,
    Setters,
};

use crate::{
    controllers::Mode,
    game::card::{
        Card,
        Suit,
    },
};

#[derive(Getters, Setters)]
pub struct RowView
{
    cards: Vec<Card>,

    #[getset(set)]
    on_action: Option<Rc<dyn Fn(&mut Cursive, Option<Mode>)>>,

    #[getset(get = "pub", set = "pub")]
    mode: Option<Mode>,
}

impl RowView
{
    pub fn new() -> RowView
    {
        RowView {
            cards: Vec::new(),
            mode: None,
            on_action: None,
        }
    }

    pub fn with_on_action<F: Fn(&mut Cursive, Option<Mode>) + 'static>(self, lambda: F) -> RowView
    {
        self.with(|view| {
            view.set_on_action(Some(Rc::new(lambda)));
        })
    }

    // ---------------------------------------------------------------------------------------------
    // Control
    // ---------------------------------------------------------------------------------------------

    pub fn push(&mut self, card: Card)
    {
        self.cards.push(card);
    }

    pub fn pop(&mut self) -> Option<Card>
    {
        self.cards.pop()
    }

    pub fn clear(&mut self)
    {
        self.cards.clear();
    }
}

// -------------------------------------------------------------------------------------------------
// Draw helper functions.
// -------------------------------------------------------------------------------------------------

fn display_card(card: &Card) -> String
{
    let text = format!(" {}", card);
    if text.len() > 3 {
        text
    } else {
        format!("{} ", text)
    }
}

fn draw_card(card: &Card, select: bool, printer: &Printer)
{
    let color = match card.suit() {
        Suit::Hearts | Suit::Diamonds => Color::Dark(BaseColor::Red),
        Suit::Spades | Suit::Clubs => Color::Dark(BaseColor::Black),
    };

    let style = if printer.focused && select {
        ColorStyle::new(color, Color::Light(BaseColor::Yellow))
    } else {
        ColorStyle::front(color)
    };

    printer.with_color(style, |printer| {
        printer.print((0, 0), &display_card(card));
    });
}

fn draw_empty(select: bool, printer: &Printer)
{
    if select {
        printer.with_color(
            ColorStyle::back(Color::Light(BaseColor::Yellow)),
            |printer| {
                printer.print((0, 0), "     ");
            },
        );
    }
}

// -------------------------------------------------------------------------------------------------
// View
// -------------------------------------------------------------------------------------------------

impl View for RowView
{
    fn draw(&self, printer: &Printer)
    {
        if self.cards.is_empty() {
            draw_empty(printer.focused, printer);
        } else {
            // Draw the cards in the row.
            for (y, card) in self.cards.iter().enumerate() {
                draw_card(card, self.cards.len() - 1 == y, &printer.offset((0, y)));
            }
        }
    }

    fn take_focus(&mut self, _: Direction) -> bool
    {
        self.mode
            .as_ref()
            .map(|mode| match mode {
                Mode::Eliminate => !self.cards.is_empty(),
                Mode::PlaceFrom => self.cards.len() > 1,
                Mode::PlaceTo => self.cards.is_empty(),
            })
            .unwrap_or(false)
    }

    fn on_event(&mut self, event: Event) -> EventResult
    {
        match event {
            Event::Key(Key::Enter) => {
                EventResult::Consumed(self.on_action.as_ref().map(|on_action| {
                    let mode = self.mode.clone();
                    let on_action = Rc::clone(&on_action);
                    Callback::from_fn_once(move |s| on_action(s, mode))
                }))
            }
            _ => EventResult::Ignored,
        }
    }

    fn required_size(&mut self, _: Vec2) -> Vec2
    {
        Vec2::new(5, self.cards.len())
    }
}
