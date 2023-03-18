use std::ops::Index;
use sdl2::keyboard::Keycode;
use crate::{map_builder::Map, light_cache::LightingRegion};
use crate::light_cache::{LightCache};
use crate::lighting::LightSource;
use crate::map_builder::{MapRegion, MapWall};
use crate::mth::Vector2;
use crate::player::Player;
use crate::ray::SolidWall;

pub(crate) struct World<'map, 'walls> {
    pub(crate) map: Map<'map>,
    pub(crate) lighting: LightCache<'map, 'walls>,
    pub(crate) regions: Vec<DynamicRegion<'map, 'walls>>,
    pub(crate) players: Vec<Player<'map, 'walls>>
}


// TODO
// I want this to be the main representation you pass around. So it should own the LightingRegion which is just a vec.
// then the methods in light_cache can act on DynamicRegion and just swap out the data in the LightingRegion.
// how to make all SolidWall impls know about their DynamicRegion. better if they only need to know the LightingRegion
// and you just need the DynamicRegion for building the light cache.
pub(crate) struct DynamicRegion<'map, 'walls> {
    pub(crate) map: &'map MapRegion<'map>,
    pub(crate) lighting: LightingRegion<'map, 'walls>,
}

impl<'map, 'walls> World<'map, 'walls> {
    pub(crate) fn new(map: Map<'map>, start_region_index: usize, start_pos: Vector2) -> World<'map, 'walls> {
        let mut world = World {
            map,
            lighting: LightCache::empty(),
            regions: Vec::new(),
            players: Vec::new()
        };

        for region in map.regions(){
            world.lighting.add(region);
        }

        world.players.push(Player::new(world.map.regions().index(start_region_index), start_pos));

        world = world.update_lighting();

        world
    }

    pub(crate) fn update_lighting<'new>(mut self) -> World<'map, 'new> {
        self.lighting = self.lighting.recalculate();
        self
    }

    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>, delta_mouse: i32){
        self.players[0].update(&pressed, delta_time, delta_mouse);
    }

    pub(crate) fn get_lighting_region(&self, map_region: &MapRegion) -> &'walls LightingRegion {
        self.lighting.lights.index(map_region.index)
    }
}

fn create_portal_pair() {
    // for the player's portals to be able to reference each other.
    // put them in boxes. never mutate. when the shoot just remake the other portal as well.
}
