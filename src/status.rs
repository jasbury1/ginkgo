use crossterm::style::Color;

use crate::{ui::{Component, Rect}, display::{CellBlock, Cell, Display}};

const DEFAULT_MSG: &'static str = "[Normal]";

pub enum StatusMsg {
    Default,
    Normal(String),
    Warn(String),
    Error(String),
}

pub struct StatusBarComponent {
    status_msg: StatusMsg,
    cursor_active: bool,
}

impl Component for StatusBarComponent {
    type Message = StatusMsg;

    fn send_msg(&mut self, msg: &StatusMsg) {
        todo!()
    }

    fn draw(&mut self, bounds: &Rect, displ: &mut Display) {
        let mut cellblock = Cell::empty_cellblock(bounds.width, bounds.height);
        let text_color: Color;
        let msg = match &self.status_msg {
            StatusMsg::Default => {text_color = Color::White; DEFAULT_MSG},
            StatusMsg::Normal(str) => {text_color = Color::White; &str},
            StatusMsg::Warn(str) => {text_color = Color::Yellow; &str},
            StatusMsg::Error(str) => {text_color = Color::Red; &str},
        };
        for (i, c) in msg.chars().enumerate() {
            if i >= bounds.width {
                break;
            }
            cellblock[0][i].c = c;
        }
        displ.draw(bounds, &cellblock);
    }
}

impl StatusBarComponent {
    pub fn new() -> Self {
        StatusBarComponent{status_msg: StatusMsg::Default, cursor_active: false}
    }
}

