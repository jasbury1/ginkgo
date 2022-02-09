use std::io::Write;

use crate::ui::Rect;

use crossterm::{
    cursor::{self, Hide, Show},
    event, execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, Result,
};

pub type CellBlock = Vec<Vec<Cell>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub c: char,
    pub text_color: Color,
    pub bg_color: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            c: ' ',
            text_color: Color::White,
            bg_color: Color::Black,
        }
    }
}

impl Cell {
    pub fn new(c: char, text_color: Color, bg_color: Color) -> Self {
        Cell {
            c,
            text_color,
            bg_color,
        }
    }

    pub fn empty_cellblock(width: usize, height: usize) -> CellBlock {
        let mut cells = Vec::with_capacity(height);
        let empty_cell = Cell::default();

        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(empty_cell.clone());
            }
            cells.push(row);
        }
        cells
    }

    pub fn filled_cellblock(
        c: char,
        text_color: Color,
        bg_color: Color,
        width: usize,
        height: usize,
    ) -> CellBlock {
        let mut cells = Vec::with_capacity(height);
        let fill_cell = Cell::new(c, text_color, bg_color);

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
            for (j, cell) in row.iter().enumerate() {
                let x = j + rect.x;
                
                // Only add to the display if these cells are in the display bounds
                if y < self.height && x < self.width {   
                    queue!(
                        self.output,
                        SetForegroundColor(cell.text_color),
                        SetBackgroundColor(cell.bg_color),
                        Print(cell.c)
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
}
