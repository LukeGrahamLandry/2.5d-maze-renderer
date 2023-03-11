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
    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>){
        self.player.update(&pressed, &self.regions, delta_time);
    }

    pub(crate) fn create_example() -> World {
        let mut regions = vec![];

        regions.push(Region::new_square(100.0, 200.0, 300.0, 400.0));
        regions.push(Region::new_square(500.0, 200.0, 700.0, 400.0));
        regions.push(Region::new_square(50.0, 50.0, 150.0, 150.0));

        regions[0].borrow_mut().floor_color = Color::RGB(0, 50, 50);
        regions[1].borrow_mut().floor_color = Color::RGB(0, 50, 0);
        regions[2].borrow_mut().floor_color = Color::RGB(0, 0, 50);
        // regions[1].light_intensity = 0.5;
        // regions[2].light_intensity = 0.01;

        let line = LineSegment2::of(Vector2::of(200.0, 300.0), Vector2::of(200.0, 325.0));
        let wall = Wall::new(line, line.normal(), &regions[0]);
        wall.borrow_mut().next_wall = Some(Rc::downgrade(&regions[2].borrow().walls[1]));
        regions[0].borrow_mut().walls.push(wall);

        let line = LineSegment2::of(Vector2::of(175.0, 300.0), Vector2::of(175.0, 325.0));
        let wall = Wall::new(line, line.normal().negate(), &regions[0]);
        wall.borrow_mut().next_wall = Some(Rc::downgrade(&regions[2].borrow().walls[0]));
        regions[0].borrow_mut().walls.push(wall);


        regions[0].borrow_mut().walls[0].borrow_mut().next_wall = Some(Rc::downgrade(&regions[1].borrow().walls[1]));

        regions[1].borrow_mut().walls[1].borrow_mut().next_wall = Some(Rc::downgrade(&regions[0].borrow().walls[0]));

        regions[1].borrow_mut().walls[2].borrow_mut().next_wall = Some(Rc::downgrade(&regions[2].borrow().walls[3]));

        regions[2].borrow_mut().walls[3].borrow_mut().next_wall = Some(Rc::downgrade(&regions[1].borrow().walls[2]));

        let mut player = Player::new(&regions[0]);
        player.pos.x = 150.0;
        player.pos.y = 250.0;

        World {
            player,
            regions
        }
    }
}

pub(crate) struct Region {
    pub(crate) walls: Vec<Rc<RefCell<Wall>>>,
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


            // Since we're using the canvas coordinate system, down is positive y.
            let (x1, x2) = (x1.max(x2), x1.min(x2));
            let (y1, y2) = (y1.min(y2), y1.max(y2));

            // Top
            let line = LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x2, y1));
            m_region.walls.push(Wall::new(line, line.normal(), &region));

            // Bottom
            let line = LineSegment2::of(Vector2::of(x1, y2), Vector2::of(x2, y2));
            m_region.walls.push(Wall::new(line, line.normal().negate(), &region));

            // Left
            let line = LineSegment2::of(Vector2::of(x2, y1), Vector2::of(x2, y2));
            m_region.walls.push(Wall::new(line, line.normal(), &region));

            // Right
            let line = LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x1, y2));
            m_region.walls.push(Wall::new(line, line.normal().negate(), &region));

            // Put a light somewhere random so I can see the shading
            m_region.light_pos = {
                let wall0 = m_region.walls[0].borrow();
                let wall2 = m_region.walls[2].borrow();
                wall0.line.a.add(&wall0.line.direction().scale(-0.25).add(&wall2.line.direction().scale(-0.25)))
            }
        }

        region
    }
}

pub(crate) struct Wall {
    pub(crate) line: LineSegment2,
    pub(crate) normal: Vector2,
    pub(crate) region: Weak<RefCell<Region>>,
    pub(crate) next_wall: Option<Weak<RefCell<Wall>>>
}

impl Wall {
    pub(crate) fn new(line: LineSegment2, normal: Vector2, region: &Rc<RefCell<Region>>) -> Rc<RefCell<Wall>> {
        let wall = Wall {
            region: Rc::downgrade(&region),
            next_wall: None,
            normal,
            line,
        };
        Rc::new(RefCell::new(wall))
    }

    pub(crate) fn is_portal(&self) -> bool {
        self.next_wall.is_some()
    }

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
