use chrono::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

use crate::game::{
    card::Card,
    RowIndex,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Event
{
    Deal([Card; 4]),
    Eliminate(RowIndex),
    Place(RowIndex, RowIndex),
}

#[derive(Debug)]
pub struct HistoryEndedError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct History
{
    start_date: Option<DateTime<Local>>,
    end_date: Option<DateTime<Local>>,
    events: Vec<Event>,
}

impl History
{
    pub fn new() -> History
    {
        History {
            start_date: Some(Local::now()),
            end_date: None,
            events: Vec::new(),
        }
    }

    pub fn push(&mut self, event: Event) -> Result<(), HistoryEndedError>
    {
        if self.end_date.is_none() {
            Ok(self.events.push(event))
        } else {
            Err(HistoryEndedError)
        }
    }

    pub fn end(mut self) -> History
    {
        self.end_date = Some(Local::now());
        self
    }

    pub fn last(&self) -> Option<&Event>
    {
        self.events.last()
    }
}
