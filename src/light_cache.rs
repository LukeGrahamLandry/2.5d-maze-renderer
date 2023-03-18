use std::{collections::HashSet, hash::Hash};
use std::collections::HashMap;
use std::hash::Hasher;
use std::ops::Index;

use crate::{map_builder::{MapLight, MapRegion, MapWall}, mth::{Vector2}, new_world::World, ray::{RaySegment, trace_clear_path_between}};
use crate::lighting::LightSource;
use crate::map_builder::Map;
use crate::material::{Colour, Material};
use crate::mth::LineSegment2;
use crate::ray::{find_shortest_path, Portal, SolidWall, trace_clear_portal_light};

pub(crate) struct LightCache<'a> {
    pub(crate) lights: Vec<LightingRegion<'a>>
}

/// Tracks all the visible lights in a region. Including those that can be seen through portals.
/// This allows lighting a point in a region without doing an expensive traversal of all portals to find lights.
/// The goal is that this can mutate because nothing ever references in. You just ask for the iterator when you need it.
pub(crate) struct LightingRegion<'a> {
    pub(crate) region: &'a MapRegion<'a>,
    pub(crate) portal_lights: Vec<PortalLight<'a>>,  // could be a set but i think iterating over vecs is easier
    dynamic_walls: HashMap<usize, Vec<Box<dyn SolidWall + Sync>>>,
    dynamic_lights: HashMap<usize, Vec<Box<dyn LightSource + Sync>>>
}

impl<'a> LightingRegion<'a> {
    pub(crate) fn update_entity(&mut self, id: usize, new_walls: Vec<Box<dyn SolidWall + Sync>>, new_lights: Vec<Box<dyn LightSource + Sync>>) {
        let old_walls = self.dynamic_walls.get(&id);
        let old_lights = self.dynamic_lights.get(&id);

        // any lights that hit the old wall
        // any walls hit by the old light

        self.dynamic_walls.insert(id, new_walls);
        self.dynamic_lights.insert(id, new_lights);
    }
}

/// A light seen through a portal.
pub(crate) struct PortalLight<'a> {
    id: usize,
    pub(crate) portal_in: &'a (dyn SolidWall + Sync),  // light goes in this portal
    pub(crate) portal_out: &'a (dyn SolidWall + Sync), // and comes out this portal
    /// The original light. There could be other PortalLights in between. It could be in the same region as either or neither portal.
    pub(crate) light: &'a MapLight<'a>,
    /// The position behind the out portal to where the light would be.
    pub(crate) fake_position: Vector2
}

impl<'a> LightCache<'a> {
    pub(crate) fn new(map: &Map) -> LightCache<'a> {
        let mut lights: Vec<LightingRegion> = map.regions().iter().map(|region| {
            LightingRegion {
                region,
                portal_lights: Vec::new(),
                dynamic_walls: Default::default(),
                dynamic_lights: Default::default(),
            }
        }).collect();

        LightCache {
            lights
        }
    }

    fn calculate_initial_lighting(&mut self){
        let mut portal_lights: HashSet<PortalLight> = HashSet::new();

        for region in &mut self.lights {
            for light in region.region.lights() {
                // There's a mutability dance going on here. It would be more space efficient to just directly put
                // each light in the correct region when its found instead of collecting them all first.
                // But I don't think I can mutate the objects while I'm looping over them.
                // Should be fine here but what about for chaining portals. TODO
                self.trace_direct_light(light, &mut portal_lights);
            }
        }

        self.insert_portal_lights(portal_lights);
    }

    // TODO: the OUT portal now needs to send the light to all the other portals in its region. With some limit on the recursion.
    //       Be smart about which ones need to propagate the change. Only additions or removals matter.
    /// Store an arbitrary set of portal lights on the region of their out_portal.
    fn insert_portal_lights(&mut self, portal_lights: HashSet<PortalLight>) {
        for light in portal_lights {
            let mut region = &mut self.lights[light.portal_out.region().index];
            region.portal_lights.push(light);
        }
    }

    /// Collect all times that a direct light in the region hits a portal.
    /// found_lights will contain PortalLights whose in_portal is in the same region as MapLight.
    fn trace_direct_light(&self, light: &MapLight, found_lights: &mut HashSet<PortalLight>){
        // For every portal, cast a ray from the light to every point on the portal. The first time one hits, we care.
        for wall in light.region.walls() {
            let line = wall.line;
            let normal = wall.normal;
            match wall.portal() {
                // If it's not a portal, we ignore it.
                None => {}
                Some(portal) => {
                    let segments = find_shortest_path(light.region, light.pos,normal, line);
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

    pub(crate) fn get_lighting_region(&self, map_region: &MapRegion) -> &LightingRegion {
        self.lights.index(map_region.index)
    }
}

impl<'a> LightSource for PortalLight<'a> {
    fn intensity(&self) -> Colour {
        self.light.intensity
    }

    fn apparent_pos(&self) -> &Vector2 {
        &self.fake_position
    }

    fn region(&self) -> &MapRegion {
        self.portal_out.region()
    }

    fn blocked_by_shadow(&self, hit_pos: &Vector2) -> bool {
        trace_clear_portal_light(self, *hit_pos).is_none()
    }
}

/// A MapLight acts only on the region that directly contains it.
/// Any other regions it should affect have a child PortalLight.
impl<'a> LightSource for MapLight<'a> {
    fn intensity(&self) -> Colour {
        self.intensity
    }

    fn apparent_pos(&self) -> &Vector2 {
        &self.pos
    }

    fn region(&self) -> &MapRegion {
        self.region
    }

    fn blocked_by_shadow(&self, hit_point: &Vector2) -> bool {
        trace_clear_path_between(self.pos, *hit_point, self.region).is_none()
    }
}

struct LightSourceIter<'a> {
    i: usize,
    region: &'a LightingRegion<'a>
}

impl<'a> Iterator for LightSourceIter<'a> {
    type Item = &'a dyn LightSource;

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


impl<'a> IntoIterator for &LightingRegion<'a> {
    type Item = &'a dyn LightSource;
    type IntoIter = LightSourceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LightSourceIter {
            i: 0,
            region: self
        }
    }
}

// this doesn't need a recursion limit, because the HashSet prevents loops.
// TODO: this will just reset everything in the whole world. Need to be smarter about which can see each other.
fn find_lights_recursively(region: &MapRegion, mut found_walls: &mut HashSet<&MapWall>, mut found_lights: &mut HashSet<&MapLight>){
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

impl<'a> Into<&'a MapRegion<'a>> for &'a LightingRegion<'a>{
    fn into(self) -> &'a MapRegion<'a> {
        self.region
    }
}

impl<'a> Hash for PortalLight<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<'a> Eq for PortalLight<'a> {}
impl<'a> PartialEq for PortalLight<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
