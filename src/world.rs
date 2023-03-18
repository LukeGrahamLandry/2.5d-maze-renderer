use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use crate::material::{Colour, Material};
use crate::mth::{EPSILON, LineSegment2, Vector2};
use crate::new_world::World;
use crate::ray::{HitKind, RaySegment, ray_trace, single_ray_trace, trace_clear_path_between};
use crate::shelf::{Shelf, ShelfPtr};

impl World {
    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>, delta_mouse: i32){
        self.player.update(&pressed, delta_time, delta_mouse);
    }

    pub(crate) fn on_mouse_click(&mut self, mouse_button: MouseButton) {
        let direction = self.player.look_direction;
        let segments = ray_trace(self.player.pos, direction , &self.player.region);
        let hit = &segments[segments.len() - 1];

        match &hit.kind {
            HitKind::HitNone => {}
            HitKind::HitPlayer { .. } => {}
            HitKind::HitWall {hit_wall, ..} => {
                let new_portal = {
                    let hit_wall = hit_wall;
                    let half_portal_direction = hit_wall.line.direction().normalize().scale(10.0);
                    let normal = if direction.dot(&hit_wall.normal) < 0.0 {
                        hit_wall.normal
                    } else {
                        hit_wall.normal.negate()
                    };
                    let start_point = hit.line.b.add(&half_portal_direction).add(&normal.scale(10.0));
                    let end_point = hit.line.b.subtract(&half_portal_direction).add(&normal.scale(10.0));

                    let wall = hit_wall.region.new_wall(LineSegment2::of(start_point, end_point), normal, Material::new(0.8, 0.3, 0.3));
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
                        self.player.region.remove_wall(&new_portal);
                        self.player.clear_portal(0);
                        self.player.clear_portal(1);
                    }
                    _ => { return; }
                }
            }
        }

        let player = self.player;
        *player.needs_render_update.write().unwrap() = true;
        Region::recalculate_lighting(player.region.clone());
    }

    fn update_player_portal(&mut self, new_portal: ShelfPtr<Wall>, replacing_index: usize, connecting_index: usize) {
        let mut player = self.player;

        // If the player already had a portal placed in this slot, remove it.
        player.clear_portal(replacing_index);

        // Put the new portal in the player's slot.
        player.portals[replacing_index] = Some(new_portal.clone());

        // If there's a portal in the other slot, connect them.
        match &player.portals[connecting_index] {
            None => {}
            Some(connecting_portal) => {
                Wall::bidirectional_portal(&mut new_portal, &mut connecting_portal)
            }
        }
    }
}

