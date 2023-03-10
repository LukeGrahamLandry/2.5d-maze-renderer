use std::f64::consts::PI;
use sdl2::keyboard::Keycode;

use crate::mth::{LineSegment2, Vector2};
use crate::world::{Region, Wall};

pub(crate) struct Player {
    pub(crate) pos: Vector2,
    pub(crate) look_direction: Vector2,
    pub(crate) speed: f64,
    pub(crate) region_index: usize
}

const MOVE_SPEED: f64 = 200.0;
const TURN_SPEED: f64 = 0.1;
impl Player {
    pub(crate) fn update(&mut self, pressed: &Vec<Keycode>, regions: &Vec<Region>, delta_time: f64) {
        if self.update_direction(pressed) {
            let mut hit_wall = false;
            let player_size = 10.0;
            let last_region = &regions[self.region_index];
            let move_direction = self.look_direction.scale(player_size * self.speed.signum());
            for wall in last_region.walls.iter() {
                if wall.hit_by(&self.pos, &move_direction) {
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
                self.pos.x += self.look_direction.x * self.speed * delta_time;
                self.pos.y += self.look_direction.y * self.speed * delta_time;
            }
        }
    }

    fn update_direction(&mut self, pressed: &Vec<Keycode>) -> bool {
        self.speed = 0.0;
        for key in pressed {
            match key {
                Keycode::W => {
                    self.speed += MOVE_SPEED;
                }
                Keycode::S => {
                    self.speed -= MOVE_SPEED;
                }
                Keycode::A => {
                    self.look_direction = self.look_direction.rotate(-TURN_SPEED);
                }
                Keycode::D => {
                    self.look_direction = self.look_direction.rotate(TURN_SPEED);
                }
                _ => (),
            }
        }

        self.speed != 0.0
    }
}

impl Player {
    pub(crate) fn new() -> Player {
        Player {
            pos: Vector2::zero(),
            look_direction: Vector2::of(1.0, 0.0),
            speed: 0.0,
            region_index: 0
        }
    }
}