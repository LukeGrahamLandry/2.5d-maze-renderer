use std::ops::Index;
use maze;
use maze::Grid;
use crate::map_builder::MapBuilder;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::new_world::World;
use crate::player::Player;

const MAZE_SIZE: i32 = 10;
const CELL_SIZE: i32 = 50;

pub(crate) fn random_maze_world<'map, 'walls>() -> World<'map, 'walls> {
    let mut builder = MapBuilder::new();
    create_maze_region(&mut builder, MAZE_SIZE, CELL_SIZE);
    let map = builder.build();
    World::new(map, 0, Vector2::of((CELL_SIZE / 2) as f64, (CELL_SIZE / 2) as f64))
}

fn create_maze_region(builder: &mut MapBuilder, maze_size: i32, cell_size: i32){
    let mut floor_material = Material::new(0.5, 0.25, 0.1);
    floor_material.ambient = 0.05;
    let region = builder.new_region(floor_material);

    let mut grid = maze::Grid::new(maze_size, maze_size);
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

pub(crate) fn example_preset<'map, 'walls>() -> World<'map, 'walls> {
    let mut builder = MapBuilder::new();

    let r0 = builder.new_square_region(100.0, 200.0, 300.0, 400.0, Material::default(Colour::rgb(0, 50, 50)));
    let r1 = builder.new_square_region(500.0, 200.0, 700.0, 400.0, Material::default(Colour::rgb(0, 50, 0)));
    let r2 = builder.new_square_region(50.0, 50.0, 150.0, 150.0, Material::default(Colour::rgb(150, 0, 50)));

    let line = LineSegment2::of(Vector2::of(200.0, 300.0), Vector2::of(200.0, 325.0));
    let w1 = builder.new_wall(r0, line, line.normal(), Material::new(0.2, 0.3, 0.8));
    builder.unidirectional_portal(r0, w1, r2, 1);

    let line = LineSegment2::of(Vector2::of(175.0, 300.0), Vector2::of(175.0, 325.0));
    let w2 = builder.new_wall(r0, line, line.normal(), Material::new(0.2, 0.3, 0.8));
    builder.unidirectional_portal(r0, w2, r2, 0);

    builder.bidirectional_portal(r0, 0, r1, 1);
    builder.bidirectional_portal(r1, 2, r2, 3);

    let map = builder.build();
    World::new(map, 0, Vector2::of(150.0, 250.0))
}
