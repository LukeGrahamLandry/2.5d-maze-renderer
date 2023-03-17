use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use crate::material::{Colour, Material};
use crate::mth::{EPSILON, LineSegment2, Vector2};
use crate::new_world::World;
use crate::ray::{HitKind, HitResult, ray_trace, single_ray_trace, trace_clear_path_between};
use crate::shelf::{Shelf, ShelfPtr};

impl World {
    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>, delta_mouse: i32){
        self.player.borrow_mut().update(&pressed, delta_time, delta_mouse);
    }

    pub(crate) fn on_mouse_click(&mut self, mouse_button: MouseButton) {
        let direction = self.player.borrow().look_direction;
        let segments = ray_trace(self.player.borrow().pos, direction , &self.player.borrow().region.borrow());
        let hit = &segments[segments.len() - 1];

        match &hit.kind {
            HitKind::HitNone => {}
            HitKind::HitPlayer { .. } => {}
            HitKind::HitWall {hit_wall, ..} => {
                let new_portal = {
                    let hit_wall = hit_wall.borrow();
                    let half_portal_direction = hit_wall.line.direction().normalize().scale(10.0);
                    let normal = if direction.dot(&hit_wall.normal) < 0.0 {
                        hit_wall.normal
                    } else {
                        hit_wall.normal.negate()
                    };
                    let start_point = hit.line.b.add(&half_portal_direction).add(&normal.scale(10.0));
                    let end_point = hit.line.b.subtract(&half_portal_direction).add(&normal.scale(10.0));

                    let wall = hit_wall.region.borrow_mut().new_wall(LineSegment2::of(start_point, end_point), normal, Material::new(0.8, 0.3, 0.3));
                    wall
                };  // Drop the borrow of the hit_wall, incase the ray tracing ran out of depth at a portal. Lets us re-borrow in place_portal.


                match mouse_button {
                    MouseButton::Left => {
                        self.update_player_portal(new_portal, 0, 1);
                    }
                    MouseButton::Right => {
                        self.update_player_portal(new_portal, 1, 0);
                    }
                    MouseButton::Middle => {
                        self.player.borrow().region.borrow_mut().remove_wall(&new_portal.borrow());
                        self.player.borrow_mut().clear_portal(0);
                        self.player.borrow_mut().clear_portal(1);
                    }
                    _ => { return; }
                }
            }
        }

        let player = self.player.borrow();
        *player.needs_render_update.write().unwrap() = true;
        Region::recalculate_lighting(player.region.clone());
    }

    fn update_player_portal(&mut self, new_portal: ShelfPtr<Wall>, replacing_index: usize, connecting_index: usize) {
        let mut player = self.player.borrow_mut();

        // If the player already had a portal placed in this slot, remove it.
        player.clear_portal(replacing_index);

        // Put the new portal in the player's slot.
        player.portals[replacing_index] = Some(new_portal.clone());

        // If there's a portal in the other slot, connect them.
        match &player.portals[connecting_index] {
            None => {}
            Some(connecting_portal) => {
                Wall::bidirectional_portal(&mut new_portal.borrow_mut(), &mut connecting_portal.borrow_mut())
            }
        }
    }
}

impl Region {
    pub(crate) fn new_square(x1: f64, y1: f64, x2: f64, y2: f64) -> Shelf<Region> {
        let region = Region::new(Material::new(0.2, 0.8, 0.8));
        {
            let mut m_region = region.borrow_mut();

            let walls = LineSegment2::new_square(x1, y1, x2, y2);
            for i in 0..4 {
                m_region.new_wall(walls[i], if i % 2 == 0 { walls[i].normal() } else { walls[i].normal().negate() }, Material::new(0.2, 0.2, 0.9));
            }

            // Put a light somewhere random so I can see the shading
            let (pos, intensity) = {
                let wall0 = m_region.get_wall(0);
                let wall2 = m_region.get_wall(2);
                let pos = wall0.line.a.add(&wall0.line.direction().scale(-0.25).add(&wall2.line.direction().scale(-0.25)));
                (pos, Colour::white())
            };
            m_region.new_light(intensity, pos);
        }

        region
    }
}

