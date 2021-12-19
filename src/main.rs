mod model;
mod terminalview;
mod terminalcontroller;

#[allow(unused_imports)]
use model::Model;
use terminalview::{TerminalView, View};
use terminalcontroller::TerminalController;
use std::io::{self, stdout, Read};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {

    let model = Rc::new(RefCell::new(Model {}));
    let view = TerminalView::new(Rc::clone(&model));
    let controller = TerminalController::new(Rc::clone(&model), &view);
    
    loop {
        view.draw();
    }
}
