use std::io::Write;

use crate::ui::Rect;

use crossterm::{
    cursor::{self, Hide, Show}, event, execute, queue,
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
}

impl Display {
    pub fn new(width: usize, height: usize) -> Self {
        let cellblock = Cell::empty_cellblock(width, height);

        Display {
            cells: cellblock,
            width: width,
            height: height,
        }
    }

    pub fn draw(&mut self, rect: &Rect, cells: &CellBlock) {
        for (i, row) in cells.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let y = i + rect.y;
                let x = j + rect.x;
                if y < self.height && x < self.width {
                    self.cells[y][x] = cell.clone();
                }
            }
        }
    }

    pub fn output<W>(&self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        execute!(w, Hide)?;
        queue!(w, cursor::MoveTo(0, 0))?;
        for row in &self.cells {
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
        Ok(())
    }
}
