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
        self.player.direction = self.player.direction.normalize();

        let mut hit_wall = false;
        let player_size = 10.0;
        let last_region = &self.regions[self.player.region_index];
        for wall in last_region.walls.iter() {
            if wall.hit_by(&self.player.pos, &self.player.direction.scale(player_size)) {
                if wall.has_next {
                    self.player.region_index = wall.next_region.unwrap();
                    let next_region = &self.regions[self.player.region_index];

                    // transform to same position but relative to the new wall, accounting for walls of different sizes.
                    let last_offset = self.player.pos.subtract(&wall.a);
                    let fraction = last_offset.length() / wall.direction().length();
                    let new_wall = &next_region.walls[wall.next_wall.unwrap()];
                    let new_offset = new_wall.direction().negate().scale(fraction);
                    self.player.pos = new_wall.a.add(&new_offset);
                    break
                }

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
        world.regions.push(Region::new_square(500.0, 200.0, 700.0, 400.0));
        world.regions.push(Region::new_square(50.0, 50.0, 150.0, 150.0));

        world.regions[0].walls[0].has_next = true;
        world.regions[0].walls[0].next_region = Some(1);
        world.regions[0].walls[0].next_wall = Some(1);

        world.regions[1].walls[1].has_next = true;
        world.regions[1].walls[1].next_region = Some(0);
        world.regions[1].walls[1].next_wall = Some(0);

        world.regions[1].walls[2].has_next = true;
        world.regions[1].walls[2].next_region = Some(2);
        world.regions[1].walls[2].next_wall = Some(3);

        world.regions[2].walls[3].has_next = true;
        world.regions[2].walls[3].next_region = Some(1);
        world.regions[2].walls[3].next_wall = Some(2);

        world.player.pos.x = 150.0;
        world.player.pos.y = 250.0;

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
            has_next: false,
            next_region: None,
            next_wall: None,
        });
        region.walls.push(Wall {
            a: Vector2::of(x1, y2),
            b: Vector2::of(x2, y2),
            has_next: false,
            next_region: None,
            next_wall: None,
        });
        region.walls.push(Wall {
            a: Vector2::of(x1, y1),
            b: Vector2::of(x1, y2),
            has_next: false,
            next_region: None,
            next_wall: None,
        });
        region.walls.push(Wall {
            a: Vector2::of(x2, y1),
            b: Vector2::of(x2, y2),
            has_next: false,
            next_region: None,
            next_wall: None,
        });

        region
    }
}

pub(crate) struct Wall {
    pub(crate) a: Vector2,
    pub(crate) b: Vector2,
    pub(crate) has_next: bool,
    pub(crate) next_region: Option<usize>,
    pub(crate) next_wall: Option<usize>
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

    fn direction(&self) -> Vector2 {
        self.a.subtract(&self.b)
    }
}