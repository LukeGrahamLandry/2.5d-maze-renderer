use std::fmt::Debug;
use std::ops::Index;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::shelf::ShelfPtr;
use crate::world_data::{Region, Wall, World};

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

impl<'a> Map<'a> {
    pub(crate) fn new(builder: &World) -> Map<'a> {
        let mut map = Map {
            regions: Vec::new()
        };

        unsafe {
            Map::build(builder, &mut map);
        }

        return map;
    }

    pub(crate) unsafe fn build(builder: &World, map: *mut Map){
        // Its very important that these vectors are sized correctly so they never reallocate and references to their elements remain valid.
        (*map).regions = Vec::with_capacity(builder.regions.len());
        for region_builder in builder.regions.iter() {
            let region_builder = region_builder.borrow();
            (*map).regions.push(MapRegion {
                walls: Vec::with_capacity(region_builder.wall_count()),
                lights: Vec::with_capacity(region_builder.lights.len()),
                floor_material: region_builder.floor_material
            });
        }

        for (r, region_builder) in builder.regions.iter().enumerate() {
            let region_builder = region_builder.borrow();
            for wall_builder in region_builder.iter_walls() {
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
                    intensity: light_builder.borrow().intensity,
                    pos: light_builder.borrow().pos
                })
            }
        }

        for (r, region_builder) in builder.regions.iter().enumerate() {
            let region_builder = region_builder.borrow();
            for (w, wall_builder) in region_builder.iter_walls().enumerate() {
                match wall_builder.get_next_wall() {
                    None => {}
                    Some(next_wall) => {
                        let target_r = builder.index_of_region(&next_wall.borrow().region.borrow()).expect("Failed region connection.");
                        let target_w = builder.regions[target_r].borrow().index_of_wall(&next_wall.borrow()).expect("Failed wall connection.");
                        let wall_ref = (*map).regions.index(target_r).walls.index(target_w);
                        (*map).regions[r].walls[w].next_wall = Some(wall_ref);
                    }
                }
            }
        }
    }
}

// If you ever resize any of the vectors, all the references are completely fucked.
// I could switch to Box<[T]> but you can't call into_boxed_slice into one without getting rid of the extra capacity.
// And I don't trust it not to reallocate even if it doesn't have to. It doesn't want uninitialized memory.
// There's some combination of MaybeUninit and raw pointers that would make this work.
// But just having a getter and being careful to be safe in this module works too.

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


#[cfg(test)]
mod tests {
    use crate::world_gen::example_preset;
    use super::*;

    #[test]
    fn bake_immutable_map() {
        let map = {
            let world = example_preset();
            Map::new(&world)
        };

        println!("{:?}", map.regions[0].walls[0].line);
    }
}
