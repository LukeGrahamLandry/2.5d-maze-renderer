use std::cmp::{max, min};
use sdl2::keyboard::Keycode;

use crate::mth::Vector2;
use crate::player::Player;

pub struct World {
    pub(crate) regions: Vec<Region>,
    pub(crate) player: Player
}

impl World {
    pub(crate) fn new() -> World {
        World {
            player: Player::new(),
            regions: vec![]
        }
    }

    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>){
        self.player.direction.x = 0.0;
        self.player.direction.y = 0.0;
        for key in pressed {
            match key {
                Keycode::W => {
                    self.player.direction.y = -1.0;
                }
                Keycode::S => {
                    self.player.direction.y = 1.0;
                }
                Keycode::A => {
                    self.player.direction.x = -1.0;
                }
                Keycode::D => {
                    self.player.direction.x = 1.0;
                }
                _ => (),
            }
        }
        self.player.direction.normalize();

        let mut hit_wall = false;
        let player_size = 10.0;
        for wall in self.regions[self.player.region_index].walls.iter() {
            if wall.hit_by(&self.player.pos, &self.player.direction.scale(player_size)) {
                hit_wall = true;
            }
        }

        if !hit_wall {
            self.player.pos.x += self.player.direction.x * self.player.speed * delta_time;
            self.player.pos.y += self.player.direction.y * self.player.speed * delta_time;
        }
    }

    pub(crate) fn create_example() -> World {
        let mut world = World::new();
        world.regions.push(Region::new_square(100.0, 200.0, 300.0, 400.0));

        world
    }
}

pub(crate) struct Region {
    pub(crate) walls: Vec<Wall>
}

impl Region {
    fn new() -> Region {
        Region {
            walls: vec![]
        }
    }

    fn new_square(x1: f64, y1: f64, x2: f64, y2: f64) -> Region {
        let mut region = Region::new();
        region.walls.push(Wall {
            a: Vector2::of(x1, y1),
            b: Vector2::of(x2, y1),
            next: -1
        });
        region.walls.push(Wall {
            a: Vector2::of(x1, y2),
            b: Vector2::of(x2, y2),
            next: -1
        });
        region.walls.push(Wall {
            a: Vector2::of(x1, y1),
            b: Vector2::of(x1, y2),
            next: -1
        });
        region.walls.push(Wall {
            a: Vector2::of(x2, y1),
            b: Vector2::of(x2, y2),
            next: -1
        });

        region
    }
}

pub(crate) struct Wall {
    pub(crate) a: Vector2,
    pub(crate) b: Vector2,
    pub(crate) next: i32
}

impl Wall {
    fn hit_by(&self, origin: &Vector2, direction: &Vector2) -> bool {
        let wall_direction = self.b.subtract(&self.a);

        let mut facing = false;
        let mut dist = -1.0;
        let mut at_wall = false;

        if wall_direction.x == 0.0 {  // vertical
            if origin.x > self.a.x {
                facing = direction.x < 0.0;
            } else {
                facing = direction.x > 0.0;
            }

            dist = (origin.x - self.a.x).abs();
            at_wall = origin.y > self.a.y.min(self.b.y) && origin.y < self.a.y.max(self.b.y);
        }

        if wall_direction.y == 0.0 {  // horizontal
            if origin.y > self.a.y {
                facing = direction.y < 0.0;
            } else {
                facing = direction.y > 0.0;
            }

            dist = (origin.y - self.a.y).abs();
            at_wall = origin.x > self.a.x.min(self.b.x) && origin.x < self.a.x.max(self.b.x);
        }

        facing && at_wall && dist > 0.0 && dist < direction.length()
    }
}