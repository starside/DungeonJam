use macroquad::math::{DVec2, IVec2};
use crate::grid2d::{Grid2D, GridCellType, RayGridCell};

#[derive(PartialEq)]
pub enum HitSide {
    Vertical, // 0 in lodev
    Horizontal // 1
}

// start amd end are in grid coordinates, assuming each cell has size 1,
// so start (3.5, 14.7) would be inside cells (3, 14)
pub fn cast_ray(grid: &Grid2D<RayGridCell>, start: &DVec2, ray_dir: &DVec2, max_ray_distance: f64) ->
                                            (f64, GridCellType, HitSide, IVec2) {
    let mut map_x = start.x as i32;
    let mut map_y = start.y as i32;

    let delta_dist_x = if ray_dir.x == 0.0 {
        f64::MAX
    } else {
        f64::abs(1.0 / ray_dir.x)
    };

    let delta_dist_y = if ray_dir.y == 0.0 {
        f64::MAX
    } else {
        f64::abs(1.0 / ray_dir.y)
    };

    let mut hit = false;

    // Calculate initial size_distance and step direction
    //let mut step_x: i32 = 0;
    // Initial X
    let (step_x, mut side_dist_x) = if ray_dir.x < 0.0 {
        (-1i32, (start.x - map_x as f64) * delta_dist_x)
    } else {
        (1i32, (map_x as f64 + 1.0 - start.x) * delta_dist_x)
    };

    // Initial Y
    let(step_y, mut side_dist_y) = if ray_dir.y < 0.0 {
        (-1i32, (start.y - map_y as f64) * delta_dist_y)
    } else {
        (1i32, (map_y as f64 + 1.0 - start.y) * delta_dist_y)
    };

    // Look for final collision
    let mut side = HitSide::Horizontal;
    let mut cell_hit_type = GridCellType::Empty;

    while !hit {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = HitSide::Vertical;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = HitSide::Horizontal;
        }

        let cell = grid.get_cell_at_grid_coords_int(IVec2{x: map_x, y:map_y});
        match cell {
            None => {hit = true;}
            Some(x) => {
                match &x.cell_type {
                    GridCellType::Empty => {}
                    GridCellType::Wall => {
                        hit = true;
                        cell_hit_type = x.cell_type;
                    }
                }
            }
        }

        let current_position = side_dist_x.min(side_dist_y) * *ray_dir;
        if current_position.length() >= max_ray_distance {
            cell_hit_type = GridCellType::Empty;
            break;
        }
    }

    let perp_wall_distance = if side == HitSide::Vertical {
        side_dist_x - delta_dist_x
    } else {
        side_dist_y - delta_dist_y
    };

    (perp_wall_distance, cell_hit_type, side, IVec2::from((map_x, map_y)))
}