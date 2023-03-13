use crate::mth::Vector2;

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) struct Colour {
    pub(crate) r: f64,
    pub(crate) g: f64,
    pub(crate) b: f64
}

impl Colour {
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

    pub(crate) fn sdl(&self) -> sdl2::pixels::Color {
        let r = (self.r.min(1.0).max(0.0) * 255.0) as u8;
        let g = (self.g.min(1.0).max(0.0) * 255.0) as u8;
        let b = (self.b.min(1.0).max(0.0) * 255.0) as u8;
        sdl2::pixels::Color::RGB(r, g, b)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) struct Material {
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
            specular: 0.9,
            shininess: 200.0,
        }
    }

    /// Returns the colour of a certain column on the wall.
    pub(crate) fn lighting(&self, light: &ColumnLight, hit_point: &Vector2, wall_normal: &Vector2, to_eye: &Vector2) -> Colour {
        let base_colour = self.colour.multiply(light.intensity);
        let ambient_colour = base_colour.scale(self.ambient);

        let dir_to_light = light.pos.subtract(&hit_point).normalize();
        let light_a_normal = dir_to_light.dot(&wall_normal);

        let mut diffuse_colour = Colour::black();
        let mut specular_colour = Colour::black();
        if light_a_normal >= 0.0 {
            diffuse_colour = base_colour.scale(self.diffuse * light_a_normal);

            let reflection_direction = dir_to_light.negate().reflect(wall_normal);
            let reflect_a_eye = reflection_direction.dot(&to_eye);

            if reflect_a_eye >= 0.0 {
                let factor = reflect_a_eye.powf(self.shininess);
                specular_colour = light.intensity.scale(self.specular * factor);
            }
        }

        ambient_colour.add(diffuse_colour).add(specular_colour)
    }
}

#[derive(Debug)]
pub(crate) struct ColumnLight {
    pub(crate) intensity: Colour,
    pub(crate) pos: Vector2
}