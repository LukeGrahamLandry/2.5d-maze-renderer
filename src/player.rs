use std::f64::consts::PI;
use std::sync::RwLock;
use winit::event::MouseButton;

use crate::entity::SquareEntity;
use crate::game::Keys;
use crate::material::{Colour, Material};
use crate::mth;
use crate::mth::{LineSegment2, Vector2};
use crate::ray::RaySegment;
use crate::world::{Portal, Wall, World};
use maze::rand;

const MOVE_SPEED: f64 = 100.0;
const TURN_SPEED: f64 = 0.002;
const PLAYER_SIZE: f64 = 4.0;

#[derive(Copy, Clone)]
pub(crate) struct WallRef {
    region: usize,
    wall: usize,
}

pub(crate) struct Player {
    pub(crate) entity: SquareEntity,
    pub(crate) look_direction: Vector2,
    pub(crate) move_direction: Vector2,
    pub(crate) has_flash_light: bool,
    pub(crate) portals: [Option<WallRef>; 2],
    pub(crate) needs_render_update: RwLock<bool>,
    pub(crate) first_person_rendering: bool,
}

impl Player {
    pub(crate) fn new(start_region: usize, pos: Vector2) -> Player {
        Player {
            entity: SquareEntity {
                id: 0,
                bb_ids: [maze::rand(), maze::rand(), maze::rand(), maze::rand()],
                pos,
                region: start_region,
                radius: 1.0,
                material: Material::new(1.0, 0.1, 0.1),
            },
            look_direction: Vector2::of(0.0, -1.0),
            move_direction: Vector2::zero(),
            has_flash_light: false,
            portals: [None, None],
            needs_render_update: RwLock::new(true),
            first_person_rendering: true,
        }
    }

    pub(crate) fn update(
        world: &mut World,
        pressed: &Keys,
        delta_time: f64,
        delta_mouse: i32,
    ) -> bool {
        let moved = { world.player_mut().update_direction(pressed, delta_mouse) };

        if moved {
            // let bb_ids = world.player().entity.bb_ids;
            // bb_ids.iter().for_each(|w| world.remove_wall(world.player().entity.region, *w));

            let dir = world.player_mut().move_direction;
            let move_direction = Player::handle_collisions(world, dir, 100.0);

            let player = world.player_mut();
            player.entity.pos.x += move_direction.x * delta_time * MOVE_SPEED;
            player.entity.pos.y += move_direction.y * delta_time * MOVE_SPEED;
            *player.needs_render_update.write().unwrap() = true;

            // let bb = world.player.entity.get_bounding_box();
            // bb.into_iter().for_each(|w| world.add_wall(w));
        }

        moved
    }

    pub(crate) fn handle_collisions(
        world: &mut World,
        mut move_direction: Vector2,
        max_dist_sq: f64,
    ) -> Vector2 {
        let player = &mut world.player;
        let region = &world.regions[player.entity.region];

        let hit = region.single_ray_trace(player.entity.pos, move_direction);
        if hit.line.length_sq() > max_dist_sq {
            return move_direction;
        }
        match hit.hit_wall {
            None => {}
            Some(wall_index) => {
                let wall = region.get_wall(wall_index);
                let hit_back = wall.normal().dot(&move_direction) > 0.0;
                let wall_dir_unit = wall.line().direction().normalize();
                let slide_direction = wall_dir_unit.scale(move_direction.dot(&wall_dir_unit));

                if hit_back {
                    move_direction = slide_direction;
                } else {
                    match wall.portal() {
                        None => {
                            move_direction = slide_direction;
                        }
                        Some(portal) => {
                            // TODO: tell the region that the entity switched
                            player.entity.region = portal.to_region;

                            let walking_backwards =
                                wall.normal().dot(&player.look_direction) > mth::EPSILON;

                            player.entity.pos = portal.translate(player.entity.pos);
                            player.look_direction = portal.rotate(player.look_direction);
                            move_direction = portal.rotate(move_direction);

                            if walking_backwards {
                                player.look_direction = player.look_direction.negate();
                            }

                            player.entity.pos = player.entity.pos.add(&move_direction);
                            return move_direction;
                        }
                    }
                }
            }
        }

        if move_direction.length() > 0.1 {
            Player::handle_collisions(world, move_direction.scale(0.9), max_dist_sq)
        } else {
            Vector2::zero()
        }
    }

