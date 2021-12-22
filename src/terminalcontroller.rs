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

    fn move_cursor(&self, key: termion::event::Key) {}

    fn delete_char(&self) {}

    fn page_down(&self) {}

    fn page_up(&self) {}

    fn insert_char(&self, c: char) {}

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
                    }
                    Key::Esc => {}
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.move_cursor(key);
                    }
                    Key::Backspace | Key::Delete | Key::Ctrl('h') => {
                        self.delete_char();
                    }
                    Key::PageDown => {
                        self.page_down();
                    }
                    Key::PageUp => {
                        self.page_up();
                    }
                    Key::Char('\r') => {
                        self.insert_newline();
                    }
                    Key::Char(c) => {
                        self.insert_char(c);
                    }
                    Key::Ctrl(_) | Key::Alt(_) => {}
                    _ => {}
                },
                // Click will automatically move/set the mouse position
                // On release if the release spot is different, we have a selection
                Event::Mouse(me) => match me {
                    MouseEvent::Press(_, x, y) => {
                        write!(stdout, "{}x", termion::cursor::Goto(x, y)).unwrap();
                    }
                    _ => (),
                },
                _ => {}
            }
            stdout.flush().unwrap();
        }
        if self.quit_times != QUIT_TIMES {
            self.abort_quit();
            self.quit_times = QUIT_TIMES;
        }
        Ok(true)
    }
}
