use crate::material::{Material};
use crate::mth::{LineSegment2, Vector2};
use crate::world::{Portal, Wall};

pub(crate) struct SquareEntity {
    pub(crate) id: usize,
    pub(crate) bb_ids: [usize; 4],
    pub(crate) pos: Vector2,
    pub(crate) region: usize,
    pub(crate) radius: f64,
    pub(crate) material: Material,

}

impl SquareEntity {
    pub(crate) fn get_bounding_box(&self) -> Vec<Wall> {
        let lines = LineSegment2::new_square(self.pos.x - self.radius, self.pos.y - self.radius, self.pos.x + self.radius, self.pos.y + self.radius);

        let mut walls = Vec::with_capacity(4);
        for (i, line) in lines.into_iter().enumerate() {
            walls.push(Wall {
                id: self.bb_ids[i],
                line,
                normal: line.normal(),
                material: self.material,
                region: self.region,
                portal: Portal::NONE,
            })
        }

        walls
    }
}
