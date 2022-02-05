use std::sync::mpsc::{self, Receiver, Sender};
use std::{io, thread};

use std::io::{stdout, Stdout};
use std::result;

use crossterm::cursor::MoveTo;
use crossterm::event::{poll, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{
    cursor::position,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};
use std::time::Duration;

use crate::display::Display;
use crate::edit::FileEditComponent;
use crate::file::FileState;
use crate::fileviewer::FileViewerComonent;
use crate::status::StatusBarComponent;
use crate::ui::{Component, Rect};

const QUIT_TIMES: u8 = 3;

enum EditMode {
    Normal,
    Insert,
}

pub struct TextEditor {
    rx: Receiver<Event>,
    display: Display,
    output: Stdout,
    file_viewer: FileViewerComonent,
    file_viewer_bounds: Rect,
    status_bar: StatusBarComponent,
    status_bar_bounds: Rect,
    quit_times: u8,
}

impl TextEditor {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || TextEditor::read_input(tx));
        let dims = TextEditor::get_terminal_size();

        TextEditor {
            rx,
            display: Display::new(dims.0, dims.1),
            output: stdout(),
            file_viewer: FileViewerComonent::new(),
            file_viewer_bounds: Rect::default(),
            status_bar: StatusBarComponent::new(),
            status_bar_bounds: Rect::default(),
            quit_times: QUIT_TIMES,
        }
    }

    pub fn run(&mut self) -> result::Result<(), Box<dyn std::error::Error>> {
        execute!(self.output, EnterAlternateScreen)?;
        enable_raw_mode()?;
        execute!(self.output, EnableMouseCapture)?;
        let size = Self::get_terminal_size();
        self.resize_editor(size.0, size.1);
        self.draw_display();

        'event: loop {
            let evt = self.rx.recv()?;
            match evt {
                Event::Resize(width, height) => {
                    self.resize_editor(width as usize, height as usize);
                }
                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('q'),
                }) => {
                    //TODO:
                    break 'event;
                    /*
                    if self.file_views.get(0).unwrap().change_count == 0 {
                        break 'event;
                    } else {
                        continue 'event;
                    }
                    */
                }
                Event::Mouse(MouseEvent {
                    kind: _,
                    column,
                    row,
                    modifiers: _,
                }) => {
                    if self.file_viewer_bounds.contains_point((column as usize, row as usize)) {
                        self.file_viewer.handle_event(evt);
                    }
                }
                _ => {
                    self.file_viewer.handle_event(evt);
                }
            }
            self.quit_times = QUIT_TIMES;
            self.draw_display();
            self.draw_cursor();
        }

        execute!(self.output, DisableMouseCapture)?;
        disable_raw_mode()?;
        execute!(self.output, LeaveAlternateScreen)?;
        Ok(())
    }

    fn read_input(tx: Sender<Event>) -> Result<()> {
        loop {
            // Blocking read
            let mut event = read()?;

            if let Event::Resize(_, _) = event {
                event = TextEditor::merge_resize_events(event);
            }

            match event {
                Event::Key(_) => {
                    tx.send(event).unwrap();
                }
                Event::Mouse(MouseEvent{kind: MouseEventKind::Moved, ..}) => {}
                Event::Mouse(_) => {
                    tx.send(event).unwrap();
                }
                Event::Resize(_, _) => {
                    tx.send(event).unwrap();
                }
            }
        }
        Ok(())
    }

    // Keeps the last resize event
    fn merge_resize_events(event: Event) -> Event {
        if let Event::Resize(x, y) = event {
            let mut last_resize = (x, y);
            while let Ok(true) = poll(Duration::from_millis(50)) {
                if let Ok(Event::Resize(x, y)) = read() {
                    last_resize = (x, y);
                }
            }
            return Event::Resize(last_resize.0, last_resize.1);
        }
        Event::Resize(0, 0)
    }

    pub fn open_files(&mut self, filenames: Vec<&str>) {
        self.file_viewer.add_files(filenames);
    }

    pub fn open_file(&mut self, filename: &str) {
        self.file_viewer.add_file(filename);
    }

    pub fn resize_editor(&mut self, width: usize, height: usize) {
        self.display = Display::new(width, height);
        self.status_bar_bounds = Rect {
            x: 0,
            y: height - 1,
            width,
            height: 1,
        };
        self.file_viewer_bounds = Rect {
            x: 0,
            y: 0,
            width,
            height: height - 1,
        };
        self.file_viewer.resize_file_views(&self.file_viewer_bounds);
    }

    pub fn draw_display(&mut self) {
        self.status_bar
            .draw(&self.status_bar_bounds, &mut self.display);
        self.file_viewer
            .draw(&self.file_viewer_bounds, &mut self.display);
        self.display.output(&mut self.output).unwrap();
    }

    pub fn draw_cursor(&mut self) {
        let cursor = self.file_viewer.get_cursor_pos();
        execute!(self.output, MoveTo(cursor.0 as u16, cursor.1 as u16)).unwrap();
    }

    fn get_terminal_size() -> (usize, usize) {
        let size = terminal::size().unwrap();
        (size.0 as usize, size.1 as usize)
    }
}
