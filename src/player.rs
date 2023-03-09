use sdl2::keyboard::Keycode;

use crate::mth::{LineSegment2, Vector2};
use crate::world::{Region, Wall};

pub(crate) struct Player {
    pub(crate) pos: Vector2,
    pub(crate) direction: Vector2,
    pub(crate) speed: f64,
    pub(crate) region_index: usize
}

impl Player {
    pub(crate) fn update(&mut self, pressed: &Vec<Keycode>, regions: &Vec<Region>, delta_time: f64) {
        if self.update_direction(pressed) {
            let mut hit_wall = false;
            let player_size = 10.0;
            let last_region = &regions[self.region_index];
            for wall in last_region.walls.iter() {
                if wall.hit_by(&self.pos, &self.direction.scale(player_size)) {
                    if wall.has_next {
                        self.region_index = wall.next_region.unwrap();
                        let next_region = &regions[self.region_index];
                        let new_wall = &next_region.walls[wall.next_wall.unwrap()];
                        self.pos = Wall::translate(&self.pos, &wall, &new_wall);
                        break
                    }

                    hit_wall = true;
                }
            }

            if !hit_wall {
                self.pos.x += self.direction.x * self.speed * delta_time;
                self.pos.y += self.direction.y * self.speed * delta_time;
            }
        }
    }

    fn update_direction(&mut self, pressed: &Vec<Keycode>) -> bool {
        let mut direction = Vector2::new();
        for key in pressed {
            match key {
                Keycode::W => {
                    direction.y = -1.0;
                }
                Keycode::S => {
                    direction.y = 1.0;
                }
                Keycode::A => {
                    direction.x = -1.0;
                }
                Keycode::D => {
                    direction.x = 1.0;
                }
                _ => (),
            }
        }

        if direction.is_zero() {
            false
        } else {
            self.direction = direction.normalize();
            true
        }
    }
}

impl Player {
    pub(crate) fn new() -> Player {
        Player {
            pos: Vector2::new(),
            direction: Vector2::of(1.0, 0.0),
            speed: 200.0,
            region_index: 0
        }
    }
}