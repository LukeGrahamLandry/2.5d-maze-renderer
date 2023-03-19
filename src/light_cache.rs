use std::{collections::HashSet, hash::Hash};
use std::collections::HashMap;
use std::hash::Hasher;
use std::ops::Index;

use crate::{map_builder::{MapLight, MapRegion, MapWall}, mth::{Vector2}, new_world::World, ray::{RaySegment}};
use crate::lighting::LightSource;
use crate::map_builder::Map;
use crate::material::{Colour, Material};
use crate::mth::LineSegment2;
use crate::ray::{Portal, SolidWall};

pub(crate) struct LightCache<'map: 'walls, 'walls> {
    pub(crate) lights: Vec<LightingRegion<'map, 'walls>>
}

/// Tracks all the visible lights in a region. Including those that can be seen through portals.
/// This allows lighting a point in a region without doing an expensive traversal of all portals to find lights.
/// The goal is that this can mutate because nothing ever references in. You just ask for the iterator when you need it.
pub(crate) struct LightingRegion<'map: 'walls, 'walls> {
    pub(crate) region: &'map MapRegion<'map>,
    pub(crate) portal_lights: Vec<PortalLight<'map, 'walls>>,  // could be a set but i think iterating over vecs is easier
    dynamic_walls: HashMap<usize, Vec<Box<dyn SolidWall<'map, 'walls> + Sync>>>,
    dynamic_lights: HashMap<usize, Vec<Box<(dyn LightSource<'map, 'walls> + Sync)>>>
}

impl<'map: 'walls, 'walls> LightingRegion<'map, 'walls> {
    // pub(crate) fn update_entity(&'walls mut self, id: usize, new_walls: Vec<Box<dyn SolidWall + Sync + 'walls>>, new_lights: Vec<Box<dyn LightSource + Sync + 'walls>>) {
    //     let old_walls = self.dynamic_walls.get(&id);
    //     let old_lights = self.dynamic_lights.get(&id);
    //
    //     // any lights that hit the old wall
    //     // any walls hit by the old light
    //
    //     // self.dynamic_walls.insert(id, new_walls);
    //     // self.dynamic_lights.insert(id, new_lights);
    // }

    pub(crate) fn walls(&'map self) -> &'map Vec<MapWall<'map>> {
        &self.region.walls()
    }
}

/// A light seen through a portal.
pub(crate) struct PortalLight<'map, 'walls> {
    id: usize,
    pub(crate) portal_in: &'walls (dyn SolidWall<'map, 'walls> + Sync),  // light goes in this portal
    pub(crate) portal_out: &'walls (dyn SolidWall<'map, 'walls> + Sync), // and comes out this portal
    /// The original light. There could be other PortalLights in between. It could be in the same region as either or neither portal.
    pub(crate) light: &'map MapLight<'map>,
    /// The position behind the out portal to where the light would be.
    pub(crate) fake_position: Vector2
}

