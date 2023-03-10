use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use crate::mth::{LineSegment2, Vector2};
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
        self.player.update(&pressed, &self.regions, delta_time);
    }

    pub(crate) fn create_example() -> World {
        let mut world = World::new();
        world.regions.push(Region::new_square(100.0, 200.0, 300.0, 400.0));
        world.regions.push(Region::new_square(500.0, 200.0, 700.0, 400.0));
        world.regions.push(Region::new_square(50.0, 50.0, 150.0, 150.0));

        world.regions[0].floor_color = Color::RGB(0, 50, 50);
        world.regions[1].floor_color = Color::RGB(0, 50, 0);
        world.regions[2].floor_color = Color::RGB(0, 0, 50);
        // world.regions[1].light_intensity = 0.5;
        // world.regions[2].light_intensity = 0.01;

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
    pub(crate) walls: Vec<Wall>,
    pub(crate) floor_color: Color,
    pub(crate) light_pos: Vector2,
    pub(crate) light_intensity: f64
}

impl Region {
    fn new() -> Region {
        Region {
            walls: vec![],
            floor_color: Color::RGBA(0, 0, 0, 255),
            light_pos: Vector2::zero(),
            light_intensity: 1.0
        }
    }

    fn new_square(x1: f64, y1: f64, x2: f64, y2: f64) -> Region {
        let mut region = Region::new();
        region.walls.push(Wall {
            line: LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x2, y1)),
            has_next: false,
            next_region: None,
            next_wall: None,
        });
        region.walls.push(Wall {
            line: LineSegment2::of(Vector2::of(x1, y2), Vector2::of(x2, y2)),
            has_next: false,
            next_region: None,
            next_wall: None,
        });
        region.walls.push(Wall {
            line: LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x1, y2)),
            has_next: false,
            next_region: None,
            next_wall: None,
        });
        region.walls.push(Wall {
            line: LineSegment2::of(Vector2::of(x2, y1), Vector2::of(x2, y2)),
            has_next: false,
            next_region: None,
            next_wall: None,
        });
        region.light_pos = region.walls[0].line.a.add(&region.walls[0].line.direction().scale(-0.25).add(&region.walls[2].line.direction().scale(-0.25)));

        region
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Wall {
    pub(crate) line: LineSegment2,
    pub(crate) has_next: bool,
    pub(crate) next_region: Option<usize>,
    pub(crate) next_wall: Option<usize>
}

impl Wall {
    pub(crate) fn hit_by(&self, origin: &Vector2, direction: &Vector2) -> bool {
        let ray = LineSegment2::from(*origin, *direction);
        self.line.overlaps(&ray)
    }

    // transform to same position but relative to the new wall, accounting for walls of different sizes.
    pub(crate) fn translate(pos: &Vector2, from: &Wall, to: &Wall) -> Vector2 {
        let last_offset = pos.subtract(&from.line.a);
        let fraction = last_offset.length() / from.line.direction().length();
        let new_offset = to.line.direction().negate().scale(fraction);

        to.line.a.add(&new_offset)
    }
}