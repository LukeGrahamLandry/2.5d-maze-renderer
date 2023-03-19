use std::{collections::HashSet, hash::Hash};
use std::hash::Hasher;
use std::ops::Index;

use crate::{mth::{Vector2}, world::World};
use crate::material::{Colour};
use crate::world::{LightSource, Region, Wall};


impl World {
    pub(crate) fn recalculate(&mut self) {
        let mut new_cache: LightCache = LightCache::empty();

        for region in &old_cache.lights {
            new_cache.add(region.region);
        }

        let lights: HashSet<PortalLight> = old_cache.collect_lights();
        LightCache::insert_portal_lights(map, &mut new_cache, lights);

        new_cache
    }


    pub(crate) fn add(&mut self, region: & MapRegion) {
        self.lights.push(LightingRegion {
            region,
            portal_lights: Vec::new(),
            dynamic_walls: Default::default(),
            dynamic_lights: Default::default(),
        })
    }

    pub(crate) fn collect_lights(& self) -> HashSet<PortalLight> {
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
    fn insert_portal_lights(map: & Map, new_cache: &mut LightCache, portal_lights: HashSet<PortalLight>) {
        for light in portal_lights {
            let new_light: PortalLight = PortalLight {
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
    fn trace_direct_light(& self, light: & MapLight, found_lights: &mut HashSet<PortalLight>){
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

    pub(crate) fn get_lighting_region(& self, map_region: & MapRegion) -> & LightingRegion {
        self.lights.index(map_region.index)
    }
}

// this doesn't need a recursion limit, because the HashSet prevents loops.
// TODO: this will just reset everything in the whole world. Need to be smarter about which can see each other.
fn find_lights_recursively(region: &Region, mut found_walls: &mut HashSet<&Wall>, mut found_lights: &mut HashSet<&LightSource>){
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

impl Into<& MapRegion> for & LightingRegion{
    fn into(self) -> & MapRegion {
        self.region
    }
}

impl Hash for PortalLight {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for PortalLight {}
impl PartialEq for PortalLight {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
