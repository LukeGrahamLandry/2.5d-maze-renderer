use sdl2::sys::uint;
use crate::mth::Vector2;
use crate::world::World;

pub(crate) struct Player {
    pub(crate) pos: Vector2,
    pub(crate) direction: Vector2,
    pub(crate) speed: f64,
    pub(crate) region_index: usize
}

impl Player {
    pub(crate) fn new() -> Player {
        Player {
            pos: Vector2::new(),
            direction: Vector2::new(),
            speed: 200.0,
            region_index: 0
        }
    }
}