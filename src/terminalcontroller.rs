use crate::model::{Model, StatusMsg};
use crate::terminalview::TerminalView;
use crate::InputHandler;
use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};

const QUIT_TIMES: u8 = 3;

enum TerminalMode {
    Normal,
    Insert,
}

pub struct TerminalController<'a> {
    model: Rc<RefCell<Model>>,
    view: &'a TerminalView,
    quit_times: u8,
    mode: TerminalMode,
}

impl<'a> TerminalController<'a> {
    pub fn new(model: Rc<RefCell<Model>>, view: &'a TerminalView) -> TerminalController<'a> {
        TerminalController {
            model,
            view,
            quit_times: QUIT_TIMES,
            mode: TerminalMode::Normal
        }
    }

    pub fn process_input_normal(&mut self) -> Result<bool, std::io::Error> {
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
                    Key::Backspace | Key::Delete | Key::Ctrl('h') | Key::Char('h') => {
                        self.move_cursor(Key::Left);
                        break;
                    }
                    Key::Char('j') => {
                        self.move_cursor(Key::Down);
                        break;
                    }
                    Key::Char('k') => {
                        self.move_cursor(Key::Up);
                        break;
                    }
                    Key::Char('l') => {
                        self.move_cursor(Key::Right);
                        break;
                    }
                    Key::Char('i') => {
                        print!("{}", termion::cursor::BlinkingBar);
                        self.model.borrow_mut().mode = 'I';
                        self.mode = TerminalMode::Insert;
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
                        self.move_cursor(Key::Down);
                        break;
                    }
                    
                    Key::Ctrl(_) | Key::Alt(_) => {}
                    _ => {
                        break;
                    }
                },
                // Mouse indices are 1-based so we subtract 1 to make 0-based
                Event::Mouse(me) => match me {
                    MouseEvent::Press(_, x, y) => {
                        self.mouse_press(x - 1, y - 1);
                        break;
                    }
                    MouseEvent::Hold(x, y) => {
                        self.mouse_hold(x - 1, y - 1);
                        break;
                    }
                    MouseEvent::Release(x, y) => {
                        self.mouse_release();
                        break;
                    }
                    _ => {
                        break;
                    }
                },
                Event::Unsupported(_) => todo!(),
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

    
    pub fn process_input_insert(&mut self) -> Result<bool, std::io::Error> {
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
                    Key::Esc => {
                        print!("{}", termion::cursor::SteadyBlock);
                        self.model.borrow_mut().mode = 'N';
                        self.mode = TerminalMode::Normal;
                        break;
                    }
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
                // Mouse indices are 1-based so we subtract 1 to make 0-based
                Event::Mouse(me) => match me {
                    MouseEvent::Press(_, x, y) => {
                        self.mouse_press(x - 1, y - 1);
                        break;
                    }
                    MouseEvent::Hold(x, y) => {
                        self.mouse_hold(x - 1, y - 1);
                        break;
                    }
                    MouseEvent::Release(x, y) => {
                        self.mouse_release();
                        break;
                    }
                    _ => {
                        break;
                    }
                },
                Event::Unsupported(_) => todo!(),
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
        let bounds_exceeded = if model.cy >= model.num_rows() {
            false
        } else {
            true
        };

        model.text_selected = false;

        match key {
            Key::Left => {
                if model.cx != 0 {
                    model.cx -= 1;
                } else if model.cy > 0 {
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
            _ => {
                return;
            }
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

    fn mouse_press(&mut self, x: u16, y: u16) {
        let (cx, cy) = self.screen_to_model_coords(x, y);
        let mut model = self.model.borrow_mut();

        model.anchor_start = (cx, cy);
        model.text_selected = false;

        model.cx = cx;
        model.cy = cy;
    }

    fn mouse_hold(&mut self, x: u16, y: u16) {
        let (cx, cy) = self.screen_to_model_coords(x, y);
        let mut model = self.model.borrow_mut();

        model.anchor_end = (cx, cy);

        if cx != model.anchor_start.0 || cy != model.anchor_start.1 {
            model.text_selected = true;
        } else {
            model.text_selected = false;
        }

        model.cx = cx;
        model.cy = cy;
    }

    fn mouse_release(&mut self) {}

    fn screen_to_model_coords(&self, x: u16, y: u16) -> (usize, usize) {
        let model = self.model.borrow();
        let mut cx = model.coloff + (x as usize);
        let mut cy = model.rowoff + (y as usize);

        let num_rows = model.num_rows();

        if cy > num_rows {
            cy = num_rows;
        }

        if cy == num_rows {
            cx = 0;
        } else {
            let len = model.row_len(cy);
            if cx > len {
                cx = len;
            }
        }
        (cx, cy)
    }
}

impl<'a> InputHandler for TerminalController<'a> {
    fn process_input(&mut self) -> Result<bool, std::io::Error> {
        match self.mode {
            TerminalMode::Normal => {
                self.process_input_normal()
            }
            TerminalMode::Insert => {
                self.process_input_insert()
            }
        }
    }
}
