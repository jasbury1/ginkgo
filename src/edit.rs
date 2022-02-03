use std::cell;

use crate::display::{Cell, CellBlock, Display};
use crate::file::FileState;
use crate::ui::{Component, Rect};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;

const BORDER: char = '┃';

pub struct FileEditComponent {
    // The state of the file we are editing
    filestate: FileState,
    // Vec of the command steps for each undo
    undo_commands: Vec<EditCommand>,
    // Number of simultaneous steps to perform a single undo
    undo_steps: Vec<usize>,
    // Vec of the command steps for each redo
    redo_commands: Vec<EditCommand>,
    // Number of simultaneous steps to perform a single redo
    redo_steps: Vec<usize>,
    // Number of unsaved changes. Can be negative for unsaved undos
    pub change_count: i32,
    // The column offset for how far down this view starts
    coloff: usize,
    // The location of the cursor
    cursor: (usize, usize),
    text_selected: bool,
    anchor_start: (usize, usize),
    anchor_end: (usize, usize),
    cell_cache: Option<CellBlock>,
}

pub enum EditCommand {
    InsertNewline {
        location: (usize, usize),
    },
    InsertString {
        location: (usize, usize),
        contents: String,
    },
    DeleteString {
        start: (usize, usize),
        end: (usize, usize),
    },
    InsertChar {
        location: (usize, usize),
        c: char,
    },
    DeleteChar {
        location: (usize, usize),
    },
}

