use crate::grid::{Grid, Pos};
use crate::rand_below;

/// For each cell, randomly choose either north or east to connect.
/// Be careful not to go out of bounds on the north and east edges.
pub fn on(grid: &mut Grid) {
    let mut near: Vec<Pos> = Vec::with_capacity(2);
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let pos = Pos::of(row, col);

            let check = grid.north(pos);
            if grid.has(check) {
                near.push(check);
            }

            let check = grid.east(pos);
            if grid.has(check) {
                near.push(check);
            }

            if near.is_empty() {
                continue;
            }

            let other = near[rand_below(near.len())];
            grid.mut_cell(pos).links.push(other);
            grid.mut_cell(other).links.push(pos);
            near.clear();
        }
    }
}