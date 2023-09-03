pub mod grid;
pub mod gen;
pub mod solve;

use rand::rngs::StdRng;
use rand::SeedableRng;
pub use grid::{Grid, Cell, Pos};

pub fn rand() -> usize {
    rand_below(usize::MAX)
}


static mut rng: Option<StdRng> = None;

pub fn rand_below(max: usize) -> usize {
    use rand::Rng;

    const seed: [u8; 32] = [1,0,0,0, 23,0,0,0, 200,1,0,0, 210,30,0,0,
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0];
    unsafe {
        match &mut rng {
            Some(r) => r.gen_range(usize::MIN..max),
            None => {
                rng = Some(StdRng::from_seed(seed));
                rand_below(max)
            }
        }
    }
}
