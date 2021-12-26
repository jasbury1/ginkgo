mod model;
mod syntax;
mod terminalcontroller;
mod terminalview;

use clap::{App, Arg};
use model::Model;
use std::cell::RefCell;
#[allow(unused_imports)]
use std::io::{self, stdout, Read};
use std::rc::Rc;
use terminalcontroller::TerminalController;
use terminalview::TerminalView;

pub trait View {
    fn draw(&self) -> ();
}

pub trait InputHandler {
    fn process_input(&mut self) -> Result<bool, std::io::Error>;
}

fn main() {
    let args = App::new("Rusk")
        .version("0.1")
        .about("Edits a file")
        .arg(
            Arg::with_name("file")
                .help("The file to open")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let filename = args.value_of("file").unwrap();

    let model = Rc::new(RefCell::new(Model::new()));
    let mut view = TerminalView::new(Rc::clone(&model));
    let mut controller = TerminalController::new(Rc::clone(&model), &view);

    model.borrow_mut().open_file(filename);

    loop {
        view.draw();
        match controller.process_input() {
            Ok(true) => {
                continue;
            }
            Ok(false) => {
                return;
            }
            Err(_) => {
                return;
            }
        }
    }
}
