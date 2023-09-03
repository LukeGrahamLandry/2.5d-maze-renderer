use std::cell::Cell;
use std::collections::HashMap;
use std::time::Instant;

use crate::{mth::Vector2, world::World};
use crate::material::Colour;
use crate::mth::{EPSILON};
use crate::world::{FloorLightCache, LightKind, LightSource, Region};
use crate::world::LightKind::PORTAL;

impl World {
    pub(crate) fn init_lighting(&mut self){
        let portal_hits = self.collect_portal_lights();
        for (_, portal_light) in portal_hits.into_iter() {
            self.add_portal_light(portal_light);
        }
    }

    fn collect_portal_lights(&self) -> HashMap<usize, LightSource> {
        let mut portal_hits = HashMap::new();
        for region in 0..self.regions.len() {
            let region = self.get_region(region);
            for light in region.lights(){
                region.trace_portal_light(light, &mut portal_hits);
            }
        }
        portal_hits
    }

    // TODO: be smart about which parts actually need to be recomputed
    pub(crate) fn update_lighting(&mut self){
        for region in self.regions.iter_mut() {
            let mut portal_lights = vec![];

            for (id, light) in region.lights.iter() {
                match light.kind {
                    LightKind::DIRECT() => {}
                    PORTAL { .. } => {
                        portal_lights.push(*id);
                    }
                }
            }

            for id in portal_lights {
                region.lights.remove(&id);
            }
        }

        self.init_lighting();

        for region in &mut self.regions {
            region.clear_floor_lighting_cache();
        }
    }

    fn add_portal_light(&mut self, portal_light: LightSource) {
        self.regions[portal_light.region].lights.insert(portal_light.id, portal_light);
    }
}

impl Region {
    /// Collect all times that a light hits a portal in its region.
    fn trace_portal_light(&self, light: &LightSource, found: &mut HashMap<usize, LightSource>){
        assert_eq!(self.id, light.region);
        for wall in self.walls() {
            match wall.portal() {
                // If it's not a portal, we ignore it.
                None => {}
                Some(portal) => {
                    // Check a bunch of points on the wall.
                    let sample_count = (wall.line().length() / Region::PORTAL_SAMPLE_LENGTH).floor();
                    for i in 0..(sample_count as i32) {
                        let t = i as f64 / sample_count;
                        let wall_point = wall.line().at_t(t);
                        let dir_to_light = light.pos.subtract(&wall_point);

                        // If it hit the back, it didnt go through the portal
                        let hits_front = dir_to_light.dot(&wall.normal()) > EPSILON;
                        if !hits_front {
                            continue;
                        }

                        if !light.blocked_by_shadow(self, &wall_point) {
                            // if there's a clear path, add it as a portal light in the next region
                            let offset = wall.line().middle().subtract(&light.pos);
                            let offset = portal.rotate(offset);
                            let new_pos = portal.to_wall_line().middle().subtract(&offset);

                            let portal_light = LightSource {
                                id: maze::rand(),
                                region: portal.to_region,
                                intensity: light.intensity,
                                pos: new_pos,
                                kind: PORTAL {
                                    portal_line: portal.to_wall_line()
                                },
                            };
                            found.insert(maze::rand(), portal_light);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn new_light_cache(min: Vector2, max: Vector2) -> FloorLightCache {
        let min = min.subtract(&Vector2::of(1.0, 1.0));
        let max = max.add(&Vector2::of(1.0, 1.0));
        let width = (max.x - min.x).abs().ceil() as usize;
        let height = (max.y - min.y).abs().ceil() as usize;

        let cache = Region::empty_light_cache(width * height);
        FloorLightCache {
            width,
            height,
            floor_light_cache: cache.clone(),
            empty_floor_light_cache: cache,
            top_left: min,
        }
    }

    fn empty_light_cache(count: usize) -> Box<[Cell<Option<Colour>>]> {
        let mut cache = Vec::with_capacity(count);
        for _ in 0..count {
            cache.push(Cell::new(None));
        }
        cache.into_boxed_slice()
    }

    pub(crate) fn horizontal_surface_colour_memoized(&self, pos: Vector2) -> Colour {
        let lighting = &self.lighting;
        let local = pos.subtract(&lighting.top_left);
        let x = local.x.floor() as usize;
        let y = local.y.floor() as usize;

        let outside = x >= lighting.width || y >= lighting.height;
        if outside {  // This shouldn't happen so make it look obviously wrong.
            Colour::rgb(255, 0, 255)
        } else {
            let cached = {
                lighting.floor_light_cache[y * lighting.width + x].get()
            };
            match cached {
                None => {
                    let colour = self.horizontal_surface_colour(pos);
                    lighting.floor_light_cache[y * lighting.width + x].replace(Some(colour));
                    colour
                }
                Some(colour) => {
                    colour
                }
            }
        }
    }

    pub(crate) fn clear_floor_lighting_cache(&mut self){
        let n = Instant::now();
        self.lighting.floor_light_cache.clone_from_slice(self.lighting.empty_floor_light_cache.as_ref());
        println!("reset lights in {} ms", (Instant::now() - n).as_millis());
    }
}
