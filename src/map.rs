use std::ops::{Index, IndexMut};
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::world_data::{World};

pub(crate) struct Map<'a> {
    regions: Vec<MapRegion<'a>>
}

pub(crate) struct MapRegion<'a> {
    walls: Vec<MapWall<'a>>,
    lights: Vec<MapLight<'a>>,
    pub(crate) floor_material: Material
}

pub(crate) struct MapWall<'a> {
    pub(crate) region: &'a MapRegion<'a>,
    pub(crate) line: LineSegment2,
    pub(crate) normal: Vector2,
    pub(crate) next_wall: Option<&'a MapWall<'a>>,
    pub(crate) material: Material,
}

pub(crate) struct MapLight<'a> {
    pub(crate) region: &'a MapRegion<'a>,
    pub(crate) intensity: Colour,
    pub(crate) pos: Vector2
}

// If you ever resize any of the vectors, all the references are completely fucked.
// I could switch to Box<[T]> but you can't call into_boxed_slice into one without getting rid of the extra capacity.
// And I don't trust it not to reallocate even if it doesn't have to. It doesn't want uninitialized memory.
// There's some combination of MaybeUninit and raw pointers that would make this work.
// But just having a getter that only gives immutable references and being careful to be safe in this module works too.

impl<'a> Map<'a> {
    pub(crate) fn regions(&self) -> &Vec<MapRegion> {
        &self.regions
    }
}

impl<'a> MapRegion<'a> {
    pub(crate) fn walls(&self) -> &Vec<MapWall> {
        &self.walls
    }

    pub(crate) fn lights(&self) -> &Vec<MapLight> {
        &self.lights
    }
}

pub(crate) struct MapBuilder {
    regions: Vec<RegionBuilder>
}

pub(crate) struct RegionBuilder {
    myself_index: usize,
    walls: Vec<WallBuilder>,
    lights: Vec<LightBuilder>,
    floor_material: Material
}

pub(crate) struct WallBuilder {
    myself_index: usize,
    region_index: usize,
    line: LineSegment2,
    normal: Vector2,
    next_region_index: Option<usize>,
    next_wall_index: Option<usize>,
    material: Material,
}

pub(crate) struct LightBuilder {
    region_index: usize,
    myself_index: usize,
    intensity: Colour,
    pos: Vector2
}

impl MapBuilder {
    pub(crate) fn build<'a>(&self) -> Map<'a> {
        let mut map = Map {
            regions: Vec::new()
        };

        unsafe {
            self.build_into(&mut map);
        }

        map
    }

    unsafe fn build_into(&self, map: *mut Map) {
        // Its very important that these vectors are sized correctly so they never reallocate and references to their elements remain valid.
        (*map).regions = Vec::with_capacity(self.regions.len());
        for region_builder in self.regions.iter() {
            (*map).regions.push(MapRegion {
                walls: Vec::with_capacity(region_builder.walls.len()),
                lights: Vec::with_capacity(region_builder.lights.len()),
                floor_material: region_builder.floor_material
            });
        }

        for (r, region_builder) in self.regions.iter().enumerate() {
            for wall_builder in region_builder.walls.iter() {
                (*map).regions[r].walls.push(MapWall {
                    region: (*map).regions.index(r),
                    line: wall_builder.line,
                    normal: wall_builder.normal,
                    next_wall: None,
                    material: wall_builder.material,
                })
            }

            for light_builder in &region_builder.lights {
                (*map).regions[r].lights.push(MapLight {
                    region: (*map).regions.index(r),
                    intensity: light_builder.intensity,
                    pos: light_builder.pos
                })
            }
        }

        for (r, region_builder) in self.regions.iter().enumerate() {
            for (w, wall_builder) in region_builder.walls.iter().enumerate() {
                match (wall_builder.next_wall_index, wall_builder.next_region_index) {
                    (Some(next_wall), Some(next_region)) => {
                        (*map).regions[r].walls.index_mut(w).next_wall = Some((*map).regions[next_region].walls.index(next_wall));
                    }
                    _ => {}
                }
            }
        }
    }
    
