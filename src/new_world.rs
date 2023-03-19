use std::marker::PhantomData;
use std::ops::Index;
use sdl2::keyboard::Keycode;
use crate::{map_builder::Map, light_cache::LightingRegion};
use crate::light_cache::{LightCache};
use crate::lighting::LightSource;
use crate::map_builder::{MapRegion, MapWall};
use crate::mth::Vector2;
use crate::player::Player;
use crate::ray::SolidWall;

pub(crate) struct World<'map> {
    pub(crate) map: Box<Map<'map>>,
    pub(crate) state: WorldState<'map, 'map>,
}

pub(crate) struct WorldState<'map: 'walls, 'walls> {
    pub(crate) lighting: LightCache<'map, 'walls>,
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

impl<'map: 'walls, 'walls> WorldState<'map, 'walls> {
    pub(crate) fn update<'new>(self, map: &'map Map<'map>) -> WorldState<'map, 'new> {
        let old_cache = self.lighting;
        WorldState {
            lighting: LightCache::recalculate(map, old_cache),
            players: vec![]
        }
    }
}

impl<'map: 'walls, 'walls> World<'map> {
    pub(crate) fn new<'w>(map: Map<'map>, start_region_index: usize, start_pos: Vector2) -> World<'map> {
        let mut world = World {
            map: Box::new(map),
            state: WorldState {
                lighting: LightCache::empty(),
                players: Vec::new()
            }
        };

        for region in world.map.regions(){
            world.state.lighting.add(region);
        }

        world.state.players.push(Player::new(world.map.as_ref().regions().index(start_region_index), start_pos));

        let new_state = {
            world.state.update(&world.map)
        };

        world.state = new_state;

        world
    }

    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>, delta_mouse: i32){
        self.state.players[0].update(&pressed, delta_time, delta_mouse);
    }

    pub(crate) fn get_lighting_region(&'map self, map_region: &'map MapRegion<'map>) -> &'walls LightingRegion {
        self.state.lighting.lights.index(map_region.index)
    }

    pub(crate) fn get_light_cache(&'map self) -> &'walls LightCache {
        &self.state.lighting
    }

    pub(crate) fn player(&'map self) -> &'walls Player {
        self.state.players.index(0)
    }
}

fn create_portal_pair() {
    // for the player's portals to be able to reference each other.
    // put them in boxes. never mutate. when the shoot just remake the other portal as well.
}
