use crate::light_cache::LightingRegion;
use crate::lighting::LightSource;
use crate::map_builder::MapRegion;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::ray::{Portal, SolidWall, trace_clear_path_between};

pub(crate) struct DynamicWall<'map, 'walls> {
    line: LineSegment2,
    material: Material,
    region: &'map MapRegion<'map>,
    portal: Option<Portal<'walls>>
}


pub(crate) struct DynamicLight<'map> {
    region: &'map MapRegion<'map>,
    intensity: Colour,
    pos: Vector2
}


pub(crate) struct SquareEntity<'map> {
    pub(crate) id: usize,
    pub(crate) pos: Vector2,
    pub(crate) region: &'map MapRegion<'map>,
    pub(crate) radius: f64,
    pub(crate) material: Material,

}

impl<'map, 'walls> SquareEntity<'map> {
    pub(crate) fn get_bounding_box(&self) -> Vec<Box<DynamicWall>> {
        LineSegment2::new_square(self.pos.x - self.radius, self.pos.y - self.radius, self.pos.x + self.radius, self.pos.y + self.radius)
            .into_iter().map(|line| Box::new(DynamicWall {
            line,
            material: self.material,
            region: self.region,
            portal: None,
        })).collect()
    }
    pub(crate) fn update_bounding_box(&mut self, region: &mut LightingRegion<'map, 'walls>){
        let bounding_box = self.get_bounding_box();

        let light = vec![Box::new(DynamicLight {
            region: self.region,
            intensity: Colour::black(),
            pos: self.pos
        })];

        // region.update_entity(self.id, bounding_box, light);
    }
}



impl<'map, 'walls> SolidWall<'walls> for DynamicWall<'map, 'walls> {
    fn portal(&self) -> Option<Portal<'walls>> {
        self.portal
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn line(&self) -> LineSegment2 {
        self.line
    }

    fn normal(&self) -> Vector2 {
        self.line.normal()
    }

    fn region(&self) -> &MapRegion<'map> {
        self.region
    }
}

// impl<'map> LightSource for DynamicLight<'map> {
//     fn intensity(&self) -> Colour {
//         self.intensity
//     }
//
//     fn apparent_pos(&self) -> &Vector2 {
//         &self.pos
//     }
//
//     // maybe Player needs to be the LightSource and then this method can check if flashlight on and facing the right way
//     // but then the light cache would need an immutable reference to it which seems bad
//     fn blocked_by_shadow(&self, hit_pos: &Vector2) -> bool {
//         trace_clear_path_between(self.pos, *hit_pos, self.region).is_none()
//     }
//
//     fn map_region(&self) -> &MapRegion {
//         self.region
//     }
// }
