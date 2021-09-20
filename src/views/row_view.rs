use std::rc::Rc;

use cursive::{
    align::HAlign,
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
use unicode_width::UnicodeWidthStr;

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
// View
// -------------------------------------------------------------------------------------------------

impl View for RowView
{
    fn draw(&self, printer: &Printer)
    {
        if self.cards.is_empty() {
            // Check if empty row should be focused.
            if printer.focused {
                printer.with_color(
                    ColorStyle::back(Color::Light(BaseColor::Yellow)),
                    |printer| {
                        printer.print((0, 0), "     ");
                    },
                );
            }
        } else {
            // Draw the cards in the row.
            for (y, card) in self.cards.iter().enumerate() {
                let color = match card.suit() {
                    Suit::Hearts | Suit::Diamonds => Color::Dark(BaseColor::Red),
                    Suit::Spades | Suit::Clubs => Color::Dark(BaseColor::Black),
                };

                let style = if printer.focused && self.cards.len() - 1 == y {
                    ColorStyle::new(color, Color::Light(BaseColor::Yellow))
                } else {
                    ColorStyle::front(color)
                };

                let s = String::from(card);
                let offset = HAlign::Center.get_offset(s.width(), printer.size.x);

                printer.with_color(style, |printer| printer.print((offset, y), &s));
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
        Vec2::from((6, self.cards.len()))
    }
}
