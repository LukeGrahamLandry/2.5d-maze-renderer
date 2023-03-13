extern crate maze;

use maze::{Grid, Cell};
use maze::gen::binary_tree;

fn main() {
    let mut grid = Grid::new(5, 5);
    binary_tree::on(&mut grid);
    println!("{}", grid);
}