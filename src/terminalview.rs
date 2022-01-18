use crate::model::{Model, StatusMsg};
use crate::{View, GINKGO_VERSION};

use std::cell::RefCell;
use std::io::{stdout, Write};
#[allow(unused_imports)]
use std::rc::Rc;
use termion::color;
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
        // Initialize cursor to a block
        print!("{}", termion::cursor::SteadyBlock);
        TerminalView {
            model,
            _stdout: stdout().into_raw_mode().unwrap(),
        }
    }

    pub fn clear_widow() {
        print!("{}", termion::clear::All);
    }

    fn get_window_size() -> TerminalSize {
        let size = termion::terminal_size().unwrap();
        TerminalSize {
            screencols: size.0 as usize,
            screenrows: size.1 as usize,
        }
    }

    pub fn get_screen_rows() -> usize {
        TerminalView::get_window_size().screenrows - 2
    }

    pub fn get_screen_cols() -> usize {
        TerminalView::get_window_size().screencols
    }

    /// This is the main public function for redrawing only the screen rows
    /// It will not redraw anything else such as the status or message bars,
    /// but it will redraw the on-screen cursor based on its current location
    pub fn refresh_rows(&self) {
        print!("{}", termion::cursor::Goto(1, 1));
        let size = TerminalView::get_window_size();
        let screenrows = size.screenrows;
        let screencols = size.screencols;
        self.draw_rows(screenrows, screencols);
        self.draw_cursor();
        stdout().flush().unwrap();
    }

    fn draw_rows(&self, screenrows: usize, screencols: usize) {
        let model = self.model.borrow();

        for r in 0..(screenrows - 2) {
            let row_idx = r + model.rowoff;
            print!("{}", termion::clear::CurrentLine);
            if row_idx < model.num_rows() {
                // Print a standard row
                self.draw_row(row_idx, screencols);
            } else if model.num_rows() == 0 && r == screenrows / 3 {
                // Print a welcome message
                self.draw_welcome(screencols);
            } else {
                // Print a row placeholder
                println!("~\r");
            }
        }
    }

    fn draw_row(&self, row_idx: usize, screencols: usize) {
        let model = self.model.borrow();
        let mut contents = &model.get_row_contents(row_idx)[..];

        // Shrink contents slice down to fit in our screen
        if contents.len() > screencols {
            contents = &contents[..screencols];
        }

        if model.text_selected && self.draw_selection(contents, row_idx) {
            return;
        } else {
            println!("{}\r", contents);
        }
    }

    /// Returns true if we drew a selection
    fn draw_selection(&self, render: &str, row_idx: usize) -> bool {
        let model = self.model.borrow();
        let (anchor_start, anchor_end) = model.get_anchors();

        if row_idx < anchor_start.1 || row_idx > anchor_end.1 {
            return false;
        }

        // Selection on same line
        if row_idx == anchor_start.1 && row_idx == anchor_end.1 {
            println!(
                "{}{}{}{}{}\r",
                &render[..(anchor_start.0)],
                color::Bg(color::LightBlue),
                &render[(anchor_start.0)..(anchor_end.0)],
                color::Bg(color::Reset),
                &render[(anchor_end.0)..]
            );
        }
        // Draw start of a selection
        else if row_idx == anchor_start.1 {
            println!(
                "{}{}{}{}\r",
                &render[..(anchor_start.0)],
                color::Bg(color::LightBlue),
                &render[(anchor_start.0)..],
                color::Bg(color::Reset)
            );
        }
        // Draw end of a selection
        else if row_idx == anchor_end.1 {
            println!(
                "{}{}{}{}\r",
                color::Bg(color::LightBlue),
                &render[..(anchor_end.0)],
                color::Bg(color::Reset),
                &render[(anchor_end.0)..]
            );
        }
        // Draw full line
        else {
            println!(
                "{}{}{}\r",
                color::Bg(color::LightBlue),
                render,
                color::Bg(color::Reset),
            );
        }
        true
    }

    fn draw_welcome(&self, screencols: usize) {
        let welcome_msg = format!("Ginkgo editor -- version {}", GINKGO_VERSION);
        let msg_len = welcome_msg.len();
        let padding = ((screencols).saturating_sub(msg_len)) / 2;

        let mut welcome_msg = format!("~{}{}", " ".repeat(padding.saturating_sub(1)), welcome_msg);
        welcome_msg.truncate(screencols);
        print!("{}", termion::clear::CurrentLine);
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
            if model.dirty {
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
        let rstatus = format!(
            "<{}> {} | {}/{} ",
            model.mode,
            extension,
            model.cy + 1,
            lines
        );
        let padding = screencols.saturating_sub(lstatus.len() + rstatus.len());
        print!("{}", termion::clear::CurrentLine);
        println!(
            "{}{}{}{}{}{}\r",
            color::Bg(color::White),
            color::Fg(color::Black),
            lstatus,
            " ".repeat(padding),
            rstatus,
            color::Bg(color::Reset)
        );
    }

    fn draw_message_bar(&self, screencols: usize) {
        let model = self.model.borrow();
        let mut message = match &model.status_msg {
            StatusMsg::Normal(msg) => {
                format!(
                    "{}{}{}",
                    color::Fg(color::White),
                    msg,
                    color::Fg(color::Reset)
                )
            }
            StatusMsg::Warn(msg) => {
                format!(
                    "{}{}{}",
                    color::Fg(color::Yellow),
                    msg,
                    color::Fg(color::Reset)
                )
            }
            StatusMsg::Error(msg) => {
                format!(
                    "{}{}{}",
                    color::Fg(color::Red),
                    msg,
                    color::Fg(color::Reset)
                )
            }
        };
        message.truncate(screencols);
        print!("{}", termion::clear::CurrentLine);
        print!("{}", message);
    }

    fn draw_cursor(&self) {
        let model = self.model.borrow();
        let y = model.cy.saturating_sub(model.rowoff);
        let x = model.cx.saturating_sub(model.coloff);

        print!("{}", termion::cursor::Hide);
        print!("{}", termion::cursor::Goto((x + 1) as u16, (y + 1) as u16));
        print!("{}", termion::cursor::Show);
    }

    pub fn cleanup(&self) {
        TerminalView::clear_widow();
    }

    pub fn draw_prompt(&self, prompt: &str, msg: &str) {
        let size = TerminalView::get_window_size();
        let screencols = size.screencols;
        print!("{}", termion::cursor::Goto(1, screencols as u16));
        print!("{}", termion::clear::CurrentLine);
        print!("{} {}", prompt, msg);
        print!(
            "{}",
            termion::cursor::Goto((prompt.len() + msg.len() + 2) as u16, screencols as u16)
        );
        stdout().flush().unwrap();
    }
}

impl View for TerminalView {
    fn draw(&self) {
        print!("{}", termion::cursor::Goto(1, 1));
        let size = TerminalView::get_window_size();
        let screenrows = size.screenrows;
        let screencols = size.screencols;
        self.draw_rows(screenrows, screencols);
        self.draw_status_bar(screencols);
        self.draw_message_bar(screencols);
        self.draw_cursor();
        stdout().flush().unwrap();
    }
}