    pub(crate) fn from_map(builder: &Map) -> MapBuilder {
        let mut map = MapBuilder {
            regions: vec![],
        };

        map.regions = Vec::with_capacity(builder.regions.len());
        for (r, region_builder) in builder.regions.iter().enumerate() {
            map.regions.push(RegionBuilder {
                myself_index: r,
                walls: Vec::with_capacity(region_builder.walls().len()),
                lights: Vec::with_capacity(region_builder.lights().len()),
                floor_material: region_builder.floor_material
            });
        }

        for (r, region_builder) in builder.regions.iter().enumerate() {
            for (w, wall_builder) in region_builder.walls().iter().enumerate() {
                map.regions[r].walls.push(WallBuilder {
                    myself_index: w,
                    region_index: r,
                    line: wall_builder.line,
                    normal: wall_builder.normal,
                    next_region_index: match wall_builder.next_wall {
                        None => { None }
                        Some(next_wall) => {
                            Some(builder.regions().iter().position(|check| (check as *const MapRegion) == (next_wall.region as *const MapRegion)).unwrap())
                        }
                    },
                    next_wall_index: match wall_builder.next_wall {
                        None => { None }
                        Some(next_wall) => {
                            Some(next_wall.region.walls().iter().position(|check| (check as *const MapWall) == (next_wall as *const MapWall)).unwrap())
                        }
                    },
                    material: wall_builder.material,
                })
            }

            for (l, light_builder) in region_builder.lights().iter().enumerate() {
                map.regions[r].lights.push(LightBuilder {
                    region_index: r,
                    myself_index: l,
                    intensity: light_builder.intensity,
                    pos: light_builder.pos
                })
            }
        }
        
        map
    }

    pub(crate) fn from_world(builder: &World) -> MapBuilder{
        let mut map = MapBuilder {
            regions: vec![],
        };

        // Its very important that these vectors are sized correctly so they never reallocate and references to their elements remain valid.
        map.regions = Vec::with_capacity(builder.regions.len());
        for (r, region_builder) in builder.regions.iter().enumerate() {
            let region_builder = region_builder.borrow();
            map.regions.push(RegionBuilder {
                myself_index: r,
                walls: Vec::with_capacity(region_builder.wall_count()),
                lights: Vec::with_capacity(region_builder.lights.len()),
                floor_material: region_builder.floor_material
            });
        }

        for (r, region_builder) in builder.regions.iter().enumerate() {
            let region_builder = region_builder.borrow();
            for (w, wall_builder) in region_builder.iter_walls().enumerate() {
                map.regions[r].walls.push(WallBuilder {
                    myself_index: w,
                    region_index: r,
                    line: wall_builder.line,
                    normal: wall_builder.normal,
                    next_region_index: match wall_builder.get_next_wall() {
                        None => { None }
                        Some(next_wall) => {
                            let target_r = builder.index_of_region(&next_wall.borrow().region.borrow()).expect("Failed region connection.");
                            Some(target_r)
                        }
                    },
                    next_wall_index: match wall_builder.get_next_wall() {
                        None => { None }
                        Some(next_wall) => {
                            let target_r = builder.index_of_region(&next_wall.borrow().region.borrow()).expect("Failed region connection.");
                            let target_w = builder.regions[target_r].borrow().index_of_wall(&next_wall.borrow()).expect("Failed wall connection.");
                            Some(target_w)
                        }
                    },
                    material: wall_builder.material,
                })
            }

            for (l, light_builder) in region_builder.lights.iter().enumerate() {
                map.regions[r].lights.push(LightBuilder {
                    region_index: r,
                    myself_index: l,
                    intensity: light_builder.borrow().intensity,
                    pos: light_builder.borrow().pos
                })
            }
        }
        
        map
    }
}

#[cfg(test)]
mod tests {
    use crate::world_gen::example_preset;
    use super::*;

    #[test]
    fn immutable_map_builder() {
        let world = example_preset();
        let builder_from_world = MapBuilder::from_world(&world);
        let map = builder_from_world.build();
        let builder_from_map = MapBuilder::from_map(&map);
        
        assert!(world.regions[0].borrow().get_wall(0).material == map.regions()[0].walls()[0].material)
    }
}
