use std::cell::RefCell;
use std::rc::Rc;
use sdl2::pixels::Color;
use maze;
use crate::material::{Colour, ColumnLight};
use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;
use crate::world::{Region, World, Wall};


// TODO: combine continuous cell sides into one wall for faster collision checking.
pub(crate) fn maze_to_regions(grid: &maze::Grid, cell_size: i32) -> Vec<Rc<RefCell<Region>>> {
    let mut walls: Vec<LineSegment2> = vec![];

    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let pos = maze::Pos::of(row, col);
            let x1 = (col * cell_size) as f64;
            let y1 = (row * cell_size) as f64;
            let x2 = ((col + 1) * cell_size) as f64;
            let y2 = ((row + 1) * cell_size) as f64;


            if !grid.has(grid.north(pos)) {
                walls.push(LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x2, y1)))
            }

            if !grid.has(grid.west(pos)) {
                walls.push(LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x1, y2)))
            }

            let cell = grid.get_cell(pos);
            if !cell.links.contains(&grid.east(pos)) {
                walls.push(LineSegment2::of(Vector2::of(x2, y1), Vector2::of(x2, y2)));
            }

            if !cell.links.contains(&grid.south(pos)) {
                walls.push(LineSegment2::of(Vector2::of(x1, y2), Vector2::of(x2, y2)));
            }
        }
    }

    let region = Region::new();
    {
        let mut m_region = region.borrow_mut();
        for wall in walls {
            m_region.walls.push(Wall::new(wall, wall.normal(), &region))
        }
        let light_pos = Vector2::of((cell_size / 2) as f64, (cell_size / 2) as f64);
        m_region.lights.push(ColumnLight {
            pos: light_pos,
            intensity: Colour::white()
        });
        m_region.floor_color = Color::RGB(100, 100, 150);
    }

    vec![region]
}

pub(crate) fn shift_the_world(world: &mut World){
    let cell_size = 50;

    let mut grid = maze::Grid::new(10, 10);
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

    let mut grid = maze::Grid::new(10, 10);
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
