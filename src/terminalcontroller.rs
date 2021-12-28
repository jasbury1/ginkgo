use crate::model::{Model, StatusMsg};
use crate::terminalview::TerminalView;
use crate::InputHandler;
use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;

const QUIT_TIMES: u8 = 3;

type PromptCallback = fn(&mut TerminalController, String) -> Result<bool, std::io::Error>;

enum TerminalMode {
    Normal,
    Insert,
    Prompt,
}

pub struct TerminalController<'a> {
    model: Rc<RefCell<Model>>,
    view: &'a TerminalView,
    quit_times: u8,
    mode: TerminalMode,
}

impl<'a> TerminalController<'a> {
    pub fn new(model: Rc<RefCell<Model>>, view: &TerminalView) -> TerminalController {
        TerminalController {
            model,
            view,
            quit_times: QUIT_TIMES,
            mode: TerminalMode::Normal,
        }
    }

    pub fn process_input_prompt(&mut self, prompt: &String, callback: PromptCallback) -> Result<bool, std::io::Error> {
        let stdin = stdin();
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
        let mut msg = String::from("");

        let test = &Self::test_callback;

        stdout.flush().unwrap();
        self.view.draw_prompt(prompt, &msg);

        for c in stdin.events() {
            let evt = c.unwrap();
            match evt {
                Event::Key(key) => match key {
                    Key::Esc | Key::Ctrl('c') => {
                        self.enter_normal_mode();
                        return Ok(true);
                    }
                    Key::Backspace | Key::Delete | Key::Ctrl('h') => {
                        msg.pop();
                        self.view.draw_prompt(prompt, &msg);
                    }
                    Key::Char('\r') | Key::Char('\n') => {
                        todo!();
                        break;
                    }
                    Key::Char(c) => {
                        msg.push(c);
                        self.view.draw_prompt(prompt, &msg);
                    }
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
                    MouseEvent::Release(_, _) => {
                        self.mouse_release();
                        break;
                    }
                },
                Event::Unsupported(_) => todo!(),
            }
        }
        stdout.flush().unwrap();
        
        Ok(true)
    }

    fn test_callback<'r, 's>(controller: &'r mut TerminalController<'s>, msg: String) -> Result<bool, std::io::Error> {
        Ok(true)
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
                    Key::Ctrl('f') => {
                        self.enter_prompt_mode();
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
                        self.enter_insert_mode();
                        break;
                    }
                    Key::Char('I') => {
                        self.enter_insert_mode();
                        self.goto_line_start();
                        break;
                    }
                    Key::Char('a') => {
                        self.enter_insert_mode();
                        self.move_cursor(Key::Right);
                        break;
                    }
                    Key::Char('A') => {
                        self.enter_insert_mode();
                        self.goto_line_end();
                        break;
                    }
                    Key::Char('o') => {
                        self.enter_insert_mode();
                        self.goto_line_end();
                        self.insert_newline();
                        break;
                    }
                    Key::Char('O') => {
                        self.enter_insert_mode();
                        self.goto_line_start();
                        self.insert_newline();
                        self.move_cursor(Key::Up);
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
                    MouseEvent::Release(_, _) => {
                        self.mouse_release();
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
                    Key::Esc | Key::Ctrl('c') => {
                        self.enter_normal_mode();
                        break;
                    }
                    Key::Ctrl('f') => {
                        self.enter_prompt_mode();
                        break;
                    }
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.move_cursor(key);
                        break;
                    }
                    Key::Backspace | Key::Delete | Key::Ctrl('h') => {
                        self.delete();
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
                    MouseEvent::Release(_, _) => {
                        self.mouse_release();
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

    fn enter_insert_mode(&mut self) {
        print!("{}", termion::cursor::BlinkingBar);
        self.model.borrow_mut().mode = 'I';
        self.mode = TerminalMode::Insert;
    }

    fn enter_prompt_mode(&mut self) {
        print!("{}", termion::cursor::BlinkingBar);
        self.model.borrow_mut().mode = 'P';
        self.mode = TerminalMode::Prompt;
    }

    fn enter_normal_mode(&mut self) {
        print!("{}", termion::cursor::SteadyBlock);
        self.model.borrow_mut().mode = 'N';
        self.mode = TerminalMode::Normal;
    }

    fn save(&self) {
        let model = &mut self.model.borrow_mut();
        model.save_file();
    }

    fn scroll(&self) {
        let model = &mut self.model.borrow_mut();
        model.rx = 0;

        if model.cy < model.num_rows() {
            model.rx = model.cx_to_rx(model.get_cur_row(), model.cx);
        }
        if model.cy < model.rowoff {
            model.rowoff = model.cy;
        }
        if model.cy >= model.rowoff + TerminalView::get_screen_rows() {
            model.rowoff = model.cy - TerminalView::get_screen_rows() + 1;
        }
        if model.rx < model.coloff {
            model.coloff = model.rx;
        }
        if model.rx >= model.coloff + TerminalView::get_screen_cols() {
            model.coloff = model.rx - TerminalView::get_screen_cols() + 1;
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

    fn goto_line_end(&self) {
        let mut model = self.model.borrow_mut();
        let len = model.cur_row_len();
        model.cx = len;
    }

    fn goto_line_start(&self) {
        let mut model = self.model.borrow_mut();
        model.cx = 0;
    }

    fn delete(&self) {
        let mut model = self.model.borrow_mut();
        if model.text_selected {
            model.delete_selection();
        } else {
            model.delete_char();
        }
        model.text_selected = false;
    }

    fn page_down(&self) {}

    fn page_up(&self) {}

    fn insert_char(&self, c: char) {
        let mut model = self.model.borrow_mut();
        if model.text_selected {
            model.delete_selection();
        }
        model.insert_char(c);
        model.text_selected = false;
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
        let temp = String::from("Find: ");
        match self.mode {
            TerminalMode::Normal => self.process_input_normal(),
            TerminalMode::Insert => self.process_input_insert(),
            TerminalMode::Prompt => self.process_input_prompt(&temp, Self::test_callback)
        }
    }
}
