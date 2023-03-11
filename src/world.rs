use std::cell::{Cell, Ref, RefCell};
use std::ops::Deref;
use std::rc::{Rc, Weak};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;

pub struct World {
    pub(crate) regions: Vec<Rc<RefCell<Region>>>,
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

        world.regions[0].borrow_mut().floor_color = Color::RGB(0, 50, 50);
        world.regions[1].borrow_mut().floor_color = Color::RGB(0, 50, 0);
        world.regions[2].borrow_mut().floor_color = Color::RGB(0, 0, 50);
        // world.regions[1].light_intensity = 0.5;
        // world.regions[2].light_intensity = 0.01;

        let line = LineSegment2::of(Vector2::of(200.0, 300.0), Vector2::of(200.0, 325.0));
        world.regions[0].borrow_mut().walls.push(Wall {
            normal: line.normal(),
            line,
            has_next: true,
            next_region: Some(2),
            next_wall: Some(1),
            region: (Rc::downgrade(&world.regions[0]))
        });

        let line = LineSegment2::of(Vector2::of(175.0, 300.0), Vector2::of(175.0, 325.0));
        let wall = Wall {
            normal: line.normal().negate(),
            line,
            has_next: true,
            next_region: Some(2),
            next_wall: Some(0),
            region: Rc::downgrade(&world.regions[0])
        };
        world.regions[0].borrow_mut().walls.push(wall);

        world.regions[0].borrow_mut().walls[0].has_next = true;
        world.regions[0].borrow_mut().walls[0].next_region = Some(1);
        world.regions[0].borrow_mut().walls[0].next_wall = Some(1);

        world.regions[1].borrow_mut().walls[1].has_next = true;
        world.regions[1].borrow_mut().walls[1].next_region = Some(0);
        world.regions[1].borrow_mut().walls[1].next_wall = Some(0);

        world.regions[1].borrow_mut().walls[2].has_next = true;
        world.regions[1].borrow_mut().walls[2].next_region = Some(2);
        world.regions[1].borrow_mut().walls[2].next_wall = Some(3);

        world.regions[2].borrow_mut().walls[3].has_next = true;
        world.regions[2].borrow_mut().walls[3].next_region = Some(1);
        world.regions[2].borrow_mut().walls[3].next_wall = Some(2);

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
    fn new() -> Rc<RefCell<Region>> {
        Rc::new(RefCell::new(Region {
            walls: vec![],
            floor_color: Color::RGBA(0, 0, 0, 255),
            light_pos: Vector2::zero(),
            light_intensity: 1.0
        }))
    }

    fn new_square(x1: f64, y1: f64, x2: f64, y2: f64) -> Rc<RefCell<Region>> {
        let region = Region::new();
        {
            let mut m_region = region.borrow_mut();

            let (x1, x2) = (x1.max(x2), x1.min(x2));
            let (y1, y2) = (y1.min(y2), y1.max(y2));

            // Top
            let line = LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x2, y1));
            m_region.walls.push(Wall {
                normal: line.normal(),
                line,
                has_next: false,
                next_region: None,
                next_wall: None,
                region: Rc::downgrade(&region)
            });

            // Bottom
            let line = LineSegment2::of(Vector2::of(x1, y2), Vector2::of(x2, y2));
            m_region.walls.push(Wall {
                normal: line.normal().negate(),
                line,
                has_next: false,
                next_region: None,
                next_wall: None,
                region: Rc::downgrade(&region)
            });

            // Left
            let line = LineSegment2::of(Vector2::of(x2, y1), Vector2::of(x2, y2));
            m_region.walls.push(Wall {
                normal: line.normal(),
                line,
                has_next: false,
                next_region: None,
                next_wall: None,
                region: Rc::downgrade(&region)
            });

            // Right
            let line = LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x1, y2));
            m_region.walls.push(Wall {
                normal: line.normal().negate(),
                line,
                has_next: false,
                next_region: None,
                next_wall: None,
                region: Rc::downgrade(&region)
            });

            m_region.light_pos = m_region.walls[0].line.a.add(&m_region.walls[0].line.direction().scale(-0.25).add(&m_region.walls[2].line.direction().scale(-0.25)));
        }

        region
    }
}

pub(crate) struct Wall {
    pub(crate) line: LineSegment2,
    pub(crate) has_next: bool,
    pub(crate) next_region: Option<usize>,
    pub(crate) next_wall: Option<usize>,
    pub(crate) normal: Vector2,
    pub(crate) region: Weak<RefCell<Region>>
}

impl Wall {
    pub(crate) fn scale_factor(from: &Wall, to: &Wall) -> f64 {
        to.line.length() / from.line.length()
    }

    // transform to same position but relative to the new wall, accounting for walls of different sizes.
    pub(crate) fn translate(pos: Vector2, from: &Wall, to: &Wall) -> Vector2 {
        let last_offset = pos.subtract(&from.line.a);
        let fraction = last_offset.length() / from.line.direction().length();
        let new_offset = to.line.direction().negate().scale(fraction);

        to.line.a.add(&new_offset)
    }

    pub(crate) fn rotate(direction: Vector2, from: &Wall, to: &Wall) -> Vector2 {
        let delta_rad = to.line.normal().angle() - from.line.normal().angle();
        direction.rotate(delta_rad)
    }
}
