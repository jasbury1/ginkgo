use std::io::Stdout;

use crossterm::event::Event;

use crate::display::{CellBlock, Display};

pub enum EventResponse {
    NoResponse,
    MoveCursor,
    RedrawDisplay,
    Quit
}

pub trait Component {
    type Message;

    fn send_msg(&mut self, msg: &Self::Message) -> EventResponse;
    fn handle_event(&mut self, event: Event) -> EventResponse;
    fn draw(&mut self, bounds: &Rect, displ: &mut Display<Stdout>);
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn default() -> Self {
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn contains_point(&self, point: (usize, usize)) -> bool {
        point.0 >= self.x
            && point.0 < (self.x + self.width)
            && point.1 >= self.y
            && point.1 < (self.y + self.height)
    }
}
