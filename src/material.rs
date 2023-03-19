use crate::mth::{EPSILON, Vector2};
use crate::world::LightSource;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Colour {
    pub(crate) r: f64,
    pub(crate) g: f64,
    pub(crate) b: f64
}

impl Colour {
    pub(crate) fn rgb(r: u8, g: u8, b: u8) -> Colour {
        Colour::new(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
    }

    pub(crate) fn black() -> Colour {
        Colour::new(0.0, 0.0, 0.0)
    }

    pub(crate) fn white() -> Colour {
        Colour::new(1.0, 1.0, 1.0)
    }

    pub(crate) fn new(r: f64, g: f64, b: f64) -> Colour {
        Colour { r, g, b }
    }

    pub(crate) fn add(&self, other: Colour) -> Colour {
        Colour::new(self.r + other.r, self.g + other.g, self.b + other.b)
    }

    pub(crate) fn scale(&self, s: f64) -> Colour {
        Colour::new(self.r * s, self.g * s, self.b * s)
    }

    pub(crate) fn multiply(&self, other: Colour) -> Colour {
        Colour::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }

    pub(crate) fn lerp(&self, other: &Colour, t: f64) -> Colour {
        Colour::new(
            Self::lerp_f(self.r, other.r, t),
            Self::lerp_f(self.g, other.g, t),
            Self::lerp_f(self.b, other.b, t)
        )
    }

    fn lerp_f(a: f64, b: f64, t: f64) -> f64 {
        let dif = b - a;
        a + (dif * t)
    }

    pub(crate) fn to_u8(&self) -> (u8, u8, u8) {
        if self.r.is_nan() || self.g.is_nan() || self.b.is_nan() {
            return (255, 255, 255);
        }

        let r = (self.r.min(1.0).max(0.0) * 255.0) as u8;
        let g = (self.g.min(1.0).max(0.0) * 255.0) as u8;
        let b = (self.b.min(1.0).max(0.0) * 255.0) as u8;
        (r, g, b)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Material {
    pub(crate) colour: Colour,
    pub(crate) ambient: f64,
    pub(crate) diffuse: f64,
    pub(crate) specular: f64,
    pub(crate) shininess: f64,
}

impl Material {
    pub(crate) fn new(r: f64, g: f64, b: f64) -> Material {
        Material {
            colour: Colour::new(r, g, b),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.2,
            shininess: 10.0,
        }
    }

    pub(crate) fn default(colour: Colour) -> Material {
        Material {
            colour,
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.2,
            shininess: 10.0,
        }
    }

    /// Does not do ray tracing. Just trusts that the values passed in are correct.
    /// https://en.wikipedia.org/wiki/Phong_reflection_model
    pub(crate) fn calculate_wall_lighting(&self, light: &LightSource, hit_point: &Vector2, mut wall_normal: Vector2, to_eye: &Vector2, in_shadow: bool) -> Colour {
        let base_colour = self.colour.multiply(light.intensity);
        let ambient_colour = base_colour.scale(self.ambient);

        if in_shadow {
            return ambient_colour;
        }

        let dir_to_light = light.pos.subtract(&hit_point).normalize();
        let light_on_front = dir_to_light.dot(&wall_normal) >= EPSILON;
        if !light_on_front {
            wall_normal = wall_normal.negate();
        }

        let cos_light_to_normal = dir_to_light.dot(&wall_normal);
        let mut diffuse_colour = Colour::black();
        let mut specular_colour = Colour::black();
        if cos_light_to_normal >= 0.0 {
            diffuse_colour = base_colour.scale(self.diffuse * cos_light_to_normal);

            let reflection_direction = dir_to_light.negate().reflect(&wall_normal);
            let cos_reflect_to_eye = reflection_direction.dot(&to_eye);

            if cos_reflect_to_eye >= 0.0 {
                let factor = cos_reflect_to_eye.powf(self.shininess);
                specular_colour = light.intensity.scale(self.specular * factor);
            }
        }

        ambient_colour.add(diffuse_colour).add(specular_colour)
    }

    // diffuse_factor = sum of cos_light_to_normal all the way up the light column.
    // The normal is just up since its the floor. We need the vector from the hit_point to the point on pillar.
    // cos = opposite / adjacent
    // opposite = height of the point up the pillar
    // adjacent = distance from the pillar to the hit_point
    // want to do that for infinitely many points up the pillar
    // f(x) = integral (x / distance) dx
    // f(x) = (1 / (2 * distance)) * x^2
    // f(0) = 0
    // we care about the interval x=(0, height) so answer is just (1 / (2 * distance)) * height^2
    const LIGHT_PILLAR_HEIGHT_SQUARED: f64 = 25.0;
    const MAX_FLOOR_LIGHT_DISTANCE: f64 = 75.0 * Material::LIGHT_PILLAR_HEIGHT_SQUARED;
    const MAX_FLOOR_LIGHT_DISTANCE_SQUARED: f64 = Material::MAX_FLOOR_LIGHT_DISTANCE * Material::MAX_FLOOR_LIGHT_DISTANCE;
    /// Does not do ray tracing. Just trusts that the values passed in are correct.
    pub(crate) fn calculate_floor_lighting(&self, light: &LightSource, hit_point: Vector2, in_shadow: bool) -> Colour {
        let base_colour = self.colour.multiply(light.intensity);
        let ambient_colour = base_colour.scale(self.ambient);
        if in_shadow {
            return ambient_colour;
        }

        let dist_to_light_sq = light.pos.subtract(&hit_point).length_sq();
        let diffuse_colour = if dist_to_light_sq < Material::MAX_FLOOR_LIGHT_DISTANCE_SQUARED {
            let diffuse_factor = (1.0 / dist_to_light_sq.sqrt()) * Material::LIGHT_PILLAR_HEIGHT_SQUARED;
            base_colour.scale(self.diffuse * diffuse_factor)
        } else {
            Colour::black()
        };

        ambient_colour.add(diffuse_colour)
    }
}
