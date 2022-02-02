use crate::display::{CellBlock, Display};

pub trait Component {
    type Message;

    fn send_msg(&mut self, msg: &Self::Message);
    fn draw(&mut self, bounds: &Rect, displ: &mut Display);
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
        Rect { x: 0, y: 0, width: 0, height: 0 }
    }
}
