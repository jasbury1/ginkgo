mod model;
mod terminalview;

#[allow(unused_imports)]
use model::Model;
use terminalview::TerminalView;
use std::io::{self, stdout, Read};
use termion::raw::IntoRawMode;

fn main() {
    let _raw = stdout().into_raw_mode().unwrap();

    let model = Model {};
    let view = TerminalView {};
    loop {

    }
}
