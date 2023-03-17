use crate::{map_builder::Map, light_cache::LightingRegion};
use crate::light_cache::{LightCache};
use crate::map_builder::{MapRegion, MapWall};
use crate::mth::Vector2;

pub(crate) struct World<'a> {
    pub(crate) map: Map<'a>,
    pub(crate) lighting: LightCache<'a>,
    regions: Vec<DynamicRegion<'a>>
    // pub(crate) player: Player<'a>
}

pub(crate) struct DynamicRegion<'a> {
    pub(crate) map: &'a MapRegion<'a>,
    pub(crate) lighting: &'a LightingRegion<'a>
}

impl<'a> World<'a> {
    pub(crate) fn new(map: Map<'a>) -> World<'a> {
        let mut world = World {
            map,
            lighting: LightCache::new(&map),
            regions: Vec::with_capacity(map.regions().len())
        };

        world
    }
}

