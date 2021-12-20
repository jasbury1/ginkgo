use crate::model::Model;

use std::cell::RefCell;
#[allow(unused_imports)]
use std::rc::Rc;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{stdin, stdout, Write};

pub trait View {
    fn draw(&self) -> ();
}

struct TerminalSize {
    screenrows: usize,
    screencols: usize,
}

pub struct TerminalView {
    model: Rc<RefCell<Model>>,
    _stdout: RawTerminal<std::io::Stdout>,
}

impl TerminalView {
    pub fn new(model: Rc<RefCell<Model>>) -> TerminalView {
        TerminalView {
            model: model,
            _stdout: stdout().into_raw_mode().unwrap(),
        }
    }

    fn clear_widow() {
        print!("{}", termion::clear::All);
    }

    fn get_window_size(&self) -> TerminalSize {
        let size = termion::terminal_size().unwrap();
        TerminalSize {
            screenrows: size.0 as usize,
            screencols: size.1 as usize,
        }
    }

    fn draw_rows(&self) {
        let model = self.model.borrow();
        let screenrows = self.get_window_size().screenrows;

        for r in 0..(screenrows - 1) {
            if r < model.num_rows() {
                // Print a standard row
                self.draw_row(r);
            } else if model.num_rows() == 0 && r == screenrows / 3 {
                // Print a welcome message
                self.draw_welcome();
            } else {
                // Print a row placeholder
                println!("~/r");
            }
        }
    }

    fn draw_row(&self, row_idx: usize) {
        
    }

    fn draw_welcome(&self) {
        let screencols = self.get_window_size().screencols;
        let welcome_msg = format!("Rusk editor -- version 0.1");
        let msg_len = welcome_msg.len();
        let padding = ((screencols).saturating_sub(msg_len)) / 2;

        let mut welcome_msg = format!("~{}{}", "".repeat(padding.saturating_sub(1)), welcome_msg);
        welcome_msg.truncate(screencols);
        println!("{}", welcome_msg);
    }
}

impl View for TerminalView {
    fn draw(&self) {
        self.draw_rows();
    }
}