impl FileEditComponent {
    pub fn new(fs: FileState) -> Self {
        FileEditComponent {
            filestate: fs,
            undo_commands: Vec::new(),
            undo_steps: Vec::new(),
            redo_commands: Vec::new(),
            redo_steps: Vec::new(),
            change_count: 0,
            coloff: 0,
            cursor: (0, 0),
            text_selected: false,
            anchor_start: (0, 0),
            anchor_end: (0, 0),
            cell_cache: None,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => {
                self.move_cursor_up();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                self.move_cursor_down();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                self.move_cursor_left();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                self.move_cursor_right();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            })
            | Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('h'),
            }) => {
                self.delete();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('\n'),
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('\r'),
                ..
            }) => {
                self.insert_newline();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => {
                self.insert_char(c);
            }
            _ => {}
        }
        //TODO: We assume any change will invalidate the cell cache
        self.cell_cache = None;
    }

    pub fn execute_command(&mut self, cmd: &EditCommand) {
        // After a change, unsaved undos count positively
        if self.change_count < 0 {
            self.change_count *= -1;
        }

        let undo_cmd = self.execute_edit_command(&cmd);
        self.redo_commands.clear();
        self.redo_steps.clear();

        // Attempt to merge this command with existing commands, and return early if we can
        if let EditCommand::InsertChar { location, c } = &cmd {
            if self.try_merge_char_cmd(*c, *location) {
                return;
            }
        }

        self.undo_commands.push(undo_cmd);
        self.undo_steps.push(1);

        self.change_count += 1;
    }

    fn try_merge_char_cmd(&mut self, new_c: char, new_location: (usize, usize)) -> bool {
        if !new_c.is_alphabetic() {
            return false;
        }

        // We can merge strings with extra consecutive characters into a longer string
        if let Some(EditCommand::DeleteString { start: _, end }) = self.undo_commands.last_mut() {
            if end.0 != new_location.0 || end.1 != new_location.1 {
                return false;
            }
            end.0 += 1;
            return true;
        }
        // We can merge consecutive characters into a string
        else if let Some(EditCommand::DeleteChar { location }) = self.undo_commands.last_mut() {
            let c = self.filestate.get_char(*location);
            // Can only merge consecutive alphabetic characters
            if !c.is_alphabetic() {
                return false;
            }
            if location.0 != new_location.0 || location.1 != new_location.1 {
                return false;
            }
            let cmd = EditCommand::DeleteString {
                start: (location.0 - 1, location.1),
                end: (location.0 + 1, location.1),
            };
            self.undo_commands.pop();
            self.undo_commands.push(cmd);
            return true;
        }
        // We cannot merge any other commands
        false
    }

    pub fn execute_undo(&mut self) {
        // Function becomes noop if undo_steps is empty
        if let Some(len) = self.undo_steps.pop() {
            // Execute one or more undo moves that should happen at once
            for _ in 0..len {
                let cmd = self.undo_commands.pop().unwrap();
                let redo_cmd = self.execute_edit_command(&cmd);
                self.redo_commands.push(redo_cmd);
            }
            self.redo_steps.push(len);

            self.change_count -= 1;
        }
    }

    pub fn execute_redo(&mut self) {
        if let Some(len) = self.redo_steps.pop() {
            for _ in 0..len {
                let cmd = self.redo_commands.pop().unwrap();
                let undo_cmd = self.execute_edit_command(&cmd);
                self.undo_commands.push(undo_cmd);
            }
            self.undo_steps.push(len);

            self.change_count += 1;
        }
    }

    pub fn reset_change_count(&mut self) {
        self.change_count = 0;
    }

    pub fn wrapped_cursor_coords(&self, width: usize) -> (usize, usize) {
        // Subtract 1 from the width since the far right column is the border wall
        let width = width - 1;
        let y = self.cursor.1;
        let x = self.cursor.0;
        let mut result = self.cursor;
        for i in self.coloff..y {
            result.1 += self.filestate.row_len(i) / width;
        }
        result.1 += x / width;
        result.0 = x % width;

        result
    }

    pub fn execute_edit_command(&mut self, cmd: &EditCommand) -> EditCommand {
        let fs = &mut self.filestate;
        match cmd {
            EditCommand::InsertNewline { location } => {
                self.cursor = fs.insert_newline(*location);
                EditCommand::DeleteChar {
                    location: self.cursor,
                }
            }
            EditCommand::InsertString { location, contents } => {
                self.cursor = fs.insert_string(&contents, *location);
                EditCommand::DeleteString {
                    start: *location,
                    end: self.cursor,
                }
            }
            EditCommand::DeleteString { start, end } => {
                let selection: String = fs.get_selection(*start, *end);
                self.cursor = fs.delete_selection(*start, *end);
                EditCommand::InsertString {
                    location: *start,
                    contents: selection,
                }
            }
            EditCommand::InsertChar { location, c } => {
                self.cursor = fs.insert_char(*c, *location);
                EditCommand::DeleteChar {
                    location: self.cursor,
                }
            }
            EditCommand::DeleteChar { location } => {
                let chr: char = fs.get_char(*location);
                self.cursor = fs.delete_char(*location);
                if chr == '\n' {
                    EditCommand::InsertNewline {
                        location: (self.cursor),
                    }
                } else {
                    EditCommand::InsertChar {
                        location: (self.cursor),
                        c: chr,
                    }
                }
            }
        }
    }

    /// Returns two anchor points as a pair of two tuples of coordinate usize values.
    /// These anchors were set when a selection was made for the model.
    /// This function always returns them in non-descending order so that you can safely assume
    /// the second pair of points returned is not before the first pair of points. Do not make
    /// assumptions about which point is the model's start_anchor value and which point is
    /// the model's end_anchor point.
    pub fn get_anchors(&self) -> ((usize, usize), (usize, usize)) {
        let anchor_start: (usize, usize);
        let anchor_end: (usize, usize);

        // Start should always be before end. Swap if necessary
        if (self.anchor_end.1 < self.anchor_start.1)
            || (self.anchor_start.1 == self.anchor_end.1 && self.anchor_start.0 > self.anchor_end.0)
        {
            anchor_start = self.anchor_end;
            anchor_end = self.anchor_start;
        } else {
            anchor_start = self.anchor_start;
            anchor_end = self.anchor_end;
        }
        (anchor_start, anchor_end)
    }

    pub fn invalidate_cell_cache(&mut self) {
        self.cell_cache = None;
    }

    fn move_cursor_up(&mut self) {
        self.text_selected = false;
        self.cursor.1 = self.cursor.1.saturating_sub(1);
        let rowlen = self.filestate.row_len(self.cursor.1);
        if self.cursor.0 > rowlen {
            self.cursor.0 = rowlen;
        }
    }

    fn move_cursor_down(&mut self) {
        self.text_selected = false;
        let num_rows = self.filestate.num_rows();
        if self.cursor.1 < num_rows {
            self.cursor.1 += 1;
        }
        let rowlen = self.filestate.row_len(self.cursor.1);
        if self.cursor.1 == num_rows {
            self.cursor.0 = 0;
        } else if self.cursor.0 > rowlen {
            self.cursor.0 = rowlen;
        }
    }

    fn move_cursor_left(&mut self) {
        self.text_selected = false;
        if self.cursor.0 != 0 {
            self.cursor.0 -= 1;
        } else if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
            self.cursor.0 = self.filestate.row_len(self.cursor.1);
        }
    }

    fn move_cursor_right(&mut self) {
        self.text_selected = false;
        if (self.cursor.1 < self.filestate.num_rows())
            && self.cursor.0 < self.filestate.row_len(self.cursor.1)
        {
            self.cursor.0 += 1;
        } else if (self.cursor.1 < self.filestate.num_rows())
            && self.cursor.0 == self.filestate.row_len(self.cursor.1)
        {
            self.cursor.1 += 1;
            self.cursor.0 = 0;
        }
    }

    fn goto_line_end(&mut self) {
        if self.cursor.1 > self.filestate.num_rows() {
            return;
        }
        let len = self.filestate.row_len(self.cursor.1);
        self.cursor.0 = len;
    }

    fn goto_line_start(&mut self) {
        self.cursor.0 = 0;
    }

    fn delete(&mut self) {
        if self.text_selected {
            let (anchor_start, anchor_end) = self.get_anchors();
            self.execute_command(&EditCommand::DeleteString {
                start: anchor_start,
                end: anchor_end,
            });
        } else {
            // if we past the end of the file, only move the cursor. No state change
            if self.cursor.1 >= self.filestate.num_rows() {
                let rowlen = if self.cursor.1 > 0 {
                    self.filestate.row_len(self.cursor.1 - 1)
                } else {
                    0
                };
                self.cursor.1 = self.cursor.1.saturating_sub(1);
                self.cursor.0 = rowlen;
                return;
            }
            self.execute_command(&EditCommand::DeleteChar {
                location: self.cursor,
            });
        }
        self.text_selected = false;
    }

    fn insert_char(&mut self, c: char) {
        if self.text_selected {
            let (anchor_start, anchor_end) = self.get_anchors();
            self.execute_command(&EditCommand::DeleteString {
                start: anchor_start,
                end: anchor_end,
            });
        }
        self.execute_command(&EditCommand::InsertChar {
            location: self.cursor,
            c,
        });
        self.text_selected = false;
    }

    fn insert_newline(&mut self) {
        self.execute_command(&EditCommand::InsertNewline {
            location: self.cursor,
        });
    }
}

