use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::io::{self, prelude::*};
use std::path::PathBuf;

pub struct Erow {
    idx: usize,
    contents: String,
}

pub struct FileState {
    pub filename: String,
    pub path: PathBuf,
    pub ext: String,
    rows: Vec<Erow>,
}

impl FileState {
    pub fn new() -> Self {
        FileState {
            filename: String::from(""),
            path: PathBuf::new(),
            ext: String::from(""),
            rows: Vec::new(),
        }
    }

    pub fn open_file(&mut self, input_path: &str) -> Result<(), io::Error> {
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
            Err(err) => {
                return Err(err);
            }
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
        Ok(())
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

    /// Insert a newline at the specified xy location specified by loc
    pub fn insert_newline(&mut self, loc: (usize, usize)) -> (usize, usize) {
        let cur_row = self.rows.get_mut(loc.1).unwrap();
        let cur_row_len = cur_row.contents.len();

        if loc.0 == 0 {
            self.insert_row(loc.1, "");
        } else if loc.0 == cur_row_len {
            self.insert_row(loc.1 + 1, "");
        } else {
            let leftover = String::from(&cur_row.contents[loc.0..]);
            self.insert_row(loc.1 + 1, &leftover);
            self.rows.get_mut(loc.1).unwrap().contents.truncate(loc.0);
        }
        (0, loc.1 + 1)
    }

    ///
    pub fn insert_char(&mut self, c: char, loc: (usize, usize)) -> (usize, usize) {
        let num_rows = self.num_rows();
        if loc.1 == num_rows {
            self.insert_row(num_rows, "");
        }

        let cur_row = self.rows.get_mut(loc.1).unwrap();
        let mut at = loc.0;

        if at > cur_row.contents.len() {
            at = cur_row.contents.len()
        }
        cur_row.contents.insert(at, c);
        (loc.0 + 1, loc.1)
    }

    /// Insert the string into the document at the XY position designated by loc.
    /// Newlines inside the input string get separated out into individual
    /// rows that will be reflected in the document. Returns the end XY coordinates
    /// of the string that was inserted
    ///
    /// # Arguments
    ///
    /// * `contents` - The string to insert
    /// * `loc` - An X, Y tuple of where to place the string
    ///
    pub fn insert_string(&mut self, contents: &str, loc: (usize, usize)) -> (usize, usize) {
        let mut loc = loc;
        // Initialize the buffer to the current line prior to the cursor
        let mut buffer: String = (&self.rows.get(loc.1).unwrap().contents[0..loc.0]).to_string();
        // Add the contents we are pushing
        buffer.push_str(contents);
        // Add the end after the cursor
        buffer.push_str(&self.rows.get(loc.1).unwrap().contents[loc.0..]);

        let mut idx = loc.1;
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
        if idx == loc.1 {
            // If the insertion was within the same line, just move cx forward
            loc.0 += contents.len();
        } else {
            // If we ended up adding additional lines, adjust cx and cy
            loc.0 = line_len;
            loc.1 = idx;
        }
        loc
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

    pub fn delete_char(&mut self, loc: (usize, usize)) -> (usize, usize) {
        let mut loc = loc;
        let num_rows = self.num_rows();

        if loc.1 == num_rows {
            return loc;
        }
        if loc.0 == 0 && loc.1 == 0 {
            return loc;
        }

        if loc.0 > 0 {
            let cur_row = &mut self.rows.get_mut(loc.1).unwrap().contents;
            if loc.0 > cur_row.len() {
                return loc;
            }
            cur_row.remove(loc.0 - 1);
            loc.0 -= 1;
        } else {
            let cur_row = self.rows.get(loc.1).unwrap().contents.clone();
            let prev_row = &mut self.rows.get_mut(loc.1 - 1).unwrap().contents;
            loc.0 = prev_row.len();
            prev_row.push_str(&cur_row);
            self.delete_row(loc.1);
            loc.1 -= 1;
        }
        loc
    }

    /// Returns the character the cursor is pointing at, or a newline
    /// character if the cursor is pointing to the beginning of the line
    pub fn get_char(&self, loc: (usize, usize)) -> char {
        let cur_row = &self.rows.get(loc.1).unwrap().contents;
        if loc.0 == 0 {
            '\n'
        } else {
            cur_row.chars().nth(loc.0 - 1).unwrap_or_default()
        }
    }

    pub fn delete_selection(
        &mut self,
        anchor_start: (usize, usize),
        anchor_end: (usize, usize),
    ) -> (usize, usize) {
        let end_row = self.rows.get(anchor_end.1).unwrap().contents.clone();
        let start_row = &mut self.rows.get_mut(anchor_start.1).unwrap().contents;
        start_row.truncate(anchor_start.0);
        start_row.push_str(&end_row[anchor_end.0..]);

        // Delete the complete lines in between the selection's starting and ending rows
        let num_deleted = anchor_end.1 - anchor_start.1;
        self.delete_rows(anchor_start.1 + 1, num_deleted);
        anchor_start
    }

    pub fn get_selection(
        &mut self,
        anchor_start: (usize, usize),
        anchor_end: (usize, usize),
    ) -> String {
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

    pub fn get_row_contents(&self, row_idx: usize) -> &String {
        &self.rows.get(row_idx).unwrap().contents
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

    pub fn clamp_to_bounds(&self, loc: (usize, usize)) -> (usize, usize) {
        let mut result = loc;
        let num_rows = self.num_rows();
        if result.1 > num_rows {
            result.1 = num_rows;
        }
        let rowlen = self.row_len(result.1);
        if result.0 > rowlen {
            result.0 = rowlen;
        }
        result
    }

    pub fn clamp_within_bounds(&self, loc: (usize, usize)) -> (usize, usize) {
        //TODO: These bounds are off. They don't handle 0 properly
        let mut result = loc;
        let num_rows = self.num_rows();
        if result.1 >= num_rows {
            result.1 = num_rows - 1;
        }
        let rowlen = self.row_len(result.1);
        if result.0 >= rowlen {
            result.0 = rowlen - 1;
        }
        result
    }
}