    fn update_direction(&mut self, pressed: &Keys, delta_mouse: i32) -> bool {
        let mut relative_move_direction = Vector2::zero();
        self.has_flash_light = false;
        if pressed.w {
            relative_move_direction.y = 1.0;
        }
        if pressed.s {
            relative_move_direction.y = -1.0;
        }
        if pressed.a {
            relative_move_direction.x = 1.0;
        }
        if pressed.d {
            relative_move_direction.x = -1.0;
        }
        if pressed.f {
            self.has_flash_light = true;
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

    pub(crate) fn mouse_click(world: &mut World, mouse_button: MouseButton) {
        let direction = world.player().look_direction;
        let hit: RaySegment = {
            let segments = world.ray_trace(
                world.player.entity.region,
                world.player().entity.pos,
                direction,
            );
            segments.last().unwrap().clone()
        };

        match hit.hit_wall {
            None => {}
            Some(hit_wall_index) => {
                let new_portal = {
                    let hit_wall = world.get_region(hit.region).get_wall(hit_wall_index);
                    let half_portal_direction = hit_wall.line.direction().normalize().scale(10.0);
                    let normal = if direction.dot(&hit_wall.normal) < 0.0 {
                        hit_wall.normal
                    } else {
                        hit_wall.normal.negate()
                    };
                    let bump_dist = 0.1;
                    let start_point = hit
                        .line
                        .b
                        .add(&half_portal_direction)
                        .add(&normal.scale(bump_dist));
                    let end_point = hit
                        .line
                        .b
                        .subtract(&half_portal_direction)
                        .add(&normal.scale(bump_dist));

                    let wall = Wall {
                        id: rand(),
                        region: hit.region,
                        line: LineSegment2::of(start_point, end_point),
                        normal,
                        material: Material::default(Colour::new(0.8, 0.3, 0.3)),
                        portal: None,
                    };

                    wall
                };

                match mouse_button {
                    MouseButton::Left => {
                        Player::place_portal(world, new_portal, 0, 1);
                    }
                    MouseButton::Right => {
                        Player::place_portal(world, new_portal, 1, 0);
                    }
                    MouseButton::Middle => {
                        Player::clear_portal(world, 0);
                        Player::clear_portal(world, 1);
                    }
                    _ => {
                        return;
                    }
                }
            }
        }

        world.update_lighting();
        *(world.player_mut().needs_render_update.write().unwrap()) = true;
    }

    pub(crate) fn clear_portal(world: &mut World, portal_index: usize) {
        let portal = world.player().portals[portal_index];
        match portal {
            None => {}
            Some(portal) => {
                world.regions[portal.region].walls.remove(&portal.wall);
            }
        }
        world.player_mut().portals[portal_index] = None;
    }

    pub(crate) fn place_portal(
        mut world: &mut World,
        mut portal: Wall,
        replacing_index: usize,
        connecting_index: usize,
    ) {
        // If the player already had a portal placed in this slot, remove it.
        Player::clear_portal(world, replacing_index);

        // Put the new portal in the player's slot.
        world.player_mut().portals[replacing_index] = Some(WallRef {
            region: portal.region,
            wall: portal.id,
        });

        // If there's a portal in the other slot, connect them.
        let connecting_portal = world.player().portals[connecting_index];
        match &connecting_portal {
            None => {}
            Some(connecting_portal) => {
                let other_portal = world.wall_mut(connecting_portal.region, connecting_portal.wall);
                portal.portal = Portal::new(&portal, other_portal);
                other_portal.portal = Portal::new(other_portal, &portal);
            }
        }

        // Add the new portal to the world.
        world.regions[portal.region].walls.insert(portal.id, portal);
    }
}
