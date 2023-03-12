use std::cell::RefCell;
use std::f64::consts::PI;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use sdl2::keyboard::Keycode;

use crate::mth::{LineSegment2, Vector2};
use crate::world::{Region, Wall};

pub(crate) struct Player {
    pub(crate) pos: Vector2,
    pub(crate) look_direction: Vector2,
    pub(crate) move_direction: Vector2,
    pub(crate) region: Rc<RefCell<Region>>,
    pub(crate) has_flash_light: bool,
    pub(crate) portals: [Option<Rc<RefCell<Wall>>>; 2],
}

const MOVE_SPEED: f64 = 100.0;
const TURN_SPEED: f64 = 0.002;
impl Player {
    pub(crate) fn update(&mut self, pressed: &Vec<Keycode>, regions: &Vec<Rc<RefCell<Region>>>, delta_time: f64, delta_mouse: i32) {
        if self.update_direction(pressed, delta_mouse) {
            let move_direction = self.handle_collisions(regions, self.move_direction);

            self.pos.x += move_direction.x * delta_time * MOVE_SPEED;
            self.pos.y += move_direction.y * delta_time * MOVE_SPEED;
        }
    }

    pub(crate) fn handle_collisions(&mut self, regions: &Vec<Rc<RefCell<Region>>>, mut move_direction: Vector2) -> Vector2 {
        let player_size = 11.0;
        let ray = LineSegment2::from(self.pos, move_direction.scale(player_size));


        let mut wall = None;
        let region = self.region.clone();
        let m_region = region.borrow();
        for check_wall in m_region.walls.iter() {
            let hit_pos = check_wall.borrow().line.intersection(&ray);
            if !hit_pos.is_nan(){
                wall = Some(check_wall);
            }
        }

        match wall {
            None => {
                move_direction
            }
            Some(wall) => {
                let wall = wall.borrow();
                let hit_pos = wall.line.intersection(&ray);
                let t = wall.line.t_of(&hit_pos).abs();
                let hit_edge = t < 0.01 || t > 0.99;
                let hit_back = wall.normal.dot(&move_direction) > 0.0;

                if wall.next_wall.is_none() || hit_back || hit_edge {
                    move_direction = wall.line.direction().normalize().scale(move_direction.dot(&wall.line.direction().normalize()));
                } else {
                    let next_wall = wall.next_wall.as_ref().unwrap().upgrade().unwrap();
                    let next_region = next_wall.borrow().region.clone();
                    self.region = next_region.upgrade().unwrap().clone();
                    let next_wall = next_wall.borrow();
                    self.pos = Wall::translate(self.pos, &wall, &next_wall);
                    self.look_direction = Wall::rotate(self.look_direction, &wall, &next_wall);
                    move_direction = Wall::rotate(move_direction, &wall, &next_wall);
                    self.pos = self.pos.add(&move_direction);
                }

                if move_direction.length() > 0.1 {
                    self.handle_collisions(regions, move_direction)
                } else {
                    move_direction
                }
            }
        }
    }

    fn update_direction(&mut self, pressed: &Vec<Keycode>, delta_mouse: i32) -> bool {
        let mut relative_move_direction = Vector2::zero();
        self.has_flash_light = false;
        for key in pressed {
            match key {
                Keycode::W => {
                    relative_move_direction.y = 1.0;
                }
                Keycode::S => {
                    relative_move_direction.y = -1.0;
                }
                Keycode::A => {
                    relative_move_direction.x = 1.0;
                }
                Keycode::D => {
                    relative_move_direction.x = -1.0;
                },
                Keycode::F => {
                    self.has_flash_light = true;
                }
                _ => (),
            }
        }

        let move_angle = relative_move_direction.normalize().angle() - (PI / 2.0);
        self.look_direction = self.look_direction.rotate(delta_mouse as f64 * TURN_SPEED);
        self.move_direction = self.look_direction.rotate(move_angle);

        !relative_move_direction.is_zero()
    }

    pub(crate) fn clear_portal(&mut self, portal_index: usize) {
        match self.portals[portal_index].as_mut() {
            None => {}
            Some(portal) => {
                portal.borrow_mut().region.upgrade().unwrap().borrow_mut().remove_wall(&portal);
            }
        }
        self.portals[portal_index] = None;
    }
}

impl Player {
    pub(crate) fn new(start_region: &Rc<RefCell<Region>>) -> Player {
        Player {
            pos: Vector2::zero(),
            look_direction: Vector2::of(0.0, -1.0),
            move_direction: Vector2::zero(),
            region: start_region.clone(),
            has_flash_light: false,
            portals: [None, None],
        }
    }
}