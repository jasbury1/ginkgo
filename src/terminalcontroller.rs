use crate::model::Model;
use crate::terminalview::TerminalView;
use crate::Controller;
use std::cell::RefCell;
use std::rc::Rc;
use std::io::{stdin, stdout, Write};
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


}

impl<'a> Controller for TerminalController<'a> {
    fn process_input(&self) -> Result<bool, std::io::Error> {
        let stdin = stdin();
        for c in stdin.keys() {
            //i reckon this speaks for itself
            match c.unwrap() {
                Key::Ctrl('q') => {
                    return Ok(false)
                },
                _ => {}
            };
        }
        Ok(true)
    }
}
