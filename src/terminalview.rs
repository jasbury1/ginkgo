use crate::model::Model;

#[allow(unused_imports)]
use std::io::stdout;
use termion::raw::{IntoRawMode, RawTerminal};
use std::cell::RefCell;
use std::rc::Rc;

pub trait View {
    fn draw(&self) -> ();
}

pub struct TerminalView {
    model: Rc<RefCell<Model>>,
    _raw_handle: RawTerminal<std::io::Stdout>,
}

impl TerminalView {
    pub fn new(model: Rc<RefCell<Model>>) -> TerminalView {
        TerminalView {
            model: model,
            _raw_handle: stdout().into_raw_mode().unwrap(),
        }
    }
}

impl View for TerminalView {
    fn draw(&self) {}
}