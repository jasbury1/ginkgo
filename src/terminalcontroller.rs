use crate::model::Model;
use crate::terminalview::TerminalView;
use crate::Controller;
use std::cell::RefCell;
use std::rc::Rc;

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
    fn process_input(&self) {
        loop {}
    }
}
