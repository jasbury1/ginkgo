use core::num;
use std::cmp;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;

#[allow(dead_code)]
pub struct Erow {
    idx: usize,
    contents: String,
    render: String,
    highlight: Vec<u8>,
    comment_open: bool,
}

#[allow(dead_code)]
pub enum StatusMsg {
    Normal(String),
    Warn(String),
    Error(String),
}

#[allow(dead_code)]
pub struct Model {
    pub cx: usize,
    pub cy: usize,
    pub rx: usize,
    pub rowoff: usize,
    pub coloff: usize,
    pub dirty: usize,
    pub filename: String,
    pub ext: String,
    pub status_msg: StatusMsg,

    pub anchor_start: (usize, usize),
    pub anchor_end: (usize, usize),
    pub text_selected: bool,

    pub mode: char,

    rows: Vec<Erow>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            cx: 0,
            cy: 0,
            rx: 0,
            rowoff: 0,
            coloff: 0,
            dirty: 0,
            rows: vec![],
            filename: String::from(""),
            ext: String::from(""),
            status_msg: StatusMsg::Normal(String::from(
                "HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find",
            )),
            anchor_start: (0, 0),
            anchor_end: (0, 0),
            text_selected: false,
            mode: 'N'
        }
    }

    pub fn open_file(&mut self, filename: &str) -> () {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename);
        let reader: BufReader<File>;

        match f {
            Ok(file) => {
                reader = BufReader::new(file);
            }
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    self.status_msg =
                        StatusMsg::Error(format!("Unable to create file {:?}.", filename));
                    return;
                }
                ErrorKind::PermissionDenied => {
                    self.status_msg = StatusMsg::Error(format!(
                        "Unable to open {:?}. Permission denied.",
                        filename
                    ));
                    return;
                }
                other_error => {
                    self.status_msg = StatusMsg::Error(format!(
                        "Problem opening file {:?}. {:?}.",
                        filename, other_error
                    ));
                    return;
                }
            },
        };

        for line_ in reader.lines() {
            let line = line_.unwrap();
            self.append_row(line);
        }
        self.dirty = 0;
    }

    pub fn save_file(&mut self) {}

    ///
    fn append_row(&mut self, line: String) -> () {
        let num_rows = self.num_rows();
        let render = line.clone();

        let row = Erow {
            idx: num_rows,
            contents: line,
            comment_open: false,
            highlight: vec![],
            render: render,
        };

        self.rows.insert(num_rows, row);
        Model::update_row_render(self.rows.get_mut(num_rows).unwrap());

        self.dirty += 1;
    }

    ///
    fn insert_row(&mut self, idx: usize, line: String) {
        let num_rows = self.num_rows();
        if idx > num_rows {
            return;
        }

        let render = line.clone();

        let row = Erow {
            idx: idx,
            contents: line,
            comment_open: false,
            highlight: vec![],
            render: render,
        };

        for i in idx..num_rows {
            self.rows.get_mut(i).unwrap().idx += 1;
        }

        self.rows.insert(idx, row);
        Model::update_row_render(self.rows.get_mut(idx).unwrap());

        self.dirty += 1;
    }

    ///
    pub fn insert_newline(&mut self) {
        let cur_row = self.rows.get_mut(self.cy).unwrap();
        let cur_row_len = cur_row.contents.len();

        if self.cx == 0 {
            self.insert_row(self.cy, String::from(""));
        } else if self.cx == cur_row_len {
            self.insert_row(self.cy + 1, String::from(""));
        } else {
            let leftover = String::from(&cur_row.contents[self.cx..]);
            self.insert_row(self.cy + 1, leftover);
            self.rows
                .get_mut(self.cy)
                .unwrap()
                .contents
                .truncate(self.cx);
            Model::update_row_render(self.rows.get_mut(self.cy).unwrap());
        }
        self.cy += 1;
        self.cx = 0;
    }

    ///
    pub fn insert_char(&mut self, c: char) {
        let num_rows = self.num_rows();
        if self.cy == num_rows {
            self.insert_row(num_rows, String::from(""));
        }

        let cur_row = self.rows.get_mut(self.cy).unwrap();
        let mut at = self.cx;

        if at > cur_row.contents.len() {
            at = cur_row.contents.len()
        }
        cur_row.contents.insert(at, c);
        Model::update_row_render(cur_row);

        self.dirty += 1;
        self.cx += 1;
    }

    pub fn delete_row(&mut self, row_idx: usize) {
        self.rows.remove(row_idx);

        let num_rows = self.num_rows();
        for i in row_idx..num_rows {
            self.rows.get_mut(i).unwrap().idx -= 1;
        }
        self.dirty += 1;
    }

    pub fn delete_rows(&mut self, row_idx: usize, num_removed: usize) {
        for _ in 0..num_removed {
            self.rows.remove(row_idx);
        }
        let num_rows = self.num_rows();
        for i in row_idx..num_rows {
            self.rows.get_mut(i).unwrap().idx -= num_removed;
        }
        self.dirty += 1;
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
            Model::update_row_render(self.rows.get_mut(self.cy).unwrap());
            self.dirty += 1;
            self.cx -= 1;
        } else {
            let cur_row = self.rows.get(self.cy).unwrap().contents.clone();
            let prev_row = &mut self.rows.get_mut(self.cy - 1).unwrap().contents;
            self.cx = prev_row.len();
            prev_row.push_str(&cur_row);
            Model::update_row_render(self.rows.get_mut(self.cy - 1).unwrap());
            self.dirty += 1;
            self.delete_row(self.cy);
            self.cy -= 1;
        }
    }

    pub fn cx_to_rx(&self, row: &Erow, cx: usize) -> usize {
        let mut rx = 0;
        for i in 0..cx {
            // TODO: Finish function
        }
        cx
    }

    pub fn rx_to_cx(&self, row: &Erow, rx: usize) -> usize {
        let mut cx = 0;
        for i in 0..rx {
            // TODO: Finish function
        }
        rx
    }

    pub fn get_cur_row(&self) -> &Erow {
        self.rows.get(self.cy).unwrap()
    }

    pub fn get_render(&self, row_idx: usize, start: usize, end: usize) -> Option<String> {
        match self.rows.get(row_idx) {
            Some(row) => {
                let end = cmp::min(end, row.render.len());
                Some(row.render.get(start..end).unwrap_or_default().to_string())
            }
            None => None,
        }
    }

    fn update_row_render(row: &mut Erow) {
        // TODO: More advanced logic later
        row.render = row.contents.clone();
    }

    pub fn cur_row_len(&self) -> usize {
        self.row_len(self.cy)
    }

    pub fn row_len(&self, row_idx: usize) -> usize {
        self.rows.get(row_idx).unwrap().contents.len()
    }

    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }
}
