use std::collections::HashMap;
use maze;
use maze::Grid;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::world::{FloorLightCache, LightKind, LightSource, Portal, Region, Wall, World};

const MAZE_SIZE: i32 = 10;
const CELL_SIZE: i32 = 50;

pub(crate) fn random_maze_world() -> World  {
    let mut builder = MapBuilder::new();
    create_maze_region(&mut builder, MAZE_SIZE, CELL_SIZE);
    World::new(builder.build(), 0, Vector2::of(CELL_SIZE as f64 * 1.5, CELL_SIZE as f64 * 1.5))
}

fn create_maze_region(builder: &mut MapBuilder, maze_size: i32, cell_size: i32){
    let mut floor_material = Material::default(Colour::rgb(100, 100, 150));
    floor_material.ambient = 0.05;

    let mut grid = maze::Grid::new(maze_size, maze_size);
    let region = builder.new_region(floor_material, FloorLightCache::new(Vector2::zero(), Vector2::of((CELL_SIZE * cell_size) as f64, (CELL_SIZE * cell_size) as f64)));
    let walls = gen_maze_lines(&mut grid, cell_size);
    let count = walls.len();
    for wall in walls {
        builder.new_wall(region, wall, wall.normal(), Material::new(0.2, 0.8, 0.2));
    }
    println!("Created world for {}x{} maze with {} walls", grid.cols, grid.rows, count);

    let lights = [
        Vector2::of((cell_size / 2) as f64, ((grid.rows * cell_size) - (cell_size / 2)) as f64),
        Vector2::of(((grid.cols * cell_size) - (cell_size / 2)) as f64, (cell_size / 2) as f64),
        Vector2::of(((grid.cols * cell_size) - (cell_size / 2)) as f64, ((grid.rows * cell_size) - (cell_size / 2)) as f64),
        Vector2::of((cell_size / 2) as f64, (cell_size / 2) as f64),
    ];
    for light_pos in lights {
        builder.new_light(region, Colour::white(), light_pos);
    }
}

fn gen_maze_lines(mut grid: &mut Grid, cell_size: i32) -> Vec<LineSegment2>{
    maze::gen::binary_tree::on(&mut grid);
    println!("{}", grid.to_string());
    let mut vertical_walls: Vec<LineSegment2> = vec![];
    let mut horizontal_walls: Vec<LineSegment2> = vec![];

    // The north wall
    horizontal_walls.push(LineSegment2::of(Vector2::of(0.0, 0.0), Vector2::of((cell_size * grid.cols) as f64, 0.0)));
    // The west wall
    vertical_walls.push(LineSegment2::of(Vector2::of(0.0, 0.0), Vector2::of(0.0, (cell_size * grid.rows) as f64)));

    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let pos = maze::Pos::of(row, col);
            let x1 = (col * cell_size) as f64;
            let y1 = (row * cell_size) as f64;
            let x2 = ((col + 1) * cell_size) as f64;
            let y2 = ((row + 1) * cell_size) as f64;
            let cell = grid.get_cell(pos);


            let wall_start = Vector2::of(x2, y1);
            let wall_end = Vector2::of(x2, y2);
            if !cell.links.contains(&grid.east(pos)) {
                vertical_walls.push(LineSegment2::of(wall_start, wall_end));
            }

            let wall_start = Vector2::of(x1, y2);
            let wall_end = Vector2::of(x2, y2);
            if !cell.links.contains(&grid.south(pos)) {
                horizontal_walls.push(LineSegment2::of(wall_start, wall_end));
            }
        }
    }

    condense_walls(horizontal_walls, vertical_walls)
}


/// Combines any continuous runs of walls into one for faster ray tracing.
fn condense_walls(horizontal: Vec<LineSegment2>, mut vertical: Vec<LineSegment2>) -> Vec<LineSegment2> {
    let mut smart_walls: Vec<LineSegment2> = vec![];

    let mut put_wall = |x1: f64, x2: f64, y1: f64, y2: f64| {
        let wall = LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x2, y2));
        smart_walls.push(wall);
    };

    {
        let mut y = 0.0;
        let mut x_start = 0.0;
        let mut x_end = 0.0;
        for wall in horizontal {
            if wall.a.y != y {
                if x_start != x_end {
                    put_wall(x_start, x_end, y, y);
                }
                x_end = 0.0;
                x_start = 0.0;
                y = wall.a.y;
            }

            if wall.a.x == x_end {
                x_end = wall.b.x;
            } else {
                if x_start != x_end {
                    put_wall(x_start, x_end, y, y);
                    x_start = wall.a.x;
                } else {
                    x_start = wall.a.x;
                }
                x_end = wall.b.x;
            }
        }
        if x_start != x_end {
            put_wall(x_start, x_end, y, y);
        }
    }
    {
        let mut x = 0.0;
        let mut y_start = 0.0;
        let mut y_end = 0.0;
        vertical.sort_by(|w1, w2| w1.a.x.total_cmp(&w2.a.x));
        for wall in vertical {
            if wall.a.x != x {
                if y_start != y_end {
                    put_wall(x, x, y_start, y_end);
                }
                y_end = 0.0;
                y_start = 0.0;
                x = wall.a.x;
            }

            if wall.a.y == y_end {
                y_end = wall.b.y;
            } else {
                if y_start != y_end {
                    put_wall(x, x, y_start, y_end);
                    y_start = wall.a.y;
                } else {
                    y_start = wall.a.y;
                }
                y_end = wall.b.y;
            }
        }
        if y_start != y_end {
            put_wall(x, x, y_start, y_end);
        }
    }

    smart_walls
}

