use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct Erow {
    idx: usize,
    contents: String,
}

pub enum StatusMsg {
    Normal(String),
    Warn(String),
    Error(String),
}

#[allow(dead_code)]
pub struct Model {
    pub cx: usize,
    pub cy: usize,
    pub rowoff: usize,
    pub coloff: usize,
    pub filename: String,
    pub path: PathBuf,
    pub ext: String,
    pub status_msg: StatusMsg,

    pub anchor_start: (usize, usize),
    pub anchor_end: (usize, usize),
    pub text_selected: bool,

    pub mode: char,

    pub dirty: bool,

    rows: Vec<Erow>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            cx: 0,
            cy: 0,
            rowoff: 0,
            coloff: 0,
            rows: vec![],
            path: PathBuf::new(),
            filename: String::from(""),
            ext: String::from(""),
            status_msg: StatusMsg::Normal(String::from(
                "HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find",
            )),
            anchor_start: (0, 0),
            anchor_end: (0, 0),
            text_selected: false,
            mode: 'N',
            dirty: false
        }
    }

    pub fn open_file(&mut self, input_path: &str) {
        self.path = PathBuf::from(input_path);

        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.path.clone());
        let reader: BufReader<File>;

        match f {
            Ok(file) => {
                reader = BufReader::new(file);
            }
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    self.status_msg =
                        StatusMsg::Error(format!("Unable to create file {:?}.", input_path));
                    return;
                }
                ErrorKind::PermissionDenied => {
                    self.status_msg = StatusMsg::Error(format!(
                        "Unable to open {:?}. Permission denied.",
                        input_path
                    ));
                    return;
                }
                other_error => {
                    self.status_msg = StatusMsg::Error(format!(
                        "Problem opening file {:?}. {:?}.",
                        input_path, other_error
                    ));
                    return;
                }
            },
        };

        for line_ in reader.lines() {
            let line = line_.unwrap();
            self.append_row(line);
        }
        self.filename = self.path.file_name().unwrap().to_str().unwrap().to_string();
        self.ext = self
            .path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
    }

    pub fn save_file(&mut self) {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.path.clone());

        let mut bytes: usize = 0;
        println!("{}", self.num_rows());
        match f {
            Ok(mut file) => {
                for row in self.rows.iter() {
                    let contents = &row.contents;
                    bytes += file.write(contents.as_bytes()).unwrap();
                    bytes += file.write(b"\n").unwrap();
                }
                self.status_msg = StatusMsg::Normal(format!("{} bytes written to disk.", bytes));
            }
            Err(err) => {
                self.status_msg =
                    StatusMsg::Error(format!("Unable to write to {}: {:?}.", self.filename, err));
            }
        }
    }

    //TODO: Will do the same as name_file, except deletes the old file with the old name
    // Should probably call 'save' too...
    pub fn rename_file(&mut self, new_name: &str) {
        todo!()
    }

    pub fn name_file(&mut self, new_name: &str) {
        let filename = OsStr::new(new_name);
        self.path.set_file_name(filename);
        self.filename = self.path.file_name().unwrap().to_str().unwrap().to_string();
        self.ext = self
            .path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
    }

    ///
    fn append_row(&mut self, line: String) {
        let num_rows = self.num_rows();

        let row = Erow {
            idx: num_rows,
            contents: line,
        };

        self.rows.insert(num_rows, row);
    }

    ///
    fn insert_row(&mut self, idx: usize, line: &str) {
        let num_rows = self.num_rows();
        if idx > num_rows {
            return;
        }

        let row = Erow {
            idx,
            contents: line.to_string(),
        };

        for i in idx..num_rows {
            self.rows.get_mut(i).unwrap().idx += 1;
        }

        self.rows.insert(idx, row);
    }

    ///
    pub fn insert_newline(&mut self) {
        let cur_row = self.rows.get_mut(self.cy).unwrap();
        let cur_row_len = cur_row.contents.len();

        if self.cx == 0 {
            self.insert_row(self.cy, "");
        } else if self.cx == cur_row_len {
            self.insert_row(self.cy + 1, "");
        } else {
            let leftover = String::from(&cur_row.contents[self.cx..]);
            self.insert_row(self.cy + 1, &leftover);
            self.rows
                .get_mut(self.cy)
                .unwrap()
                .contents
                .truncate(self.cx);
        }
        self.cy += 1;
        self.cx = 0;
    }

    ///
    pub fn insert_char(&mut self, c: char) {
        let num_rows = self.num_rows();
        if self.cy == num_rows {
            self.insert_row(num_rows, "");
        }

        let cur_row = self.rows.get_mut(self.cy).unwrap();
        let mut at = self.cx;

        if at > cur_row.contents.len() {
            at = cur_row.contents.len()
        }
        cur_row.contents.insert(at, c);

        self.cx += 1;
    }

    /// Insert the string into the document at the current cursor XY position.
    /// Newlines inside the input string get separated out into individual
    /// rows that will be reflected in the document.
    ///
    /// # Arguments
    ///
    /// * `contents` - The string to insert
    ///
    pub fn insert_string(&mut self, contents: &str) {
        // Initialize the buffer to the current line prior to the cursor
        let mut buffer: String =
            (&self.rows.get(self.cy).unwrap().contents[0..self.cx]).to_string();
        // Add the contents we are pushing
        buffer.push_str(contents);
        // Add the end after the cursor
        buffer.push_str(&self.rows.get(self.cy).unwrap().contents[self.cx..]);

        let mut idx = self.cy;
        self.delete_row(idx);

        // Insert each line as a new row, deliminating by newline characters in the buffer
        let mut line_len = 0;
        for line in buffer.split('\n') {
            line_len = line.len();
            self.insert_row(idx, line);
            idx += 1;
        }

        // Subtract 1 from idx since it was incremented at the end of the for loop
        idx -= 1;

        // Move the cursor to the end of the string we inserted
        if idx == self.cy {
            // If the insertion was within the same line, just move cx forward
            self.cx += contents.len();
        } else {
            // If we ended up adding additional lines, adjust cx and cy
            self.cx = line_len;
            self.cy = idx;
        }
    }

    pub fn delete_row(&mut self, row_idx: usize) {
        self.rows.remove(row_idx);

        let num_rows = self.num_rows();
        for i in row_idx..num_rows {
            self.rows.get_mut(i).unwrap().idx -= 1;
        }
    }

    pub fn delete_rows(&mut self, row_idx: usize, num_removed: usize) {
        for _ in 0..num_removed {
            self.rows.remove(row_idx);
        }
        let num_rows = self.num_rows();
        for i in row_idx..num_rows {
            self.rows.get_mut(i).unwrap().idx -= num_removed;
        }
    }

    pub fn delete_char(&mut self) {
        let num_rows = self.num_rows();

        if self.cy == num_rows {
            return;
        }
        if self.cx == 0 && self.cy == 0 {
            return;
        }

        if self.cx > 0 {
            let cur_row = &mut self.rows.get_mut(self.cy).unwrap().contents;
            if self.cx > cur_row.len() {
                return;
            }
            cur_row.remove(self.cx.saturating_sub(1));
            self.cx -= 1;
        } else {
            let cur_row = self.rows.get(self.cy).unwrap().contents.clone();
            let prev_row = &mut self.rows.get_mut(self.cy - 1).unwrap().contents;
            self.cx = prev_row.len();
            prev_row.push_str(&cur_row);
            self.delete_row(self.cy);
            self.cy -= 1;
        }
    }

    /// Returns the character the cursor is pointing at, or a newline
    /// character if the cursor is pointing to the beginning of the line
    pub fn get_char(&self) -> char {
        let cur_row = &self.rows.get(self.cy).unwrap().contents;
        if self.cx == 0 {
            '\n'
        } else {
            cur_row.chars().nth(self.cx - 1).unwrap_or_default()
        }
    }

    pub fn get_char_at(&mut self, location: (usize, usize)) -> char {
        let row = &self.rows.get(location.1).unwrap().contents;
        if location.0 == 0 {
            '\n'
        } else {
            row.chars().nth(location.0 - 1).unwrap_or_default()
        }
    }

    pub fn delete_selection(&mut self) {
        let (anchor_start, anchor_end) = self.get_anchors();

        let end_row = self.rows.get(anchor_end.1).unwrap().contents.clone();
        let start_row = &mut self.rows.get_mut(anchor_start.1).unwrap().contents;
        start_row.truncate(anchor_start.0);
        start_row.push_str(&end_row[anchor_end.0..]);

        // Delete the complete lines in between the selection's starting and ending rows
        let num_deleted = anchor_end.1 - anchor_start.1;
        self.delete_rows(anchor_start.1 + 1, num_deleted);
        self.set_cursor(anchor_start.0, anchor_start.1);
    }

    pub fn get_selection(&mut self) -> String {
        let (anchor_start, anchor_end) = self.get_anchors();

        let mut contents = String::from("");

        // If the anchors are on the same line, just take that selection
        if anchor_start.1 == anchor_end.1 {
            contents.push_str(
                &self.rows.get(anchor_end.1).unwrap().contents[(anchor_start.0)..(anchor_end.0)],
            );
        }
        // Else copy over all selected lines
        else {
            contents.push_str(&self.rows.get(anchor_start.1).unwrap().contents[(anchor_start.0)..]);
            contents.push('\n');
            for idx in (anchor_start.1 + 1)..(anchor_end.1) {
                contents.push_str(&self.rows.get(idx).unwrap().contents);
                contents.push('\n');
            }
            contents.push_str(&self.rows.get(anchor_end.1).unwrap().contents[0..(anchor_end.0)]);
        }
        contents
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        let num_rows = self.num_rows();
        let cy = if y > num_rows { num_rows } else { y };
        let row_len = self.row_len(cy);
        let cx = if x > row_len { row_len } else { x };

        self.cx = cx;
        self.cy = cy;
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


    pub fn get_row_contents(&self, row_idx: usize) -> &String {
        &self.rows.get(row_idx).unwrap().contents
    }


    pub fn cur_row_len(&self) -> usize {
        self.row_len(self.cy)
    }

    pub fn row_len(&self, row_idx: usize) -> usize {
        let num_rows = self.num_rows();
        if row_idx >= num_rows {
            0
        } else {
            self.rows.get(row_idx).unwrap().contents.len()
        }
    }

    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }
}
