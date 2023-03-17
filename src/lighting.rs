use crate::light_cache::LightSource;
use crate::material::{Colour, Material};
use crate::mth::{EPSILON, Vector2};

/// Returns the colour of a certain column on the wall.
pub(crate) fn wall_lighting(material: &Material, light: &dyn LightSource, hit_point: &Vector2, wall_normal: Vector2, to_eye: &Vector2) -> Colour {
    let dir_to_light = light.apparent_pos().subtract(&hit_point).normalize();
    let light_on_front = dir_to_light.is_pointing_opposite(&wall_normal);
    let eye_on_front = to_eye.is_pointing_opposite(&wall_normal);
    let in_shadow = light_on_front != eye_on_front || light.blocked_by_shadow(hit_point);

    material.calculate_wall_lighting(light, hit_point, wall_normal, to_eye, in_shadow)
}

pub(crate) fn portal_lighting(material: &Material, light: &dyn LightSource, hit_point: Vector2) -> Colour {
    material.calculate_floor_lighting(light, hit_point, light.blocked_by_shadow(&hit_point))
}
