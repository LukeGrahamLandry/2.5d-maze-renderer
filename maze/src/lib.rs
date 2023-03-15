pub mod grid;
pub mod gen;
pub mod solve;

pub use grid::{Grid, Cell, Pos};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
