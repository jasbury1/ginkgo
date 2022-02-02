mod display;
mod edit;
mod editor;
mod file;
mod status;
mod ui;
mod fileviewer;

use clap::{App, Arg};
use crossterm::style::Color;
use display::{Cell, Display};
use edit::FileEditComponent;
use editor::TextEditor;
use file::FileState;
use status::StatusBarComponent;
use std::io::{self, Write};
use ui::{Component, Rect};

const GINKGO_VERSION: &str = "0.1";

fn main() {
    
    let args = App::new("Ginkgo")
        .version(GINKGO_VERSION)
        .about("Edits a file")
        .arg(
            Arg::with_name("file")
                .help("The file to open")
                .takes_value(true)
                .multiple(true)
                .required(true),
        )
        .get_matches();

    let files: Vec<&str> = args.values_of("file").unwrap().collect();

    let mut editor = TextEditor::new();
    editor.open_files(files);
    editor.run();
    
    /*
    let mut fs = FileState::new();
    fs.open_file("test/test2.c").unwrap();
    let edit = FileEditComponent::new(fs);
    let status = StatusBarComponent::new();

    let rect: Rect = Rect {
        x: 1,
        y: 2,
        width: 10,
        height: 10,
    };
    let rect2: Rect = Rect {
        x: 0,
        y: 0,
        width: 15,
        height: 15,
    };
    let rect3: Rect = Rect {
        x: 0,
        y: 13,
        width: 15,
        height: 1,
    }; 
    let text_cells = edit.draw(rect);
    let status_cells = status.draw(rect3);
    let bg_cells = Cell::filled_cellblock(
        ' ',
        crossterm::style::Color::Black,
        crossterm::style::Color::Red,
        15,
        15,
    );

    let mut display = Display::new(15, 15);
    display.draw(&rect2, &bg_cells);
    display.draw(&rect, &text_cells);
    display.draw(&rect3, &status_cells);
    let mut out = io::stdout();
    display.output(&mut out).unwrap();

    */
}
