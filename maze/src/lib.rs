pub mod grid;
pub mod gen;
pub mod solve;

use rand::{thread_rng, Rng};
pub use grid::{Grid, Cell, Pos};

pub fn rand() -> usize {
    rand_below(usize::MAX)
}

pub fn rand_below(max: usize) -> usize {
    thread_rng().gen_range(usize::MIN..max)
}
