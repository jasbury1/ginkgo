mod model;
mod terminalcontroller;
mod terminalview;

use clap::{App, Arg};
use model::Model;
use std::cell::RefCell;
#[allow(unused_imports)]
use std::io::{self, stdout, Read};
use std::rc::Rc;
use terminalcontroller::{Controller, TerminalController};
use terminalview::{TerminalView, View};

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
        controller.process_input();
    }
}
