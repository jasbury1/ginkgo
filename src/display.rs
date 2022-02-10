use std::io::Write;

use crate::ui::Rect;

use crossterm::{
    cursor::{self, Hide, Show},
    event, execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, Result,
};

mod coordinate {
    type Coord = (usize, usize);

    trait Position<T> {
        fn before(self, other: T) -> bool;
        fn after(self, other: T) -> bool;
        fn between(self, first: T, second: T) -> bool;
    }

    impl Position<Coord> for Coord {
        #[inline]
        fn before(self, other: Coord) -> bool {
            self.1 < other.1 || (self.1 == other.1 && self.0 < other.0)
        }

        #[inline]
        fn after(self, other: Coord) -> bool {
            self.1 > other.1 || (self.1 == other.1 && self.0 > other.0)
        }

        #[inline]
        fn between(self, first: Coord, second: Coord) -> bool {
            self.after(first) && self.before(second)
        }
    }
}

pub type CellBlock = Vec<Vec<Cell>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub text: String,
    pub text_color: Color,
    pub bg_color: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            text: String::new(),
            text_color: Color::White,
            bg_color: Color::Black,
        }
    }
}

impl Cell {
    pub fn new(text: &str, text_color: Color, bg_color: Color) -> Self {
        Cell {
            text: String::from(text),
            text_color,
            bg_color,
        }
    }

    pub fn empty_cellblock(rows: usize) -> CellBlock {
        let mut cells = Vec::with_capacity(rows);

        for _ in 0..rows {
            let mut row = Vec::new();
            cells.push(row);
        }
        cells
    }

    pub fn filled(
        text: &str,
        text_color: Color,
        bg_color: Color,
        width: usize,
        height: usize,
    ) -> CellBlock {
        let mut cells = Vec::with_capacity(height);
        let fill_cell = Cell::new(text, text_color, bg_color);

        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(fill_cell.clone());
            }
            cells.push(row);
        }
        cells
    }
}

pub struct Display<W>
where
    W: Write,
{
    pub width: usize,
    pub height: usize,
    pub output: W,
}

impl<W> Display<W>
where
    W: Write,
{
    pub fn new(width: usize, height: usize, w: W) -> Self {
        Display {
            width: width,
            height: height,
            output: w,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    pub fn draw(&mut self, rect: &Rect, cells: &CellBlock) -> Result<()> {
        execute!(self.output, Hide)?;
        // Add the contents of these cells to the display
        for (i, row) in cells.iter().enumerate() {
            let y = i + rect.y;
            queue!(self.output, cursor::MoveTo(rect.x as u16, y as u16))?;
            let mut line_len: usize = 0;
            for cell in row.iter() {
                let cell_len = cell.text.len();
                // Only add to the display if these cells are in the display bounds
                let text: &str;
                if line_len + cell_len < (self.width - rect.x) {
                    text = &cell.text;
                    line_len += cell_len;
                } else {
                    text = &cell.text[0..((self.width - rect.x) - line_len)];
                    line_len = (self.width - rect.x);
                }
                if y < self.height {
                    queue!(
                        self.output,
                        SetForegroundColor(cell.text_color),
                        SetBackgroundColor(cell.bg_color),
                        Print(text)
                    )?;
                }
            }
            queue!(self.output, ResetColor)?;
            queue!(self.output, cursor::MoveToNextLine(1))?;
            self.output.flush()?
        }
        execute!(self.output, Show)?;
        Ok(())
    }

    pub fn draw_fill(&mut self, rect: &Rect, text: &str, text_color: Color, bg_color: Color) {
        todo!();
    }
}
