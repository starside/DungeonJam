use crate::grid2d::{Grid2D, RayGridCell, GridCellType};
use macroquad::prelude::*;

pub fn draw_grid2d(world: &Grid2D<RayGridCell>, screen_size: (f32, f32)) {
    let (ww, wh) = world.get_size();

    let cell_width: f32 = screen_size.0 / (ww as f32);
    let cell_height: f32 = screen_size.1 / (wh as f32);

    let cell_border: f32 = 1.0;

    for y in 0..wh {
        for x in 0..ww {
            let cells = world.get_cells();
            let cell = &cells[y * ww + x];
            let (cell_x, cell_y) = (
                x as f32 * cell_width,
                y as f32 * cell_height
            );
            match cell.cell_type {
                GridCellType::Empty => {
                    draw_rectangle(cell_x + cell_border, cell_y + cell_border,
                                   cell_width - cell_border, cell_height - cell_border,
                                   GRAY);
                }
                GridCellType::Wall => {
                    draw_rectangle(cell_x + cell_border, cell_y + cell_border,
                                   cell_width - cell_border, cell_height - cell_border,
                                   GREEN);
                }
            }
        }
    }
}