use std::cell::RefCell;
use std::rc::Rc;
use maze;
use crate::material::{Colour, ColumnLight};
use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;
use crate::world::{Region, World, Wall};

const SIZE: i32 = 10;

pub(crate) fn maze_to_regions(grid: &maze::Grid, cell_size: i32) -> Vec<Rc<RefCell<Region>>> {
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

    let walls = condense_walls(horizontal_walls, vertical_walls);

    let region = Region::new();
    {
        let mut m_region = region.borrow_mut();
        let count = walls.len();
        for wall in walls {
            m_region.walls.push(Wall::new(wall, wall.normal(), &region))
        }
        println!("Created world for {}x{} maze with {} walls", grid.cols, grid.rows, count);

        let lights = [
            Vector2::of((cell_size / 2) as f64, ((grid.rows * cell_size) - (cell_size / 2)) as f64),
            Vector2::of(((grid.cols * cell_size) - (cell_size / 2)) as f64, (cell_size / 2) as f64),
            Vector2::of(((grid.cols * cell_size) - (cell_size / 2)) as f64, ((grid.rows * cell_size) - (cell_size / 2)) as f64),
            Vector2::of((cell_size / 2) as f64, (cell_size / 2) as f64),
        ];
        for light_pos in lights {
            m_region.lights.push(Rc::new(ColumnLight {
                pos: light_pos,
                intensity: Colour::white()
            }));
        }
        m_region.floor_material.colour = Colour::rgb(100, 100, 150);
    }
    Region::recalculate_lighting(region.clone());

    vec![region]
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

pub(crate) fn shift_the_world(world: &mut World){
    let cell_size = 50;

    let mut grid = maze::Grid::new(SIZE, SIZE);
    maze::gen::binary_tree::on(&mut grid);
    let regions = maze_to_regions(&grid, cell_size);

    let weak_player = Rc::downgrade(&world.player);
    regions[0].borrow_mut().things.insert(world.player.borrow().id, weak_player);
    world.player.borrow_mut().region = regions[0].clone();
    world.player.borrow_mut().clear_portal(0);
    world.player.borrow_mut().clear_portal(1);

    world.regions = regions;
}

pub(crate) fn random_maze_world() -> World {
    let cell_size = 50;

    let mut grid = maze::Grid::new(SIZE, SIZE);
    maze::gen::binary_tree::on(&mut grid);
    let regions = maze_to_regions(&grid, cell_size);

    let mut player = Player::new(&regions[0]);
    player.pos.x = (cell_size / 2) as f64;
    player.pos.y = (cell_size / 2) as f64;
    player.update_bounding_box();
    let id = player.id;

    let player = Rc::new(RefCell::new(player));
    let weak_player = Rc::downgrade(&player);
    regions[0].borrow_mut().things.insert(id, weak_player);

    World {
        regions,
        player
    }
}
