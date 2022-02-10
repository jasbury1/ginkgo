use core::num;
use std::cell;
use std::io::Stdout;

use crate::display::coordinate::{Coord, Position};
use crate::display::{Cell, CellBlock, Display};
use crate::file::{self, FileState};
use crate::ui::{Component, EventResponse, Rect};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use crossterm::style::Color;
use syntect::html::css_for_theme_with_class_style;

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
    rowoff: usize,
    // The location of the cursor
    cursor: Coord,
    text_selected: bool,
    anchor_start: Coord,
    anchor_end: Coord,
    text_wrap: bool,
    wrap_width: usize,
}

pub enum EditCommand {
    InsertNewline { location: Coord },
    InsertString { location: Coord, contents: String },
    DeleteString { start: Coord, end: Coord },
    InsertChar { location: Coord, c: char },
    DeleteChar { location: Coord },
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
            rowoff: 0,
            cursor: (0, 0),
            text_selected: false,
            anchor_start: (0, 0),
            anchor_end: (0, 0),
            text_wrap: true,
            wrap_width: 0,
        }
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

    fn try_merge_char_cmd(&mut self, new_c: char, new_location: Coord) -> bool {
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

    pub fn wrapped_cursor_coords(&self, width: usize) -> Coord {
        // Subtract 1 from the width since the far right column is the border wall
        let width = width - 1;
        let y = self.cursor.1;
        let x = self.cursor.0;
        let mut result = self.cursor;
        for i in self.rowoff..y {
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
    pub fn get_anchors(&self) -> (Coord, Coord) {
        let anchor_start: Coord;
        let anchor_end: Coord;

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

    fn mouse_down(&mut self, x: u16, y: u16) {
        self.cursor = self.screen_to_text_coords(x as usize, y as usize);
        self.anchor_start = self.cursor;
        self.text_selected = false;
    }

    fn mouse_drag(&mut self, x: u16, y: u16) {
        self.anchor_end = self.screen_to_text_coords(x as usize, y as usize);
        if self.anchor_end.0 != self.anchor_start.0 || self.anchor_end.1 != self.anchor_start.1 {
            self.text_selected = true;
        } else {
            self.text_selected = false;
        }
        self.cursor = self.anchor_end;
    }

    fn screen_to_text_coords(&self, x: usize, y: usize) -> Coord {
        // i is just around to keep incrementing by height
        let mut i: usize = 0;
        // Row is the number of jumps AKA the row in the file we are on
        let mut row: usize = 0;
        // Sum keeps track of the excess of each row
        while i + self.row_height(row) <= y {
            i += self.row_height(row);
            row += 1;
        }
        self.filestate
            .clamp_to_bounds((((y - i) * (self.wrap_width - 1)) + x, row))
    }

    fn row_height(&self, row: usize) -> usize {
        return 1 + ((self.filestate.row_len(row).saturating_sub(1)) / (self.wrap_width - 1));
    }

    /*
    fn draw_file_content(&mut self, bounds: &Rect, cellblock: &mut CellBlock) {
        let mut i = 0;
        let mut j = 0;
        let mut row = self.rowoff;

        // Draw the file contents
        'outer: loop {
            if i >= bounds.height - 1 {
                break 'outer;
            } else if row >= self.filestate.num_rows() {
                cellblock[i][j].c = '~';
                i += 1;
                j = 0;
            } else {
                for (c_idx, c) in self.filestate.get_row_contents(row).chars().enumerate() {
                    if j >= bounds.width - 1 {
                        i += 1;
                        j = 0;
                    }
                    if i >= bounds.height - 1 {
                        break 'outer;
                    }
                    cellblock[i][j].c = c;
                    if self.text_selected {
                        let (start, end) = self.get_anchors();
                        // Check all possible conditions for us to be within a selection region
                        if (row == start.1 && row == end.1 && c_idx >= start.0 && c_idx < end.0)
                            || (row == start.1 && row != end.1 && c_idx >= start.0)
                            || (row == end.1 && row != start.1 && c_idx < end.0)
                            || (row > start.1 && row < end.1)
                        {
                            cellblock[i][j].bg_color = Color::Cyan;
                        }
                    }
                    j += 1;
                }
                i += 1;
                j = 0;
                row += 1;
            }
        }
    }
    */

    fn draw_file_content(&mut self, bounds: &Rect, cellblock: &mut CellBlock) {
        let mut view_row: usize = 0;
        let mut file_row: usize = self.rowoff;
        let num_rows = self.filestate.num_rows();

        // i and j represent a range into a file row content slice
        let mut i = 0;
        let mut j = 0;

        while view_row < bounds.height - 1 {
            // Rows past the end of the file are represented by a "~"
            if file_row >= num_rows {
                let mut cell = Cell::new("~", Color::White, Color::Black);
                cell.text.push_str(&" ".repeat(bounds.width - 2));
                cell.text.push(BORDER);
                cellblock[view_row].push(cell);
            } else {
                let cur_row = self.filestate.get_row_contents(file_row);
                let cur_row_len = cur_row.len();

                // Take the largest slice of the file row we can fit in our view
                j = if cur_row.len() - i > bounds.width - 1 {
                    i + bounds.width - 1
                } else {
                    cur_row_len
                };

                // Check to see if we have to highlight text as part of a selection
                if self.text_selected {
                    let (start, end) = self.get_anchors();
                    // A section is highlighted in the middle of our view row
                    if start.between((i, file_row), (j, file_row))
                        && end.between((i, file_row), (j, file_row))
                    {
                        cellblock[view_row].push(Cell::new(
                            &cur_row[i..start.0],
                            Color::White,
                            Color::Black,
                        ));
                        cellblock[view_row].push(Cell::new(
                            &cur_row[start.0..end.0],
                            Color::Black,
                            Color::Cyan,
                        ));
                        cellblock[view_row].push(Cell::new(
                            &cur_row[end.0..j],
                            Color::White,
                            Color::Black,
                        ));
                    }
                    // The entire view row is highlighted
                    else if !start.after((i, file_row)) && !end.before((j, file_row)) {
                        cellblock[view_row].push(Cell::new(
                            &cur_row[i..j],
                            Color::Black,
                            Color::Cyan,
                        ));
                    }
                    // The beginning of the view row is highlighted
                    else if end.between((i, file_row), (j, file_row)) {
                        cellblock[view_row].push(Cell::new(
                            &cur_row[i..end.0],
                            Color::Black,
                            Color::Cyan,
                        ));
                        cellblock[view_row].push(Cell::new(
                            &cur_row[end.0..j],
                            Color::White,
                            Color::Black,
                        ));
                    }
                    // The end of the view row is highlighted
                    else if start.between((i, file_row), (j, file_row)) {
                        cellblock[view_row].push(Cell::new(
                            &cur_row[i..start.0],
                            Color::White,
                            Color::Black,
                        ));
                        cellblock[view_row].push(Cell::new(
                            &cur_row[start.0..j],
                            Color::Black,
                            Color::Cyan,
                        ));
                    } else {
                        cellblock[view_row].push(Cell::new(
                            &cur_row[i..j],
                            Color::White,
                            Color::Black,
                        ));
                    }
                } else {
                    cellblock[view_row].push(Cell::new(&cur_row[i..j], Color::White, Color::Black));
                }

                let mut cell = Cell::new(
                    &" ".repeat(bounds.width.saturating_sub(1 + (cur_row_len - i))),
                    Color::White,
                    Color::Black,
                );
                cell.text.push(BORDER);
                cellblock[view_row].push(cell);

                // Adjust the start of our next slice based on the last slice we displayed
                if j == cur_row.len() {
                    i = 0;
                    file_row += 1;
                } else {
                    i = j;
                }
            }
            view_row += 1;
        }
    }

    fn draw_file_info(&mut self, bounds: &Rect, cellblock: &mut CellBlock) {
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

        let lstatus = format!(" {} - {} lines {}", filename, lines, modified);
        let rstatus = format!("{} ", extension,);
        let padding = bounds.width.saturating_sub(lstatus.len() + rstatus.len());

        let info_message = format!("{}{}{}", lstatus, " ".repeat(padding), rstatus);
        let mut cell: Cell;
        if info_message.len() > bounds.width {
            cell = Cell::new(&info_message[0..bounds.width], Color::Black, Color::Grey);
        } else {
            cell = Cell::new(&info_message, Color::Black, Color::Grey);
            cell.text
                .push_str(&" ".repeat(bounds.width - info_message.len()));
        }
        cellblock[bounds.height - 1].push(cell);
    }
}

impl Component for FileEditComponent {
    type Message = EditCommand;

    fn send_msg(&mut self, msg: &EditCommand) -> EventResponse {
        self.execute_command(msg);
        EventResponse::RedrawDisplay
    }

    fn handle_event(&mut self, event: Event) -> EventResponse {
        let mut response = EventResponse::NoResponse;
        match event {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(_),
                column,
                row,
                modifiers: _,
            }) => {
                if self.text_selected {
                    response = EventResponse::RedrawDisplay;
                } else {
                    response = EventResponse::MoveCursor;
                }
                self.mouse_down(column, row);
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Drag(_),
                column,
                row,
                modifiers: _,
            }) => {
                self.mouse_drag(column, row);
                response = EventResponse::RedrawDisplay;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => {
                self.move_cursor_up();
                response = EventResponse::MoveCursor;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                self.move_cursor_down();
                response = EventResponse::MoveCursor;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                self.move_cursor_left();
                response = EventResponse::MoveCursor;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                self.move_cursor_right();
                response = EventResponse::MoveCursor;
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
                response = EventResponse::RedrawDisplay;
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
                response = EventResponse::RedrawDisplay;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => {
                self.insert_char(c);
                response = EventResponse::RedrawDisplay;
            }
            _ => {
                response = EventResponse::NoResponse;
            }
        }
        response
    }

    fn draw(&mut self, bounds: &Rect, displ: &mut Display<Stdout>) {
        if bounds.width < 1 {
            return;
        }
        let mut cellblock = Cell::empty_cellblock(bounds.height);

        // Draw the contents of the file we are editing
        self.draw_file_content(bounds, &mut cellblock);
        self.draw_file_info(bounds, &mut cellblock);
        displ.draw(bounds, &cellblock).unwrap();
    }

    fn resize(&mut self, bounds: &Rect) {
        self.wrap_width = bounds.width;
    }
}
