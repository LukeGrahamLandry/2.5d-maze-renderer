use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use crate::camera::{HitKind, HitResult, ray_direction_for_x, ray_trace, SCREEN_WIDTH};

use crate::mth::{LineSegment2, Vector2};
use crate::player::{Player, WorldThing};

pub struct World {
    pub(crate) regions: Vec<Rc<RefCell<Region>>>,
    pub(crate) player: Rc<RefCell<Player>>
}

impl World {
    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>, delta_mouse: i32){
        self.player.borrow_mut().update(&pressed, &self.regions, delta_time, delta_mouse);
    }

    pub(crate) fn on_mouse_click(&mut self, mouse_button: MouseButton) {
        let direction = ray_direction_for_x((SCREEN_WIDTH / 2) as i32, &self.player.borrow().look_direction);
        let hit: HitResult = {
            let segments = ray_trace(self.player.borrow().pos, direction , &self.player.borrow().region);
            segments.last().unwrap().clone()
        };

        match &hit.kind {
            HitKind::None => {}
            HitKind::Player { .. } => {}
            HitKind::Wall {hit_wall, ..} => {
                let new_portal = {
                    let hit_wall = hit_wall.upgrade().unwrap();
                    let hit_wall = hit_wall.borrow();
                    let half_portal_direction = hit_wall.line.direction().normalize().scale(10.0);
                    let normal = if direction.dot(&hit_wall.normal) < 0.0 {
                        hit_wall.normal
                    } else {
                        hit_wall.normal.negate()
                    };
                    let start_point = hit.line.b.add(&half_portal_direction).add(&normal.scale(10.0));
                    let end_point = hit.line.b.subtract(&half_portal_direction).add(&normal.scale(10.0));

                    Wall::new(LineSegment2::of(start_point, end_point), normal, &hit_wall.region.upgrade().unwrap())
                };  // Drop the borrow of the hit_wall, incase the ray tracing ran out of depth at a portal. Lets us re-borrow in place_portal.

                match mouse_button {
                    MouseButton::Left => {
                        self.place_portal(new_portal, 0, 1);
                    }
                    MouseButton::Right => {
                        self.place_portal(new_portal, 1, 0);
                    }
                    MouseButton::Middle => {
                        self.player.borrow_mut().clear_portal(0);
                        self.player.borrow_mut().clear_portal(1);
                    }
                    _ => { return; }
                }
            }
        }
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
        player.update_bounding_box();
        let id = player.id;

        let player = Rc::new(RefCell::new(player));
        let weak_player = Rc::downgrade(&player);
        regions[0].borrow_mut().things.insert(id, weak_player);

        World {
            player,
            regions
        }
    }
    fn place_portal(&mut self, new_portal: Rc<RefCell<Wall>>, replacing_index: usize, connecting_index: usize) {
        let mut player = self.player.borrow_mut();

        // If the player already had a portal placed in this slot, remove it.
        player.clear_portal(replacing_index);

        // Put the new portal in the player's slot.
        player.portals[replacing_index] = Some(new_portal.clone());

        // If there's a portal in the other slot, connect them.
        match &player.portals[connecting_index] {
            None => {}
            Some(connecting_portal) => {
                new_portal.borrow_mut().next_wall = Some(Rc::downgrade(connecting_portal));
                connecting_portal.borrow_mut().next_wall = Some(Rc::downgrade(&new_portal));
            }
        }

        // Add the new portal to the world.
        let region = new_portal.borrow().region.upgrade().unwrap();
        let mut region = region.borrow_mut();
        region.walls.push(new_portal);
    }
}

#[derive(Debug)]
pub(crate) struct Region {
    pub(crate) walls: Vec<Rc<RefCell<Wall>>>,
    pub(crate) floor_color: Color,
    pub(crate) light_pos: Vector2,
    pub(crate) light_intensity: f64,
    pub(crate) things: HashMap<u64, Weak<RefCell<dyn WorldThing>>>
}

impl Region {
    pub(crate) fn remove_wall(&mut self, wall: &Rc<RefCell<Wall>>){
        let mut to_remove = None;
        for (i, w) in self.walls.iter().enumerate() {
            if Rc::ptr_eq(wall, w) {
                to_remove = Some(i);
                break;
            }
        }

        match to_remove {
            None => {}
            Some(i) => {
                self.walls.remove(i);
            }
        }
    }

    pub(crate) fn new() -> Rc<RefCell<Region>> {
        Rc::new(RefCell::new(Region {
            walls: vec![],
            floor_color: Color::RGB(0, 0, 0),
            light_pos: Vector2::zero(),
            light_intensity: 1.0,
            things: HashMap::with_capacity(1)
        }))
    }

    fn new_square(x1: f64, y1: f64, x2: f64, y2: f64) -> Rc<RefCell<Region>> {
        let region = Region::new();
        {
            let mut m_region = region.borrow_mut();

            let walls = LineSegment2::new_square(x1, y1, x2, y2);
            m_region.walls.push(Wall::new(walls[0], walls[0].normal(), &region));
            m_region.walls.push(Wall::new(walls[1], walls[1].normal().negate(), &region));
            m_region.walls.push(Wall::new(walls[2], walls[2].normal(), &region));
            m_region.walls.push(Wall::new(walls[3], walls[3].normal().negate(), &region));

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

#[derive(Debug)]
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
        let rot_offset = from.normal.angle_between(&to.normal.negate());
        let dir = direction.rotate(rot_offset);
        if dir.dot(&to.normal) > 0.0 {
            dir
        } else {
            dir.negate()
        }
    }
}
