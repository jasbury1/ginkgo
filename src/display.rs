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

pub struct Display {
    pub cells: CellBlock,
    pub width: usize,
    pub height: usize,
    // Min/max row keep track of the range of rows that have been updated
    // since the display was last outputed
    pub min_row: usize,
    pub max_row: usize,
}

impl Display {
    pub fn new(width: usize, height: usize) -> Self {
        let cellblock = Cell::empty_cellblock(width, height);

        Display {
            cells: cellblock,
            width: width,
            height: height,
            min_row: usize::MAX,
            max_row: usize::MIN,
        }
    }

    pub fn draw(&mut self, rect: &Rect, cells: &CellBlock) {
        // Update the min and max row range for next time we call output
        let max_row = if rect.y + rect.height >= self.height {
            self.height
        } else {
            rect.y + rect.height
        };
        let min_row = if rect.y >= self.height {
            self.height
        } else {
            rect.y
        };
        if max_row > self.max_row {
            self.max_row = max_row;
        }
        if min_row < self.min_row {
            self.min_row = min_row;
        }
        // Add the contents of these cells to the display
        for (i, row) in cells.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let y = i + rect.y;
                let x = j + rect.x;
                // Only add to the display if these cells are in the display bounds
                if y < self.height && x < self.width {
                    self.cells[y][x] = cell.clone();
                }
            }
        }
    }

    pub fn output<W>(&mut self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        execute!(w, Hide)?;
        queue!(w, cursor::MoveTo(self.min_row as u16, 0))?;
        for i in self.min_row..self.max_row {
            let row = self.cells.get(i).unwrap();
            queue!(w, terminal::Clear(ClearType::CurrentLine))?;
            for cell in row {
                queue!(
                    w,
                    SetForegroundColor(cell.text_color),
                    SetBackgroundColor(cell.bg_color),
                    Print(cell.c)
                )?;
            }
            queue!(w, ResetColor)?;
            queue!(w, cursor::MoveToNextLine(1))?;
            w.flush()?
        }
        execute!(w, Show)?;
        self.min_row = usize::MAX;
        self.max_row = usize::MIN;
        Ok(())
    }
}
