use std::{collections::HashSet, hash::Hash, thread};
use std::collections::HashMap;
use std::hash::Hasher;
use std::ops::Index;
use std::sync::atomic::AtomicI8;
use std::sync::{Arc, RwLock};

use crate::{mth::{Vector2}, world::World};
use crate::material::{Colour};
use crate::mth::{EPSILON, LineSegment2};
use crate::ray::RaySegment;
use crate::world::{FloorLightCache, LightKind, LightSource, Portal, Region, Wall};
use crate::world::LightKind::PORTAL;

impl World {
    pub(crate) fn init_lighting(&mut self){
        let portal_hits = self.collect_portal_lights();
        for (id, portal_light) in portal_hits.into_iter() {
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

    pub(crate) fn new_light_cache(min: Vector2, max: Vector2) -> Arc<FloorLightCache> {
        let width = (max.x - min.x).abs().ceil() as usize;
        let height = (max.y - min.y).abs().ceil() as usize;

        Arc::new(FloorLightCache {
            width,
            height,
            floor_light_cache: Region::empty_light_cache(width * height),
            top_left: min,
        })
    }

    fn empty_light_cache(count: usize) -> Box<[RwLock<Option<Colour>>]> {
        let mut cache = Vec::with_capacity(count);
        for _ in 0..count {
            cache.push(RwLock::new(None));
        }
        cache.into_boxed_slice()
    }

    pub(crate) fn horizontal_surface_colour_memoized(&self, pos: Vector2) -> Colour {
        let lighting = &self.lighting;
        let local = pos.subtract(&lighting.top_left);
        let x = local.x.floor() as usize;
        let y = local.y.floor() as usize;

        let outside = x >= lighting.width || y >= lighting.height;
        if outside {
            Colour::black()
        } else {
            let cached = {
                lighting.floor_light_cache[y * lighting.width + x].read().unwrap().clone()
            };
            match cached {
                None => {
                    let colour = self.horizontal_surface_colour(pos);
                    let mut cached = lighting.floor_light_cache[y * lighting.width + x].write().unwrap();
                    *cached = Some(colour);
                    colour
                }
                Some(colour) => {
                    colour
                }
            }
        }
    }

    // this could take mut and just replace the whole lock object instead of checking to get locked write access every time which would be ~10x faster
    // but still noticeably slow for very large regions. putting it in a thread that gradually resets takes longer over all
    // but means the game can continue as its happening and you just get the wrong floor lighting for a tiny amount of time.
    // but it does mean that the whole light cache needs to be in an Arc because the thread can't hold the borrow of the region.
    // but the arc has a cost for cloning, not for reading so it doesnt matter.

    pub(crate) fn clear_floor_lighting_cache(&self){
        let lighting = self.lighting.clone();

        thread::spawn(move || {
            println!("start clear_floor_lighting_cache");
            lighting.floor_light_cache.iter().for_each(|cache| {
                *cache.write().unwrap() = None;
            });
            println!("done clear_floor_lighting_cache");
        });
    }
}
