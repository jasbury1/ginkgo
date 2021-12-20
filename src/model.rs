use std::cmp;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;

#[allow(dead_code)]
struct Erow {
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
            status_msg: StatusMsg::Normal(String::from("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find")),
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

    pub fn save_file(&mut self) {

    }

    fn append_row(&mut self, line: String) -> () {
        let idx = self.rows.len();
        // TODO: Remove these two clones
        let row = Erow {
            idx: idx,
            contents: line.clone(),
            comment_open: false,
            highlight: vec![],
            render: line.clone(),
        };

        self.rows.insert(idx, row);
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

    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }
}
