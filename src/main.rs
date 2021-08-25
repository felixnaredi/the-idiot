#![feature(iter_zip)]

mod controllers;
mod game;
mod views;

use cursive::{
    logger,
    Cursive,
};
use log::{
    Level,
    Record,
};

use crate::controllers::ViewController;

pub fn main()
{
    let mut siv = cursive::default();

    logger::init();
    siv.add_global_callback('?', Cursive::toggle_debug_console);

    logger::log(
        &Record::builder()
            .level(Level::Debug)
            .args(format_args!("Logger initialized"))
            .build(),
    );

    let _view_controller = ViewController::new(&mut siv);
    siv.run();
}
