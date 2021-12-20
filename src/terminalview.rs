use crate::model::Model;
use crate::View;

use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
#[allow(unused_imports)]
use std::rc::Rc;
use termion::raw::{IntoRawMode, RawTerminal};

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
            screencols: size.0 as usize,
            screenrows: size.1 as usize,
        }
    }

    fn draw_rows(&self) {
        let model = self.model.borrow();
        let screenrows = self.get_window_size().screenrows;

        for r in 0..(screenrows - 2) {
            let row_idx = r + model.rowoff;

            if row_idx < model.num_rows() {
                // Print a standard row
                self.draw_row(row_idx);
            } else if model.num_rows() == 0 && r == screenrows / 3 {
                // Print a welcome message
                self.draw_welcome();
            } else {
                // Print a row placeholder
                println!("~\r");
            }
        }
    }

    fn draw_row(&self, row_idx: usize) {
        let model = self.model.borrow();
        let screencols = self.get_window_size().screencols;
        let render = model.get_render(row_idx, 0, screencols).unwrap();
        println!("{}\r", render);
    }

    fn draw_welcome(&self) {
        let screencols = self.get_window_size().screencols;
        let welcome_msg = format!("Rusk editor -- version 0.1");
        let msg_len = welcome_msg.len();
        let padding = ((screencols).saturating_sub(msg_len)) / 2;

        let mut welcome_msg = format!("~{}{}", " ".repeat(padding.saturating_sub(1)), welcome_msg);
        welcome_msg.truncate(screencols);
        println!("{}\r", welcome_msg);
    }

    fn draw_status_bar(&self, screencols: usize) {
        let model = self.model.borrow();

        let filename = {
            if model.filename.is_empty() {
                String::from("[No name]")
            } else {
                model.filename.clone()
            }
        };

        let modified = {
            if model.dirty > 0 {
                String::from("(modified)")
            } else {
                String::from("")
            }
        };

        let extension = {
            if model.ext.is_empty() {
                String::from("Plaintext")
            } else {
                model.ext.clone()
            }
        };

        let lines = model.num_rows();

        let lstatus = format!("{} - {} lines {}", filename, lines, modified);
        let rstatus = format!("{} | {}/{}", extension, model.cy + 1, lines);
        let padding = screencols.saturating_sub(lstatus.len() + rstatus.len());
        println!("{}{}{}\r", lstatus, " ".repeat(padding), rstatus);
    }

    fn draw_message_bar(&self) {}

    fn draw_cursor(&self) {
        let model = self.model.borrow();
        let y = model.cy.saturating_sub(model.rowoff);
        let x = model.rx.saturating_sub(model.coloff);

        print!("{}", termion::cursor::Goto((x + 1) as u16, (y + 1) as u16));
        print!("{}", termion::cursor::Show);
    }
}

impl View for TerminalView {
    fn draw(&self) {
        let size = self.get_window_size();
        let screenrows = size.screenrows;
        let screencols = size.screencols;
        
        self.draw_rows();
        self.draw_status_bar(screencols);
        self.draw_message_bar();
        self.draw_cursor();
    }
}
