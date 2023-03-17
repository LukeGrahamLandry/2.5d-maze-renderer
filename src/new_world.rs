use crate::{map_builder::Map, light_cache::LightingRegion};
use crate::light_cache::{LightCache};
use crate::map_builder::MapWall;
use crate::mth::Vector2;
use crate::world_data::Player;

pub(crate) struct World<'a> {
    pub(crate) map: Map<'a>,
    pub(crate) lighting: LightCache<'a>,
    // pub(crate) player: Player<'a>
}

impl<'a> World<'a> {
    pub(crate) fn new(map: Map<'a>) -> World<'a> {
        let mut world = World {
            map,
            lighting: LightCache::new(&map)
        };

        world
    }
}

impl<'a> MapWall<'a> {
    pub(crate) fn is_portal(&self) -> bool {
        self.next_wall.is_some()
    }

    pub(crate) fn scale_factor(from: &MapWall, to: &MapWall) -> f64 {
        to.line.length() / from.line.length()
    }

    // transform to same position but relative to the new wall, accounting for walls of different sizes.
    pub(crate) fn translate(pos: Vector2, from: &MapWall, to: &MapWall) -> Vector2 {
        let last_offset = pos.subtract(&from.line.a);
        let fraction = last_offset.length() / from.line.direction().length();
        let new_offset = to.line.direction().negate().scale(fraction);

        to.line.a.add(&new_offset)
    }

    pub(crate) fn rotate(direction: Vector2, from: &MapWall, to: &MapWall) -> Vector2 {
        let rot_offset = from.normal.angle_between(&to.normal.negate());
        let dir = direction.rotate(rot_offset);
        if dir.dot(&to.normal) > 0.0 {
            dir
        } else {
            dir.negate()
        }
    }
}
