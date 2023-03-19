use std::collections::HashMap;
use sdl2::keyboard::Keycode;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;


pub(crate) struct World {
    pub(crate) regions: Vec<Region>,
    player: Player
}

pub(crate) struct Region {
    pub(crate) id: usize,
    pub(crate) walls: HashMap<usize, Wall>,
    pub(crate) lights: HashMap<usize, LightSource>,
    pub(crate) floor_material: Material
}

pub(crate) enum Portal {
    NONE,
    PORTAL { next_wall: usize, next_region: usize }
}

pub(crate) struct Wall {
    pub(crate) id: usize,
    pub(crate) region: usize,
    pub(crate) line: LineSegment2,
    pub(crate) normal: Vector2,
    pub(crate) material: Material,
    pub(crate) portal: Portal
}

pub(crate) struct LightSource {
    pub(crate) id: usize,
    pub(crate) intensity: Colour,
    pub(crate) pos: Vector2,
    pub(crate) kind: LightKind
}

pub(crate) enum LightKind {
    DIRECT(),
    PORTAL { line: LineSegment2 }
}

impl Wall {
    pub(crate) fn portal(&self) -> &Portal {
        &self.portal
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

        world
    }

    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>, delta_mouse: i32){
        self.player.update(self, &pressed, delta_time, delta_mouse);
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
