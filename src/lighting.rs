use crate::material::{Colour, Material};
use crate::mth::{Vector2};
use crate::world::{LightSource, Region};


impl Region {
    /// Calculates the colour of a column of wall based on all lights in the region.
    pub(crate) fn vertical_surface_colour(&self, hit_point: &Vector2, wall: usize, ray_direction: Vector2) -> Colour {
        let mut colour = Colour::black();
        let to_eye = ray_direction.negate().normalize();
        let wall = self.get_wall(wall);
        for light in self.lights() {
            colour = colour.add(self.wall_lighting(wall.material(), light, hit_point, wall.normal(), &to_eye));
        }

        colour
    }

    /// Calculates the colour of a point on the floor based on all lights in the region.
    pub(crate) fn horizontal_surface_colour(&self, hit_pos: Vector2) -> Colour {
        let mut colour = Colour::black();
        for light in self.lights() {
            colour = colour.add(self.floor_lighting(&self.floor_material, light, hit_pos));
        }
        colour
    }

    fn wall_lighting(&self, material: &Material, light: &LightSource, hit_point: &Vector2, wall_normal: Vector2, to_eye: &Vector2) -> Colour {
        let dir_to_light = light.pos.subtract(&hit_point).normalize();
        let light_on_front = dir_to_light.is_pointing_opposite(&wall_normal);
        let eye_on_front = to_eye.is_pointing_opposite(&wall_normal);
        let in_shadow = light_on_front != eye_on_front || light.blocked_by_shadow(self, hit_point);

        material.calculate_wall_lighting(light, hit_point, wall_normal, to_eye, in_shadow)
    }

    fn floor_lighting(&self, material: &Material, light: &LightSource, hit_point: Vector2) -> Colour {
        material.calculate_floor_lighting(&light, hit_point, light.blocked_by_shadow(self, &hit_point))
    }
}
