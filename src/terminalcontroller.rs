use crate::model::{Model, StatusMsg};
use crate::terminalview::TerminalView;
use crate::Controller;
use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;

const QUIT_TIMES: u8 = 3;

pub struct TerminalController<'a> {
    model: Rc<RefCell<Model>>,
    view: &'a TerminalView,
    quit_times: u8,
}

impl<'a> TerminalController<'a> {
    pub fn new(model: Rc<RefCell<Model>>, view: &'a TerminalView) -> TerminalController<'a> {
        TerminalController {
            model: model,
            view: view,
            quit_times: QUIT_TIMES,
        }
    }

    fn save(&self) {}

    fn scroll(&self) {
        let model = &mut self.model.borrow_mut();
        model.rx = 0;

        if model.cy < model.num_rows() {
            model.rx = model.cx_to_rx(model.get_cur_row(), model.cx);
        }
        if model.cy < model.rowoff {
            model.rowoff = model.cy;
        }
        if model.cy >= model.rowoff + self.view.get_screen_rows() {
            model.rowoff = model.cy - self.view.get_screen_rows() + 1;
        }
        if model.rx < model.coloff {
            model.coloff = model.rx;
        }
        if model.rx >= model.coloff + self.view.get_screen_cols() {
            model.coloff = model.rx - self.view.get_screen_cols() + 1;
        }
    }

    fn move_cursor(&self, key: termion::event::Key) {
        let model = &mut self.model.borrow_mut();
        let bounds_exceeded = if model.cy >= model.num_rows() {false} else {true};

        match key {
            Key::Left => {
                if model.cx != 0 {
                    model.cx -= 1;
                }
                else if model.cy > 0 {
                    model.cy -= 1;
                    model.cx = model.cur_row_len();
                }
            }
            Key::Right => {
                if bounds_exceeded && model.cx < model.cur_row_len() {
                    model.cx += 1;
                } else if bounds_exceeded && model.cx == model.cur_row_len() {
                    model.cy += 1;
                    model.cx = 0;
                }
            }
            Key::Up => {
                model.cy = model.cy.saturating_sub(1);
            }
            Key::Down => {
                if model.cy < model.num_rows() {
                    model.cy += 1;
                }
            }
            _ => {return;}            
        }

        let mut rowlen = 0;
        if model.cy < model.num_rows() {
            rowlen = model.cur_row_len();
        }
        if model.cx > rowlen {
            model.cx = rowlen;
        }
    }

    fn delete_char(&self) {
        let mut model = self.model.borrow_mut();
        model.delete_char();
    }

    fn page_down(&self) {}

    fn page_up(&self) {}

    fn insert_char(&self, c: char) {
        let mut model = self.model.borrow_mut();

        model.insert_char(c);
    }

    fn insert_newline(&self) {
        let mut model = self.model.borrow_mut();
        model.insert_newline();
    }

    fn quit(&self) -> u8 {
        let mut model = self.model.borrow_mut();
        let quit_times = if model.dirty == 0 {
            0
        } else {
            self.quit_times - 1
        };
        if quit_times > 0 {
            model.status_msg = StatusMsg::Warn(format!(
                "File has unsaved changes! Quit {} more times to force-quit.",
                quit_times
            ));
        }
        quit_times
    }

    fn abort_quit(&self) {
        let mut model = self.model.borrow_mut();
        model.status_msg = StatusMsg::Normal(String::from(""));
    }
}

impl<'a> Controller for TerminalController<'a> {
    fn process_input(&mut self) -> Result<bool, std::io::Error> {
        let stdin = stdin();
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

        stdout.flush().unwrap();
        for c in stdin.events() {
            let evt = c.unwrap();
            match evt {
                Event::Key(key) => match key {
                    Key::Ctrl('q') => {
                        self.quit_times = self.quit();
                        if self.quit_times == 0 {
                            return Ok(false);
                        }
                        return Ok(true);
                    }
                    Key::Ctrl('s') => {
                        self.save();
                        break;
                    }
                    Key::Esc => {}
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.move_cursor(key);
                        break;
                    }
                    Key::Backspace | Key::Delete | Key::Ctrl('h') => {
                        self.delete_char();
                        break;
                    }
                    Key::PageDown => {
                        self.page_down();
                        break;
                    }
                    Key::PageUp => {
                        self.page_up();
                        break;
                    }
                    Key::Char('\r') | Key::Char('\n') => {
                        self.insert_newline();
                        break;
                    }
                    Key::Char(c) => {
                        self.insert_char(c);
                        break;
                    }
                    Key::Ctrl(_) | Key::Alt(_) => {}
                    _ => {
                        break;
                    }
                },
                // Click will automatically move/set the mouse position
                // On release if the release spot is different, we have a selection
                Event::Mouse(me) => match me {
                    MouseEvent::Press(_, x, y) => {
                        write!(stdout, "{}x", termion::cursor::Goto(x, y)).unwrap();
                        break;
                    }
                    _ => {
                        break;
                    },
                },
                _ => {
                    break;
                }
            }
            stdout.flush().unwrap();
        }
        stdout.flush().unwrap();
        if self.quit_times != QUIT_TIMES {
            self.abort_quit();
            self.quit_times = QUIT_TIMES;
        }
        self.scroll();
        Ok(true)
    }
}
