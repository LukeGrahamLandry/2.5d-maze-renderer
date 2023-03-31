use std::cell::Cell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sdl2::keyboard::Keycode;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;


pub(crate) struct World {
    pub(crate) regions: Vec<Region>,
    pub(crate) player: Player
}

pub(crate) struct Region {
    pub(crate) id: usize,
    pub(crate) walls: HashMap<usize, Wall>,
    pub(crate) lights: HashMap<usize, LightSource>,
    pub(crate) floor_material: Material,
    pub(crate) lighting: FloorLightCache
}

pub(crate) struct FloorLightCache {
    pub(crate) floor_light_cache: Box<[Cell<Option<Colour>>]>,
    pub(crate) empty_floor_light_cache: Box<[Cell<Option<Colour>>]>,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) top_left: Vector2
}

#[derive(Clone, Copy)]
pub(crate) struct Portal {
    pub(crate) from_wall: usize,
    pub(crate) from_region: usize,
    pub(crate) to_wall: usize,
    pub(crate) to_region: usize,
    pub(crate) transform: Transformation
}

pub(crate) struct Wall {
    pub(crate) id: usize,
    pub(crate) region: usize,
    pub(crate) line: LineSegment2,
    pub(crate) normal: Vector2,
    pub(crate) material: Material,
    pub(crate) portal: Option<Portal>
}

#[derive(Clone, Copy)]
pub(crate) struct LightSource {
    pub(crate) id: usize,
    pub(crate) region: usize,
    pub(crate) intensity: Colour,
    pub(crate) pos: Vector2,
    pub(crate) kind: LightKind
}

#[derive(Clone, Copy)]
pub(crate) enum LightKind {
    DIRECT(),
    PORTAL { portal_line: LineSegment2 }
}

impl Wall {
    pub(crate) fn portal(&self) -> Option<&Portal> {
        self.portal.as_ref()
    }
    pub(crate) fn material(&self) -> &Material {
        &self.material
    }
    pub(crate) fn line(&self) -> LineSegment2 {
        self.line
    }
    pub(crate) fn normal(&self) -> Vector2 {
        self.normal
    }
    pub(crate) fn region(&self) -> usize {
        self.region
    }
}

impl World {
    pub(crate) fn new<'w>(regions: Vec<Region>, start_region_index: usize, start_pos: Vector2) -> World {
        let mut world = World {
            regions,
            player: Player::new(start_region_index, start_pos)
        };

        // let bb = world.player.entity.get_bounding_box();
        // bb.into_iter().for_each(|w| world.add_wall(w));

        world.init_lighting();

        world
    }

    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>, delta_mouse: i32){
        Player::update(self, &pressed, delta_time, delta_mouse);
    }

    pub(crate) fn regions(&self) -> impl Iterator<Item=&Region> {
        self.regions.iter()
    }

    pub(crate) fn get_region(&self, id: usize) -> &Region {
        &self.regions[id]
    }

    pub(crate) fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    pub(crate) fn player(&self) -> &Player {
        &self.player
    }

    pub(crate) fn wall_mut(&mut self, region: usize, wall: usize) -> &mut Wall {
        self.regions[region].walls.get_mut(&wall).expect("Invalid wall index.")
    }

    pub(crate) fn remove_wall(&mut self, region: usize, wall: usize) {
        self.regions[region].walls.remove(&wall).expect("Invalid wall index");
    }

    pub(crate) fn add_wall(&mut self, wall: Wall) {
        self.regions[wall.region].walls.insert(wall.id, wall);
    }
}

impl Region {
    pub(crate) fn get_wall(&self, id: usize) -> &Wall {
        self.walls.get(&id).expect("Invalid wall id.")
    }

    pub(crate) fn walls(&self) -> impl Iterator<Item=&Wall> {
        self.walls.values()
    }

    pub(crate) fn lights(&self) -> impl Iterator<Item=&LightSource> {
        self.lights.values()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Transformation {
    pub(crate) to_line: LineSegment2,
    pub(crate) to_normal: Vector2,
    pub(crate) from_line: LineSegment2,
    pub(crate) from_normal: Vector2,
}

impl Portal {
    pub(crate) fn new(from_wall: &Wall, to_wall: &Wall) -> Option<Portal> {
        Some(Portal {
            to_region: to_wall.region,
            to_wall: to_wall.id,
            from_region: from_wall.region,
            from_wall: from_wall.id,
            transform: Transformation {
                to_line: to_wall.line(),
                to_normal: to_wall.normal(),
                from_line: from_wall.line(),
                from_normal: from_wall.normal(),
            }
        })
    }

    pub(crate) fn scale_factor(&self) -> f64 {
        // Calculate ratio of lengths with only one square root makes me feel very clever.
        (self.transform.to_line.length_sq() / self.transform.from_line.length_sq()).sqrt()
    }

    // transform to same position but relative to the new wall, accounting for walls of different sizes.
    pub(crate) fn translate(&self, pos: Vector2) -> Vector2 {
        let last_offset = pos.subtract(&self.transform.from_line.a);
        let fraction = last_offset.length() / self.transform.from_line.direction().length();
        let new_offset = self.transform.to_line.direction().negate().scale(fraction);
        self.transform.to_line.a.add(&new_offset)

    }

    // TODO: should try scaling the direction as well,
    //       if im not superfluously normalizing it during the ray tracing,
    //       would change the length of the basis unit vector which might look cool
    pub(crate) fn rotate(&self, dir: Vector2) -> Vector2 {
        let rot_offset = self.transform.from_normal.angle_between(&self.transform.to_normal.negate());
        let dir = dir.rotate(rot_offset);
        if dir.dot(&self.transform.to_normal) > 0.0 {
            dir
        } else {
            dir.negate()
        }
    }

    pub(crate) fn to_wall_line(&self) -> LineSegment2 {
        self.transform.to_line
    }
}

