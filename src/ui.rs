use std::io::Stdout;

use crossterm::event::Event;

use crate::display::{CellBlock, Display};

pub enum EventResponse {
    NoResponse,
    MoveCursor,
    RedrawDisplay,
    Quit
}

pub trait Component {
    type Message;

    fn send_msg(&mut self, msg: &Self::Message) -> EventResponse;
    fn handle_event(&mut self, event: Event) -> EventResponse;

    /// Called when a component must re-draw itself to the display.
    /// Create a Cellblock representation of this component and use it to draw to the display
    /// 
    /// # Arguments
    ///
    /// * `bounds` - The bounds that self has to draw itself within
    /// * `display` - The display we draw our representation to
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Drawing a rectangle of '#' characters within our bounds to the display
    /// 
    /// impl Component for HashRect
    ///     fn draw(&mut self, bounds: &Rect, displ: &mut Display<Stdout>) {
    ///       let mut cellblock = Cell::empty_cellblock(bounds.height);
    ///       for row in 0..bounds.height {
    ///           for _ in bounds.width {
    ///               cellblock[row].push(Cell::new("#", Color::White, Color::Black));
    ///          }
    ///       }
    ///       displ.draw(bounds, &cellblock);
    ///     }
    /// 
    ///     ...
    /// }
    /// 
    /// // Calling the method
    /// let bounds = Rect {
    ///      x: 0,
    ///      y: 0,
    ///      width: 5,
    ///      height: 5,
    /// };
    /// let mut display = Display::new(10, 10, stdout());
    /// let hs = HashRect::new()
    /// hs.draw(&bounds, &mut display);
    /// ```
    fn draw(&mut self, bounds: &Rect, displ: &mut Display<Stdout>);

    /// Called on a component after the component's parent has adjusted the components bounds
    /// Only handle any internal state changes necessary for your resize. Other methods such
    /// as draw will be called separately following the resize event.
    /// 
    /// The component that the resize method was called on is responsible for adjusting the
    /// sizes of its own children and calling resize on those children with their new
    /// respective sizes.
    /// 
    /// resize can be a noop if no state changes are necessary.
    fn resize(&mut self, bounds: &Rect);
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn default() -> Self {
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn contains_point(&self, point: (usize, usize)) -> bool {
        point.0 >= self.x
            && point.0 < (self.x + self.width)
            && point.1 >= self.y
            && point.1 < (self.y + self.height)
    }
}
