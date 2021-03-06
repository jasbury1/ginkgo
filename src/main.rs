mod model;
mod terminalcontroller;
mod terminalview;
mod command;
mod syntax;

use clap::{App, Arg};
use model::Model;
use std::cell::RefCell;
use std::rc::Rc;
use terminalcontroller::TerminalController;
use terminalview::TerminalView;


const GINKGO_VERSION: &str = "0.1";


pub trait View {
    fn draw(&self);
}

pub trait InputHandler {
    fn process_input(&mut self) -> Result<bool, std::io::Error>;
}

fn main() {
    let args = App::new("Ginkgo")
        .version(GINKGO_VERSION)
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
    let view = TerminalView::new(Rc::clone(&model));
    let mut controller = TerminalController::new(Rc::clone(&model), &view);

    model.borrow_mut().open_file(filename);
     
    loop {
        view.draw();
        // Returns true to continue processing input, or false to terminate
        match controller.process_input() {
            Ok(true) => {
                continue;
            }
            Ok(false) => {
                view.cleanup();
                return;
            }
            Err(_) => {
                return;
            }
        }
    }
}
