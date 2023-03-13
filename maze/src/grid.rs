use std::fmt::{Display, Formatter};

pub struct Grid {
    pub rows: i32,
    pub cols: i32,
    pub cells: Vec<Cell>
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Pos {
    pub row: i32,
    pub col: i32
}

impl Pos {
    pub fn of(row: i32, col: i32) -> Pos {
        Pos { row, col }
    }
}

pub struct Cell {
    pub pos: Pos,
    pub links: Vec<Pos>
}

impl PartialEq for Cell {
    fn eq(&self, other: &Cell) -> bool {
        self.pos == other.pos
    }
}

impl Grid {
    pub fn new(width: i32, height: i32) -> Grid {
        let mut grid = Grid {
            rows: height,
            cols: width,
            cells: Vec::new()
        };
        grid.prepare_grid();
        grid
    }

    fn prepare_grid(&mut self) {
        self.cells.reserve_exact((self.rows * self.cols) as usize);
        for row in 0..self.rows {
            for col in 0..self.cols {
                self.cells.push(Cell {
                    pos: Pos::of(row, col),
                    links: vec![]
                });
            }
        }
    }

    pub fn north(&self, pos: Pos) -> Pos {
        Pos::of(pos.row - 1, pos.col)
    }

    pub fn south(&self, pos: Pos) -> Pos {
        Pos::of(pos.row + 1, pos.col)
    }

    pub fn east(&self, pos: Pos) -> Pos {
        Pos::of(pos.row, pos.col + 1)
    }

    pub fn west(&self, pos: Pos) -> Pos {
        Pos::of(pos.row, pos.col - 1)
    }

    pub fn has(&self, pos: Pos) -> bool {
        pos.row >= 0 && pos.row < self.rows && pos.col < self.cols && pos.col >= 0
    }

    pub fn mut_cell(&mut self, pos: Pos) -> &mut Cell {
        if !self.has(pos) {
            panic!("Invalid pos {:?}", pos);
        }

        let index = (pos.row * self.cols) + pos.col;
        &mut self.cells[index as usize]
    }

    pub fn get_cell(&self, pos: Pos) -> &Cell {
        if !self.has(pos) {
            panic!("Invalid pos {:?}", pos);
        }

        let index = (pos.row * self.cols) + pos.col;
        &self.cells[index as usize]
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let chars_per_row = 2 + (4 * self.cols) as usize;
        let total_chars = chars_per_row * self.rows as usize;
        let mut result = String::with_capacity(total_chars);

        // The top row
        result.push('+');
        for _ in 0..self.cols {
            result.push_str("---+");
        }
        result.push('\n');

        for row in 0..self.rows {
            let mut top = String::with_capacity(chars_per_row);
            top.push('+');
            let mut bottom = String::with_capacity(chars_per_row);
            bottom.push('|');
            for col in 0..self.cols {
                let pos = Pos::of(row, col);
                if self.get_cell(pos).links.contains(&self.east(pos)) {
                    top.push_str("    ");
                } else {
                    top.push_str("   |");
                }
                if self.get_cell(pos).links.contains(&self.south(pos)) {
                    bottom.push_str("   +");
                } else {
                    bottom.push_str("---+");
                }
            }
            result.push_str(&*top);
            result.push('\n');
            result.push_str(&*bottom);
            result.push('\n');
        }

        write!(f, "{}", result)
    }
}