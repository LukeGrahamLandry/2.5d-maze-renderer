extern crate maze;

fn main() {
    let mut grid = maze::Grid::new(40, 15);
    maze::gen::binary_tree::on(&mut grid);
    println!("{}", grid);
}