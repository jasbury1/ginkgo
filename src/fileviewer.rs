use crossterm::event::Event;

use crate::{edit::FileEditComponent, ui::{Rect, Component}, display::Display, file::FileState};

pub struct FileViewerComonent {
    file_views: Vec<FileEditComponent>,
    file_view_bounds: Vec<Rect>,
    active_view: isize,
}

impl FileViewerComonent {
    pub fn new() -> Self {
        FileViewerComonent {
            file_views: vec![],
            file_view_bounds: vec![],
            active_view: -1,
        }
    }

    pub fn add_files(&mut self, filenames: Vec<&str>) {
        for filename in filenames {
            self.add_file(filename);
        }
    }

    pub fn add_file(&mut self, filename: &str) {
        let mut fs = FileState::new();
        fs.open_file(filename).unwrap();
        self.file_views.push(FileEditComponent::new(fs));
        self.active_view = (self.file_views.len() - 1) as isize;
    }

    /// Resize each sub-view given the bounds of the file viewer
    pub fn resize_file_views(&mut self, outer_bounds: &Rect) {
        let view_count = self.file_views.len();
        self.file_view_bounds = vec![Rect::default(); view_count];
        let mut n = outer_bounds.width + 1;
        for i in (1..=view_count).rev() {
            let bounds = self.file_view_bounds.get_mut(i - 1).unwrap();
            self.file_views.get_mut(i - 1).unwrap().invalidate_cell_cache();
            bounds.height = outer_bounds.height;
            let mut temp = n / i;
            if temp < 2 {
                temp = 2;
            }
            bounds.width = temp;
            bounds.x = n - temp + outer_bounds.x;
            bounds.y = outer_bounds.y;
            n = n - bounds.width;
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        if self.active_view > -1 {
            self.file_views.get_mut(self.active_view as usize).unwrap().handle_event(event);
        }
    }

    pub fn get_cursor_pos(&self) -> (usize, usize) {
        if self.active_view > -1 {
            let cur_view = self.file_views.get(self.active_view as usize).unwrap();
            let cur_bounds = self.file_view_bounds.get(self.active_view as usize).unwrap();
            let mut cursor = cur_view.wrapped_cursor_coords(cur_bounds.width);
            cursor.0 += cur_bounds.x;
            cursor.1 += cur_bounds.y;
            cursor
        } else {
            (0, 0)
        }
    }
}

impl Component for FileViewerComonent {
    type Message = ();

    fn send_msg(&mut self, msg: &Self::Message) {
        todo!()
    }

    fn draw(&mut self, bounds: &Rect, displ: &mut Display) {
        for i in 0..self.file_views.len() {
            let bounds = self.file_view_bounds.get(i).unwrap();
            self.file_views.get_mut(i).unwrap().draw(bounds, displ);
        } 
    }
}
