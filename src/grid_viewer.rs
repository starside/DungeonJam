use crate::grid2d::{Grid2D, RayGridCell, GridCellType};
use macroquad::prelude::*;

pub fn draw_grid2d(world: &Grid2D<RayGridCell>, screen_size: (f32, f32)) {
    let (ww, wh) = world.get_size();
    let cell_dim = world.get_cell_screen_size(screen_size);
    let cell_border: f32 = 1.0;

    for y in 0..wh {
        for x in 0..ww {
            let cells = world.get_cells();
            let cell = &cells[y * ww + x];
            let cell_pos = Vec2::from((
                x as f32 * cell_dim.x,
                y as f32 * cell_dim.y
            ));

            draw_grid2d_cell(cell_pos, cell.cell_type, cell_border, world, screen_size);
        }
    }
}

pub fn draw_grid2d_cell(pos: Vec2, cell_type: GridCellType, cell_border: f32, world: &Grid2D<RayGridCell>, screen_size: (f32, f32)) {
    let cell_dim = world.get_cell_screen_size(screen_size);

    let color = match cell_type {
        GridCellType::Empty => {
            GRAY
        }
        GridCellType::Wall => {
            GREEN
        }
    };

    draw_rectangle(pos.x + cell_border, pos.y + cell_border,
                   cell_dim.x - cell_border, cell_dim.y - cell_border,
                   color);
}