pub(crate) fn example_preset() -> World  {
    let mut builder = MapBuilder::new();

    let r0 = builder.new_square_region(100.0, 200.0, 300.0, 400.0, Material::default(Colour::rgb(0, 50, 50)));
    let r1 = builder.new_square_region(500.0, 200.0, 700.0, 400.0, Material::default(Colour::rgb(0, 50, 0)));
    let r2 = builder.new_square_region(50.0, 50.0, 150.0, 150.0, Material::default(Colour::rgb(150, 0, 50)));

    let line = LineSegment2::of(Vector2::of(200.0, 300.0), Vector2::of(200.0, 325.0));
    let w1 = builder.new_wall(r0, line, line.normal(), Material::new(0.2, 0.3, 0.8));
    builder.unidirectional_portal(r0, w1, r2, 1);

    let line = LineSegment2::of(Vector2::of(175.0, 300.0), Vector2::of(175.0, 325.0));
    let w2 = builder.new_wall(r0, line, line.normal().negate(), Material::new(0.2, 0.3, 0.8));
    builder.unidirectional_portal(r0, w2, r2, 0);

    builder.bidirectional_portal(r0, 0, r1, 1);
    builder.bidirectional_portal(r1, 2, r2, 3);

    World::new(builder.build(), 0, Vector2::of(150.0, 250.0))
}

struct MapBuilder {
    regions: Vec<Region>
}

impl MapBuilder {
    pub(crate) fn new() -> MapBuilder {
        MapBuilder {
            regions: vec![]
        }
    }

    pub(crate) fn new_region(&mut self, floor_material: Material, lighting: FloorLightCache) -> usize {
        let i = self.regions.len();
        self.regions.push(Region {
            id: i,
            walls: HashMap::new(),
            lights: HashMap::new(),
            floor_material,
            lighting
        });

        i
    }

    pub(crate) fn new_square_region(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, material: Material) -> usize {
        let region = self.new_region(material, FloorLightCache::new(Vector2::of(x1, y1), Vector2::of(x2, y2)));

        let walls = LineSegment2::new_square(x1, y1, x2, y2);
        let light_pos = walls[0].a.add(&walls[0].direction().scale(-0.25).add(&walls[2].direction().scale(-0.25)));
        for i in 0..4 {
            self.new_wall(region, walls[i], if i % 2 == 0 { walls[i].normal() } else { walls[i].normal().negate() }, Material::new(0.2, 0.2, 0.9));
        }
        self.new_light(region, Colour::white(), light_pos);

        region
    }

    pub(crate) fn new_wall(&mut self, region_index: usize, line: LineSegment2, normal: Vector2, material: Material) -> usize {
        let mut walls = &mut self.regions[region_index].walls;
        let i = walls.len();
        walls.insert(i, Wall {
            id: i,
            region: region_index,
            line,
            normal,
            material,
            portal: None,
        });
        i
    }

    pub(crate) fn unidirectional_portal(&mut self, from_region: usize, from_wall: usize, to_region: usize, to_wall: usize){
        let regions = &mut self.regions;

        assert!(from_region < regions.len());
        assert!(to_region < regions.len());

        if from_region == to_region {
            assert_ne!(from_wall, to_wall);

            let region = &mut regions[from_region];
            let portal = Portal::new(region.get_wall(from_wall), region.get_wall(to_wall));
            region.walls.get_mut(&from_wall).expect("Invalid wall index.").portal = portal;
        } else {
            let portal = {
                let from_region = &regions[from_region];
                let to_region = &regions[to_region];
                Portal::new(from_region.get_wall(from_wall), to_region.get_wall(to_wall))
            };

            regions[from_region].walls.get_mut(&from_wall).expect("Invalid wall index.").portal = portal;
        }
    }

    pub(crate) fn bidirectional_portal(&mut self, from_region: usize, from_wall: usize, to_region: usize, to_wall: usize){
        self.unidirectional_portal(from_region, from_wall, to_region, to_wall);
        self.unidirectional_portal(to_region, to_wall, from_region, from_wall);
    }

    pub(crate) fn new_light(&mut self, region_index: usize, intensity: Colour, pos: Vector2){
        let mut lights = &mut self.regions[region_index].lights;

        let i = lights.len();
        lights.insert(i, LightSource {
            id: i,
            region: region_index,
            intensity,
            pos,
            kind: LightKind::DIRECT(),
        });
    }

    pub(crate) fn build(self) -> Vec<Region> {
        self.regions
    }

    pub(crate) fn from_world(world: World) -> MapBuilder {
        MapBuilder {
            regions: world.regions,
        }
    }
}