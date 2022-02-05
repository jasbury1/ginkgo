use crossterm::event::{Event, MouseEvent, MouseEventKind};

use crate::{
    display::Display,
    edit::FileEditComponent,
    file::FileState,
    ui::{Component, Rect},
};

const NO_ACTIVE_VIEW: usize = usize::MAX;

pub struct FileViewerComonent {
    file_views: Vec<FileEditComponent>,
    file_view_bounds: Vec<Rect>,
    active_view: usize,
}

impl FileViewerComonent {
    pub fn new() -> Self {
        FileViewerComonent {
            file_views: vec![],
            file_view_bounds: vec![],
            active_view: NO_ACTIVE_VIEW,
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
        self.active_view = (self.file_views.len() - 1);
    }

    /// Resize each sub-view given the bounds of the file viewer
    pub fn resize_file_views(&mut self, outer_bounds: &Rect) {
        let view_count = self.file_views.len();
        self.file_view_bounds = vec![Rect::default(); view_count];
        let mut n = outer_bounds.width + 1;
        for i in (1..=view_count).rev() {
            let bounds = self.file_view_bounds.get_mut(i - 1).unwrap();
            bounds.height = outer_bounds.height;
            let mut temp = n / i;
            if temp < 2 {
                temp = 2;
            }
            bounds.width = temp;
            bounds.x = n - temp + outer_bounds.x;
            bounds.y = outer_bounds.y;
            n = n - bounds.width;
            self.file_views
                .get_mut(i - 1)
                .unwrap()
                .set_wrap_width(bounds.width);
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        if self.active_view == NO_ACTIVE_VIEW {
            return;
        }

        let cur_view = self.file_views.get_mut(self.active_view as usize).unwrap();

        match event {
            Event::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::Down(_) => self.handle_mouse_down(mouse_event),
                MouseEventKind::Up(_) => {},
                MouseEventKind::Drag(_) => self.handle_mouse_drag(mouse_event),
                MouseEventKind::Moved => {}
                MouseEventKind::ScrollDown => {}
                MouseEventKind::ScrollUp => {}
            },
            _ => {
                cur_view.handle_event(event);
            }
        }
    }

    fn handle_mouse_down(&mut self, event: MouseEvent) {
        for (i, bounds) in self.file_view_bounds.iter().enumerate() {
            if bounds.contains_point((event.column as usize, event.row as usize)) {
                if i != self.active_view {
                    self.active_view = i;
                    return;
                }
                // Normalize the coordinates to the view's bounds, and pass it to that view
                self.file_views
                    .get_mut(i)
                    .unwrap()
                    .handle_event(Event::Mouse(MouseEvent {
                        kind: event.kind,
                        column: event.column - bounds.x as u16,
                        row: event.row - bounds.y as u16,
                        modifiers: event.modifiers,
                    }));
            }
        }
    }

    fn handle_mouse_drag(&mut self, event: MouseEvent) {
        let bounds = self.file_view_bounds.get(self.active_view).unwrap();

        // Only pass along a drag if it hapens within the currently active view
        if bounds.contains_point((event.column as usize, event.row as usize)) {
            // Normalize the coordinates to the view's bounds, and pass it to that view
            self.file_views
                .get_mut(self.active_view)
                .unwrap()
                .handle_event(Event::Mouse(MouseEvent {
                    kind: event.kind,
                    column: event.column - bounds.x as u16,
                    row: event.row - bounds.y as u16,
                    modifiers: event.modifiers,
                }));
        }
    }

    pub fn get_cursor_pos(&self) -> (usize, usize) {
        if self.active_view != NO_ACTIVE_VIEW {
            let cur_view = self.file_views.get(self.active_view as usize).unwrap();
            let cur_bounds = self
                .file_view_bounds
                .get(self.active_view as usize)
                .unwrap();
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
