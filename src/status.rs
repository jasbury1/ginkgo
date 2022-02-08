use crossterm::style::Color;

use crate::{
    display::{Cell, CellBlock, Display},
    ui::{Component, Rect, EventResponse},
};

const DEFAULT_MSG: &'static str = "[Normal]";

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StatusMsg {
    Default,
    Normal(String),
    Warn(String),
    Error(String),
}

pub struct StatusBarComponent {
    status_msg: StatusMsg,
    cell_cache: Option<CellBlock>,
}

impl Component for StatusBarComponent {
    type Message = StatusMsg;

    fn send_msg(&mut self, msg: &StatusMsg) -> EventResponse {
        self.status_msg = (*msg).clone();
        EventResponse::RedrawDisplay
    }

    fn draw(&mut self, bounds: &Rect, displ: &mut Display) {
        let mut cellblock = Cell::empty_cellblock(bounds.width, bounds.height);
        let text_color: Color;
        let msg = match &self.status_msg {
            StatusMsg::Default => {
                text_color = Color::White;
                DEFAULT_MSG
            }
            StatusMsg::Normal(str) => {
                text_color = Color::White;
                &str
            }
            StatusMsg::Warn(str) => {
                text_color = Color::Yellow;
                &str
            }
            StatusMsg::Error(str) => {
                text_color = Color::Red;
                &str
            }
        };
        for (i, c) in msg.chars().enumerate() {
            if i >= bounds.width {
                break;
            }
            cellblock[0][i].c = c;
            cellblock[0][i].text_color = text_color;
        }
        displ.draw(bounds, &cellblock);
    }

    fn handle_event(&mut self, event: crossterm::event::Event) -> EventResponse {
        todo!()
    }
}

impl StatusBarComponent {
    pub fn new() -> Self {
        StatusBarComponent {
            status_msg: StatusMsg::Default,
            cell_cache: None,
        }
    }
}