impl Component for FileEditComponent {
    type Message = EditCommand;

    fn send_msg(&mut self, msg: &EditCommand) {
        self.execute_command(msg);
    }

    fn draw(&mut self, bounds: &Rect, displ: &mut Display) {
        if bounds.width < 1 {
            return;
        }
        // Shortcut if we have a cached version of the cells
        if let Some(cells) = &self.cell_cache {
            displ.draw(bounds, cells);
            return;
        }
        self.cell_cache = Some(Cell::empty_cellblock(bounds.width, bounds.height));
        let cellblock = self.cell_cache.as_mut().unwrap();

        let mut i = 0;
        let mut j = 0;
        let mut row = self.coloff;

        // Draw the file contents
        'outer: loop {
            if i >= bounds.height - 1 {
                break 'outer;
            } else if row >= self.filestate.num_rows() {
                cellblock[i][j].c = '~';
                i += 1;
                j = 0;
            } else {
                for c in self.filestate.get_row_contents(row).chars() {
                    if j >= bounds.width - 1 {
                        i += 1;
                        j = 0;
                    }
                    if i >= bounds.height - 1 {
                        break 'outer;
                    }
                    cellblock[i][j].c = c;
                    j += 1;
                }
                i += 1;
                j = 0;
                row += 1;
            }
        }
        // Draw the info bar at the bottom
        let filename = {
            if self.filestate.filename.is_empty() {
                "[No name]"
            } else {
                &self.filestate.filename
            }
        };

        let modified = {
            if self.change_count != 0 {
                "(modified)"
            } else {
                ""
            }
        };

        let extension = {
            if self.filestate.ext.is_empty() {
                "Plaintext"
            } else {
                &self.filestate.ext
            }
        };

        let lines = self.filestate.num_rows();

        let lstatus = format!("{} - {} lines {}", filename, lines, modified);
        let rstatus = format!("{} ", extension,);
        let padding = bounds.width.saturating_sub(lstatus.len() + rstatus.len());

        let mut info_message = format!("{}{}{}", lstatus, " ".repeat(padding), rstatus);
        if info_message.len() > bounds.width {
            info_message.truncate(bounds.width);
        }

        for (i, c) in info_message.chars().enumerate() {
            cellblock[bounds.height - 1][i].c = c;
            cellblock[bounds.height - 1][i].bg_color = Color::Grey;
            cellblock[bounds.height - 1][i].text_color = Color::Black;
        }

        // Draw border wall
        for i in 0..bounds.height - 1 {
            cellblock[i][bounds.width - 1].c = BORDER;
        }

        displ.draw(bounds, &cellblock);
    }
}
