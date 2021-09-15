use std::{
    cell::RefCell,
    iter::zip,
    rc::Rc,
};

use cursive::{
    theme::{
        BaseColor,
        Color,
        ColorStyle,
    },
    view::{
        Nameable,
        Resizable,
    },
    views::{
        Button,
        Dialog,
        DummyView,
        Layer,
        LinearLayout,
        NamedView,
    },
    Cursive,
};

use crate::{
    controllers::ModelController,
    game::{
        Event,
        GameResult,
        RowIndex,
    },
    views::RowView,
};

#[derive(Debug, Clone)]
pub enum Mode
{
    Eliminate,
    PlaceFrom,
    PlaceTo,
}

pub struct ViewController
{
    _model_controller: Rc<RefCell<ModelController>>,
    _from_index: Rc<RefCell<Option<RowIndex>>>,
}

static ROW_NAMES: [&str; 4] = ["row-0", "row-1", "row-2", "row-3"];

impl ViewController
{
    pub fn new(s: &mut Cursive) -> ViewController
    {
        let model_controller = Rc::new(RefCell::new(ModelController::new()));
        let from_index = Rc::new(RefCell::new(None));

        //
        // Initialize RowViews.
        //
        let mut rows = LinearLayout::horizontal();

        for (index, &name) in ROW_NAMES.iter().enumerate() {
            let model_controller = Rc::downgrade(&model_controller);
            let from_index = Rc::downgrade(&from_index);

            rows.add_child(
                RowView::new()
                    .with_on_action(move |s, mode| {
                        mode.map(|mode| match mode {
                            Mode::Eliminate => {
                                model_controller.upgrade().map(|model_controller| {
                                    model_controller.borrow_mut().eliminate(s, index)
                                });
                            }
                            Mode::PlaceFrom => {
                                from_index.upgrade().map(|from_index| {
                                    let _ = from_index.borrow_mut().insert(index);
                                });
                                set_mode(s, Mode::PlaceTo);
                            }
                            Mode::PlaceTo => {
                                from_index.upgrade().map(|from_index| {
                                    from_index.borrow_mut().take().map(|from_index| {
                                        model_controller.upgrade().map(|model_controller| {
                                            model_controller
                                                .borrow_mut()
                                                .place(s, from_index, index);
                                        })
                                    })
                                });
                                set_mode(s, Mode::Eliminate);
                            }
                        });
                    })
                    .with_name(name),
            )
        }

        //
        // Add screen to cursive.
        //
        s.add_layer(
            LinearLayout::horizontal()
                .child(rows)
                .child(DummyView.fixed_width(1))
                .child(
                    LinearLayout::vertical()
                        .child({
                            let model_controller = Rc::downgrade(&model_controller);

                            Button::new("Deck[-]", move |s| {
                                model_controller
                                    .upgrade()
                                    .map(|model_controller| model_controller.borrow_mut().deal(s));
                            })
                            .with_name("deck-button")
                        })
                        .child(
                            // TODO:
                            //   It does not render well when the button is highlighted due to place
                            //   mode being enabled.
                            // TODO:
                            //   If the button is disabled after a place and the player moved the
                            //   focus to the buttons, the focus will end up at the view
                            //   'new-game-button'. It would be preferable if it focused the view
                            //   'deck-button' instead.
                            Layer::new(Button::new("Place", |_| {}).with_name("place-button"))
                                .with_name("place-button-layer"),
                        )
                        .child(DummyView)
                        .child({
                            let model_controller = Rc::downgrade(&model_controller);

                            Button::new("New Game", move |s| {
                                model_controller.upgrade().map(|model_controller| {
                                    model_controller.borrow_mut().new_game(s)
                                });
                            })
                        })
                        .child({
                            let model_controller = Rc::downgrade(&model_controller);

                            Button::new("Quit", move |s| {
                                model_controller.upgrade().map(|model_controller| {
                                    model_controller.borrow_mut().drop_game()
                                });
                                s.quit();
                            })
                        })
                        .child(DummyView.fixed_height(1)),
                ),
        );

        //
        // Set up model controller lambdas.
        //
        model_controller
            .borrow_mut()
            .set_on_event(Some(Box::new(move |s, event| match event {
                Event::Deal(cards) => {
                    for (name, card) in zip(ROW_NAMES, cards) {
                        s.call_on_name(name, |view: &mut RowView| view.push(card.clone()));
                    }
                }
                Event::Eliminate(index) => {
                    s.call_on_name(ROW_NAMES[*index], |view: &mut RowView| view.pop());
                }
                Event::Place(from, to) => {
                    if let Some(card) = s
                        .call_on_name(ROW_NAMES[*from], |view: &mut RowView| view.pop())
                        .flatten()
                    {
                        s.call_on_name(ROW_NAMES[*to], move |view: &mut RowView| view.push(card));
                    }
                    set_mode(s, Mode::Eliminate);
                }
            })));
        model_controller
            .borrow_mut()
            .set_on_new_game(Some(Box::new(|s| {
                for name in ROW_NAMES {
                    s.call_on_name(name, |view: &mut RowView| view.clear());
                }
            })));
        model_controller
            .borrow_mut()
            .set_on_deck_size_changed(Some(Box::new(|s, len| {
                s.call_on_name("deck-button", |button: &mut Button| {
                    button.set_label(format!("Deck[{}]", len));
                });
            })));
        model_controller
            .borrow_mut()
            .set_on_check_deal_possible(Some(Box::new(|s, possible| {
                s.call_on_name("deck-button", |button: &mut Button| {
                    // TODO:
                    //   If the deck button is disabled after a deal the focus disappears. It would
                    //   be preferable if it was moved to some row on the table.
                    if possible {
                        button.enable();
                    } else {
                        button.disable();
                    }
                });
            })));
        model_controller
            .borrow_mut()
            .set_on_check_place_possible(Some(Box::new(|s, possible| {
                s.call_on_name("place-button", |button: &mut Button| {
                    if possible {
                        button.enable();
                    } else {
                        button.disable();
                    }
                });
            })));
        model_controller
            .borrow_mut()
            .set_on_game_over(Some(Box::new(|s, result| match result {
                // TODO:
                //   When this dialog is dismissed it would be preferable if the focus was set on
                //   the 'new-game' button.
                GameResult::Win => s.add_layer(Dialog::info("You solved the idiot! Great job!")),
                GameResult::Lose => s.add_layer(Dialog::info(
                    "Game over, you did not solve the idiot this time.",
                )),
            })));

        //
        // Prepare game.
        //
        set_mode(s, Mode::Eliminate);
        model_controller.borrow_mut().new_game(s);

        ViewController {
            _model_controller: model_controller,
            _from_index: from_index,
        }
    }
}

fn set_mode(s: &mut Cursive, mode: Mode)
{
    for name in ROW_NAMES {
        s.call_on_name(name, |view: &mut RowView| {
            view.set_mode(Some(mode.clone()));
        });
    }

    match mode {
        Mode::Eliminate => {
            s.call_on_name(
                "place-button-layer",
                |view: &mut Layer<NamedView<Button>>| {
                    view.set_color(ColorStyle::inherit_parent());
                },
            );
        }
        Mode::PlaceFrom | Mode::PlaceTo => {
            s.call_on_name(
                "place-button-layer",
                |view: &mut Layer<NamedView<Button>>| {
                    view.set_color(ColorStyle::back(Color::Light(BaseColor::Yellow)));
                },
            );
        }
    }

    s.call_on_name("place-button", |button: &mut Button| match mode {
        Mode::Eliminate => button.set_callback(|s| set_mode(s, Mode::PlaceFrom)),
        _ => button.set_callback(|s| set_mode(s, Mode::Eliminate)),
    });
}
