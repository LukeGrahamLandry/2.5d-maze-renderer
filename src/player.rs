use std::f64::consts::PI;


use sdl2::keyboard::Keycode;


use crate::mth::{LineSegment2, Vector2};
use crate::ray::{HitKind, HitResult, VIEW_DIST};

use crate::world_data::{Player, Wall, WorldThing};

const MOVE_SPEED: f64 = 100.0;
const TURN_SPEED: f64 = 0.002;
const PLAYER_SIZE: f64 = 4.0;

impl Player {
    pub(crate) fn update(&mut self, pressed: &Vec<Keycode>, delta_time: f64, delta_mouse: i32) {
        if self.update_direction(pressed, delta_mouse) {
            let move_direction = self.handle_collisions(self.move_direction);

            self.pos.x += move_direction.x * delta_time * MOVE_SPEED;
            self.pos.y += move_direction.y * delta_time * MOVE_SPEED;
            self.update_bounding_box();
        }
    }

    pub(crate) fn handle_collisions(&mut self, mut move_direction: Vector2) -> Vector2 {
        let player_size = 11.0;
        let ray = LineSegment2::from(self.pos, move_direction.scale(player_size));

        {
            let mut wall = None;
            let region = self.region.borrow();
            for check_wall in region.iter_walls() {
                let hit_pos = check_wall.line.intersection(&ray);
                if !hit_pos.is_nan() {
                    wall = Some(check_wall.clone());
                }
            }


            match wall {
                None => {
                    return move_direction
                }
                Some(wall) => {
                    let hit_pos = wall.line.intersection(&ray);
                    let t = wall.line.t_of(&hit_pos).abs();
                    let hit_edge = t < 0.01 || t > 0.99;
                    let hit_back = wall.normal.dot(&move_direction) > 0.0;

                    if wall.get_next_wall().is_none() || hit_back || hit_edge {
                        move_direction = wall.line.direction().normalize().scale(move_direction.dot(&wall.line.direction().normalize()));
                    } else {
                        let next_wall = wall.get_next_wall().unwrap();
                        let next_region = next_wall.borrow().region.clone();

                        if self.region != next_region {
                            self.region.borrow_mut().remove_thing(self.get_myself());
                            next_region.borrow_mut().add_thing(self.get_myself());
                            self.region = next_region;
                        }

                        let next_wall = next_wall.borrow();
                        self.pos = Wall::translate(self.pos, &wall, &next_wall);
                        self.look_direction = Wall::rotate(self.look_direction, &wall, &next_wall);
                        move_direction = Wall::rotate(move_direction, &wall, &next_wall);
                        self.pos = self.pos.add(&move_direction);
                    }
                }
            }
        }

        if move_direction.length() > 0.1 {
            self.handle_collisions(move_direction)
        } else {
            move_direction
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
        let needs_physics_update = !relative_move_direction.is_zero();

        if delta_mouse != 0 || needs_physics_update {
            *self.needs_render_update.write().unwrap() = true;
        }

        needs_physics_update
    }

    pub(crate) fn clear_portal(&mut self, portal_index: usize) {
        match self.portals[portal_index].as_mut() {
            None => {}
            Some(portal) => {
                portal.borrow_mut().region.borrow_mut().remove_wall(&portal.borrow());
            }
        }
        self.portals[portal_index] = None;
    }

    pub(crate) fn update_bounding_box(&mut self) {
        let s = PLAYER_SIZE / 2.0;
        self.bounding_box = LineSegment2::new_square(self.pos.x - s, self.pos.y - s, self.pos.x + s, self.pos.y + s);
        *self.needs_render_update.write().unwrap() = true;
    }

    pub(crate) fn collide_bounding_box(&self, origin: Vector2, direction: Vector2) -> HitResult {
        let empty = HitResult::empty(self.region.clone(), origin, direction);
        if origin.subtract(&self.pos).length() < (PLAYER_SIZE / 2.0)  {
            return empty;
        }

        let ray = LineSegment2::from(origin, direction.scale(VIEW_DIST));

        let mut shortest_hit_distance = f64::INFINITY;
        let mut closest_hit_point = Vector2::NAN;
        let mut hit_side = None;

        for side in &self.bounding_box {
            let hit = side.intersection(&ray);
            let to_hit = origin.subtract(&hit);
            if !hit.is_nan() && to_hit.length() < shortest_hit_distance {
                hit_side = Some(side);
                shortest_hit_distance = to_hit.length();
                closest_hit_point = hit;
            }
        }

        match hit_side {
            None => {
                empty
            }
            Some(hit_side) => {
                HitResult {
                    region: self.region.clone(),
                    line: LineSegment2::of(origin, closest_hit_point),
                    kind: HitKind::HitPlayer {
                        box_side: hit_side.clone()
                    }
                }
            }
        }
    }
}
