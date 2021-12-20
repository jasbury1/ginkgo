use crate::model::Model;
use crate::terminalview::TerminalView;
use crate::Controller;
use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct TerminalController<'a> {
    model: Rc<RefCell<Model>>,
    view: &'a TerminalView,
}

impl<'a> TerminalController<'a> {
    pub fn new(model: Rc<RefCell<Model>>, view: &'a TerminalView) -> TerminalController<'a> {
        TerminalController {
            model: model,
            view: view,
        }
    }

    fn save(&self) {}

    fn move_cursor(&self, key: termion::event::Key) {}

    fn delete_char(&self) {}

    fn page_down(&self) {}

    fn page_up(&self) {}

    fn insert_char(&self, c: char) {}
}

impl<'a> Controller for TerminalController<'a> {
    fn process_input(&self) -> Result<bool, std::io::Error> {
        let stdin = stdin();
        for k in stdin.keys() {
            //i reckon this speaks for itself
            let key = k.unwrap();
            match key {
                Key::Ctrl('q') => {
                    return Ok(false)
                },
                Key::Ctrl('s') => {
                    self.save();
                },
                Key::Esc => {}
                Key::Left | Key::Right | Key::Up | Key::Down => {
                    self.move_cursor(key);
                },
                Key::Backspace | Key::Delete | Key::Ctrl('h') => {
                    self.delete_char();
                },
                Key::PageDown => {
                    self.page_down();
                },
                Key::PageUp => {
                    self.page_up();
                },
                Key::Char(c) => {
                    self.insert_char(c);
                },
                Key::Ctrl(_) | Key::Alt(_) => {},
                _ => {},
            };
        }
        Ok(true)
    }
}
