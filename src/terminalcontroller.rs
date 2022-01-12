use crate::command::{CommandState, Command};
use crate::model::{Model, StatusMsg};
use crate::terminalview::TerminalView;
use crate::InputHandler;
use crate::View;
use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;

const QUIT_TIMES: u8 = 3;

type PromptCallback = fn(&mut TerminalController, &str) -> Result<bool, std::io::Error>;

enum PromptType {
    Find,
    Rename,
    Command,
}

enum TerminalMode {
    Normal,
    Insert,
    Prompt(PromptType),
}

pub struct TerminalController<'a> {
    model: Rc<RefCell<Model>>,
    view: &'a TerminalView,
    quit_times: u8,
    mode: TerminalMode,
    states: CommandState
}

impl<'a> TerminalController<'a> {
    pub fn new(model: Rc<RefCell<Model>>, view: &TerminalView) -> TerminalController {
        TerminalController {
            model,
            view,
            quit_times: QUIT_TIMES,
            mode: TerminalMode::Normal,
            states: CommandState::new()
        }
    }

    pub fn process_input_prompt(
        &mut self,
        prompt: String,
        callback: PromptCallback,
    ) -> Result<bool, std::io::Error> {
        let stdin = stdin();
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
        let mut msg = String::from("");

        stdout.flush().unwrap();
        self.view.draw_prompt(&prompt, &msg);

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
                        self.view.draw_prompt(&prompt, &msg);
                    }
                    Key::Char('\r') | Key::Char('\n') => {
                        let result = callback(self, &msg);
                        self.enter_normal_mode();
                        return result;
                    }
                    Key::Char(c) => {
                        msg.push(c);
                        self.view.draw_prompt(&prompt, &msg);
                    }
                    _ => {
                        continue;
                    }
                },
                _ => {
                    continue;
                }
            }
        }
        stdout.flush().unwrap();
        Ok(true)
    }

    fn find_callback<'r, 's>(
        controller: &'r mut TerminalController<'s>,
        term: &str,
    ) -> Result<bool, std::io::Error> {
        let num_rows = controller.model.borrow_mut().num_rows();
        let term_len = term.len();

        let mut occurrences: Vec<(usize, usize)> = vec![];

        // Find and save in our vector the row and column for every occurrance's start
        {
            // model limited to inner scope since it is borrowed for later view-related functions
            let model = &mut controller.model.borrow_mut();
            for i in 0..num_rows {
                model
                    .get_row_contents(i)
                    .match_indices(term)
                    .for_each(|idx| occurrences.push((idx.0, i)));
            }
            // Return if no matches were found
            if occurrences.is_empty() {
                model.status_msg =
                    StatusMsg::Warn(format!("No occurrences found for \'{}\'", term));
                return Ok(true);
            }
            model.status_msg = StatusMsg::Normal("n = next, N = prev".to_string());
        }

        let mut idx = 0;
        loop {
            let stdin = stdin();
            let o = occurrences.get(idx).unwrap();

            // Use limited scope for model
            {
                let model = &mut controller.model.borrow_mut();
                model.anchor_start = (o.0, o.1);
                model.anchor_end = (o.0 + term_len, o.1);
                model.text_selected = true;
                model.set_cursor(o.0, o.1);
            }

            // Model is borrowed immutably for view functions
            controller.scroll();
            controller.view.draw();

            'keys: for c in stdin.keys() {
                match c.unwrap() {
                    // 'n' searches forward
                    Key::Char('n') => {
                        idx = (idx + 1) % occurrences.len();
                        break 'keys;
                    }
                    // 'N' searches backward
                    Key::Char('N') => {
                        idx = if idx == 0 {
                            occurrences.len() - 1
                        } else {
                            idx - 1
                        };
                        break 'keys;
                    }
                    Key::Esc | Key::Ctrl('c') => {
                        controller.model.borrow_mut().status_msg =
                            StatusMsg::Normal(String::from(""));
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }
    }

    fn rename_callback<'r, 's>(
        controller: &'r mut TerminalController<'s>,
        name: &str,
    ) -> Result<bool, std::io::Error> {
        // Model borrow confined to nested scope
        {
            let model = &mut controller.model.borrow_mut();
            model.name_file(name);
        }
        controller.save();
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
                        self.enter_prompt_mode(PromptType::Find);
                        break;
                    }
                    Key::Ctrl('r') => {
                        self.states.execute_redo(&mut self.model.borrow_mut());
                        break; 
                    }
                    Key::Char('u') => {
                        self.states.execute_undo(&mut self.model.borrow_mut());
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
                    Key::Ctrl('S') => {
                        self.enter_prompt_mode(PromptType::Rename);
                        break; 
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
                        self.enter_prompt_mode(PromptType::Find);
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

    fn enter_prompt_mode(&mut self, prompt: PromptType) {
        print!("{}", termion::cursor::BlinkingBar);
        self.model.borrow_mut().mode = 'P';
        self.mode = TerminalMode::Prompt(prompt);
    }

    fn enter_normal_mode(&mut self) {
        print!("{}", termion::cursor::SteadyBlock);
        self.model.borrow_mut().mode = 'N';
        self.mode = TerminalMode::Normal;
    }

    fn save(&mut self) {
        let model = &mut self.model.borrow_mut();
        self.states.reset_change_count();
        model.save_file();
    }

    fn scroll(&self) {
        let model = &mut self.model.borrow_mut();
        let screenrows = TerminalView::get_screen_rows();

        // If our cursor went above the view, scroll up
        if model.cy < model.rowoff {
            model.rowoff = model.cy;
        }
        // If our cursor is below the view, ccroll down
        if model.cy >= model.rowoff + screenrows {
            model.rowoff = model.cy - screenrows + 1;
        }
        // If cursor is off-screen to the left, scroll left
        if model.cx < model.coloff {
            model.coloff = model.cx;
        }
        // If cursor is off-screen to the right, scroll right
        if model.cx >= model.coloff + screenrows {
            model.coloff = model.cx - screenrows + 1;
        }
    }

    fn move_cursor(&self, key: termion::event::Key) {
        let model = &mut self.model.borrow_mut();

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
                if (model.cy < model.num_rows()) && model.cx < model.cur_row_len() {
                    model.cx += 1;
                } else if (model.cy < model.num_rows()) && model.cx == model.cur_row_len() {
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

    fn delete(&mut self) {
        let model = &mut self.model.borrow_mut();
        if model.text_selected {
            let (anchor_start, anchor_end) = model.get_anchors();
            self.states.execute_command(Command::DeleteString{start: anchor_start, end: anchor_end}, model);
        } else {
            self.states.execute_command(Command::DeleteChar{ location: (model.cx, model.cy) }, model);
        }
        model.text_selected = false;
    }

    fn page_down(&self) {}

    fn page_up(&self) {}

    fn insert_char(&mut self, c: char) {
        let model = &mut self.model.borrow_mut();
        if model.text_selected {
            let (anchor_start, anchor_end) = model.get_anchors();
            self.states.execute_command(Command::DeleteString{start: anchor_start, end: anchor_end}, model);
        }
        self.states.execute_command(Command::InsertChar{ location: (model.cx, model.cy), c }, model);
        model.text_selected = false;
    }

    fn insert_newline(&mut self) {
        let model = &mut self.model.borrow_mut();
        self.states.execute_command(Command::InsertNewline{ location: (model.cx, model.cy)}, model)
    }

    fn quit(&self) -> u8 {
        let mut model = self.model.borrow_mut();
        let quit_times = if !model.dirty {
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
        // Model is 'dirty' if we have unsaved changes
        self.model.borrow_mut().dirty = self.states.change_count != 0;

        // Process input based on the mode we are in
        match &self.mode {
            TerminalMode::Normal => self.process_input_normal(),
            TerminalMode::Insert => self.process_input_insert(),
            TerminalMode::Prompt(p) => match p {
                PromptType::Find => self.process_input_prompt(
                    String::from("Find:"),
                    TerminalController::find_callback,
                ),
                PromptType::Command => self.process_input_prompt(
                    String::from("Name file:"),
                    TerminalController::rename_callback,
                ),
                PromptType::Rename => todo!()
            },
        }
    }
}
