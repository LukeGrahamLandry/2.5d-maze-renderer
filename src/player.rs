use std::f64::consts::PI;
use std::sync::RwLock;

use sdl2::keyboard::Keycode;
use crate::entity::{SquareEntity};
use crate::material::Material;
use crate::mth::{LineSegment2, Vector2};
use crate::world::{Wall, World};


const MOVE_SPEED: f64 = 100.0;
const TURN_SPEED: f64 = 0.002;
const PLAYER_SIZE: f64 = 4.0;

pub(crate) struct Player {
    pub(crate) entity: SquareEntity,
    pub(crate) look_direction: Vector2,
    pub(crate) move_direction: Vector2,
    pub(crate) has_flash_light: bool,
    pub(crate) portals: [Option<Wall>; 2],
    pub(crate) needs_render_update: RwLock<bool>,
    pub(crate) first_person_rendering: bool
}

impl Player {
    pub(crate) fn new(start_region: usize, pos: Vector2) -> Player {
        Player {
            entity: SquareEntity {
                id: 0,
                bb_ids: [1, 2, 3, 4],
                pos,
                region: start_region,
                radius: 0.0,
                material: Material::new(1.0, 0.1, 0.1),
            },
            look_direction: Vector2::of(0.0, -1.0),
            move_direction: Vector2::zero(),
            has_flash_light: false,
            portals: [None, None],
            needs_render_update: RwLock::new(true),
            first_person_rendering: false
        }
    }

    pub(crate) fn update(world: &mut World, pressed: &Vec<Keycode>, delta_time: f64, delta_mouse: i32) -> bool {
        let moved = {
            world.player_mut().update_direction(pressed, delta_mouse)
        };

        if moved {
            let dir = world.player_mut().move_direction;
            let move_direction= Player::handle_collisions(world, dir);

            let player = world.player_mut();
            player.entity.pos.x += move_direction.x * delta_time * MOVE_SPEED;
            player.entity.pos.y += move_direction.y * delta_time * MOVE_SPEED;
            *player.needs_render_update.write().unwrap() = true;
        }

        moved
    }

    pub(crate) fn handle_collisions(world: &mut World, mut move_direction: Vector2) -> Vector2 {
        let player = &mut world.player;
        let region = &world.regions[player.entity.region];
        let player_size = 11.0;
        let ray = LineSegment2::from(player.entity.pos, move_direction.scale(player_size));

        let mut wall = None;
        for check_wall in region.walls() {
            let hit_pos = check_wall.line().intersection(&ray);
            if !hit_pos.is_nan() {
                wall = Some(check_wall.clone());
            }
        }

        match wall {
            None => {
                return move_direction
            }
            Some(wall) => {
                let hit_pos = wall.line().intersection(&ray);
                let t = wall.line().t_of(&hit_pos).abs();
                let hit_edge = t < 0.01 || t > 0.99;
                let hit_back = wall.normal().dot(&move_direction) > 0.0;
                let wall_dir_unit = wall.line().direction().normalize();
                let slide_direction = wall_dir_unit.scale(move_direction.dot(&wall_dir_unit));

                if hit_back || hit_edge {
                    move_direction = slide_direction;
                } else {
                    match &wall.portal() {
                        None => {
                            move_direction = slide_direction;
                        }
                        Some(portal) => {
                            // TODO: tell the region that the entity switched
                            player.entity.region = portal.to_region;

                            player.entity.pos = portal.translate(player.entity.pos);
                            player.look_direction = portal.rotate(player.look_direction);
                            player.move_direction = portal.rotate(player.move_direction);
                            player.entity.pos = player.entity.pos.add(&move_direction);
                        }
                    }
                }
            }
        }

        if move_direction.length() > 0.1 {
            Player::handle_collisions(world, move_direction)
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
}
