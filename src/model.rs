use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::prelude::*;

#[allow(dead_code)]
struct Erow {
    idx: usize,
    contents: String,
    render: String,
    highlight: Vec<u8>,
    comment_open: bool,
}

#[allow(dead_code)]
enum StatusMsg {
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

    rows: Vec<Erow>,
    filename: String,
    ext: String,
    status_msg: StatusMsg,
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
            status_msg: StatusMsg::Normal(String::from("")),
        }
    }

    pub fn open_file(&mut self, filename: &str) -> () {
        let f = OpenOptions::new().read(true).create(true).open(filename);
        let reader: BufReader<File>;

        match f {
            Ok(file) => {
                reader = BufReader::new(file);
            },
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    self.status_msg = StatusMsg::Error(format!("Unable to create file {:?}.", filename));
                    return
                },
                ErrorKind::PermissionDenied => {
                    self.status_msg = StatusMsg::Error(format!("Unable to open {:?}. Permission denied.", filename));
                    return
                },
                other_error => {
                    self.status_msg = StatusMsg::Error(format!("Problem opening file {:?}. {:?}.", filename, other_error));
                    return
                },
            }
        };

        for line_ in reader.lines() {
            let line = line_.unwrap();
            self.append_row(line);
        }
        
        self.dirty = 0;
    }


    fn append_row(&mut self, line: String) -> () {
        let idx = self.rows.len();
        let row = Erow {
            idx: idx,
            contents: line,
            comment_open: false,
            highlight: vec![],
            render: String::from(""),
        };

        self.rows.insert(idx, row);
    }

    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }

}
