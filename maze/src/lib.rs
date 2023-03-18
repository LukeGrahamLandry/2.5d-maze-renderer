pub mod grid;
pub mod gen;
pub mod solve;

pub use grid::{Grid, Cell, Pos};

pub fn rand() -> usize {
    fastrand::usize(usize::MIN..usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