impl<'map: 'old + 'new, 'old, 'new> LightCache<'map, 'old> {
    pub(crate) fn empty() -> LightCache<'map, 'new> {
        LightCache {
            lights: Vec::new()
        }
    }

    pub(crate) fn recalculate(map: &'map Map<'map>, old_cache: LightCache<'map, 'old>) -> LightCache<'map, 'new> {
        let mut new_cache: LightCache<'map, 'new> = LightCache::empty();

        for region in &old_cache.lights {
            new_cache.add(region.region);
        }

        let lights: HashSet<PortalLight<'map, 'old>> = old_cache.collect_lights();
        LightCache::insert_portal_lights(map, &mut new_cache, lights);

        new_cache
    }


    pub(crate) fn add(&mut self, region: &'map MapRegion<'map>) {
        self.lights.push(LightingRegion {
            region,
            portal_lights: Vec::new(),
            dynamic_walls: Default::default(),
            dynamic_lights: Default::default(),
        })
    }

    pub(crate) fn collect_lights(&'old self) -> HashSet<PortalLight<'map, 'old>> {
        let mut portal_lights = HashSet::new();
        for region in &self.lights {
            for light in region.region.lights() {
                // There's a mutability dance going on here. It would be more space efficient to just directly put
                // each light in the correct region when its found instead of collecting them all first.
                // But I don't think I can mutate the objects while I'm looping over them.
                // Should be fine here but what about for chaining portals. TODO
                self.trace_direct_light(light, &mut portal_lights);
            }
        }
        portal_lights
    }

    // TODO: the OUT portal now needs to send the light to all the other portals in its region. With some limit on the recursion.
    //       Be smart about which ones need to propagate the change. Only additions or removals matter.
    /// Store an arbitrary set of portal lights on the region of their out_portal.
    /// needs to already have the new walls at this point
    fn insert_portal_lights(map: &'map Map<'map>, new_cache: &mut LightCache<'map, 'new>, portal_lights: HashSet<PortalLight<'map, 'old>>) {
        for light in portal_lights {
            let new_light: PortalLight<'map, 'new> = PortalLight {
                id: light.id,
                portal_in: &map.regions()[light.portal_in.region().index].walls()[0],
                portal_out: &map.regions()[light.portal_out.region().index].walls()[0],
                light: light.light,
                fake_position: light.fake_position,
            };
            new_cache.lights[light.portal_out.region().index].portal_lights.push(new_light);
        }
    }

    /// Collect all times that a direct light in the region hits a portal.
    /// found_lights will contain PortalLights whose in_portal is in the same region as MapLight.
    fn trace_direct_light(&'old self, light: &'map MapLight<'map>, found_lights: &mut HashSet<PortalLight<'map, 'old>>){
        // For every portal, cast a ray from the light to every point on the portal. The first time one hits, we care.
        for wall in light.region.walls() {
            let line = wall.line;
            let normal = wall.normal;
            match wall.portal() {
                // If it's not a portal, we ignore it.
                None => {}
                Some(portal) => {
                    let segments = self.get_lighting_region(light.region).find_shortest_path(light.pos,normal, line);
                    match segments {
                        // If the light doesn't hit it, we ignore it.
                        None => {}
                        Some(path) => {
                            let adjusted_origin =portal.translate(path.line.b);
                            // let adjusted_direction = MapWall::rotate(path.line.direction(), wall, next_wall).negate();

                            found_lights.insert(PortalLight {
                                id: maze::rand(),
                                portal_in: wall,
                                portal_out: portal.to_wall,
                                light,
                                fake_position: adjusted_origin,
                            });
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn get_lighting_region(&'old self, map_region: &'map MapRegion) -> &'old LightingRegion<'map, 'old> {
        self.lights.index(map_region.index)
    }
}

impl<'map: 'walls, 'walls> LightSource<'map, 'walls> for PortalLight<'map, 'walls> {
    fn intensity(&self) -> Colour {
        self.light.intensity
    }

    fn apparent_pos(&self) -> &Vector2 {
        &self.fake_position
    }

    fn region(&self) -> &MapRegion {
        self.portal_out.region()
    }

    fn blocked_by_shadow(&'walls self, region: &'walls LightingRegion<'map, 'walls>, hit_pos: &Vector2) -> bool {
        region.trace_clear_portal_light(self, *hit_pos).is_none()
    }
}

/// A MapLight acts only on the region that directly contains it.
/// Any other regions it should affect have a child PortalLight.
impl<'map: 'walls, 'walls> LightSource<'map, 'walls> for MapLight<'map> {
    fn intensity(&self) -> Colour {
        self.intensity
    }

    fn apparent_pos(&self) -> &Vector2 {
        &self.pos
    }

    fn region(&self) -> &MapRegion {
        self.region
    }

    fn blocked_by_shadow(&self, region: &'walls LightingRegion<'map, 'walls>, hit_point: &Vector2) -> bool {
        region.trace_clear_path_no_portals_between(self.pos, *hit_point).is_none()
    }
}

struct LightSourceIter<'map, 'walls> {
    i: usize,
    region: &'walls LightingRegion<'map, 'walls>
}

impl<'map: 'walls, 'walls> Iterator for LightSourceIter<'map, 'walls> {
    type Item = &'walls (dyn LightSource<'map, 'walls>);

    fn next(&mut self) -> Option<Self::Item> {
        let direct_count = self.region.region.lights().len();
        let indirect_count = self.region.portal_lights.len();
        if self.i < direct_count {
            Some(&self.region.region.lights()[self.i])
        } else if self.i < (direct_count + indirect_count){
            Some(&self.region.portal_lights[self.i - direct_count])
        } else {
            None
        }
    }
}


impl<'map: 'walls, 'walls> IntoIterator for &'walls LightingRegion<'map, 'walls> {
    type Item = &'walls dyn LightSource<'map, 'walls>;
    type IntoIter = LightSourceIter<'map, 'walls>;

    fn into_iter(self) -> Self::IntoIter {
        LightSourceIter {
            i: 0,
            region: self
        }
    }
}

// this doesn't need a recursion limit, because the HashSet prevents loops.
// TODO: this will just reset everything in the whole world. Need to be smarter about which can see each other.
fn find_lights_recursively<'map>(region: &'map MapRegion<'map>, mut found_walls: &mut HashSet<&MapWall<'map>>, mut found_lights: &mut HashSet<&MapLight<'map>>){
    for (i, light) in region.lights().iter().enumerate() {
        found_lights.insert(light);
    }

    for wall in region.walls() {
        match wall.next_wall {
            None => {}
            Some(next_wall) => {
                if found_walls.insert(wall) {
                    find_lights_recursively(next_wall.region.clone(), &mut found_walls, &mut found_lights);
                }
            }
        }
    }
}

impl<'map, 'walls> Into<&'map MapRegion<'map>> for &'map LightingRegion<'map, 'walls>{
    fn into(self) -> &'map MapRegion<'map> {
        self.region
    }
}

impl<'map, 'walls> Hash for PortalLight<'map, 'walls> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<'map, 'walls> Eq for PortalLight<'map, 'walls> {}
impl<'map, 'walls> PartialEq for PortalLight<'map, 'walls> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
