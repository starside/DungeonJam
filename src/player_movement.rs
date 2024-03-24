use macroquad::math::IVec2;
use crate::grid2d::{GridCellType, RayGridCell};
use crate::level::Level;

pub fn has_floor(pos: IVec2, level: &Level ) -> bool {
    let down_pos = pos + IVec2::from((0, 1));
    let cell = level.grid.get_cell_at_grid_coords_int(down_pos);
    match cell {
        None => {true}
        Some(x) => {
            match x.cell_type {
                GridCellType::Empty => {false}
                GridCellType::Wall => {true}
            }
        }
    }
}

pub fn has_ceiling(pos: IVec2, level: &Level ) -> bool {
    let up_pos = pos + IVec2::from((0, -1));
    let cell = level.grid.get_cell_at_grid_coords_int(up_pos);
    match cell {
        None => {true}
        Some(x) => {
            match x.cell_type {
                GridCellType::Empty => {false}
                GridCellType::Wall => {true}
            }
        }
    }
}

pub fn can_stem(pos: IVec2, level: &Level) -> bool {
    let left_pos = pos + IVec2::from((-1, 0));
    let right_pos = pos + IVec2::from((1, 0));
    let left_cell = level.grid.get_cell_at_grid_coords_int(left_pos).unwrap().cell_type;
    let right_cell = level.grid.get_cell_at_grid_coords_int(right_pos).unwrap().cell_type;

    if left_cell == GridCellType::Wall && right_cell == GridCellType::Wall {
        true
    } else {
        false
    }
}

pub fn can_straddle_drop(pos: IVec2, level: &Level) -> Option<bool> {
    if has_floor(pos, level) { // Not a pit
        return None;
    }

    let down_pos = pos + IVec2::from((0, 1));
    Some(can_stem(down_pos, level))
}

pub fn can_climb_up(pos: IVec2, level: &Level) -> bool {
    if has_ceiling(pos, level) {
        return false;
    }

    let up_pos = pos + IVec2::from((0, -1));
    can_stem(up_pos, level) || is_supported_position(pos, level)
}

pub fn can_climb_down(pos: IVec2, level: &Level) -> bool {
    if has_floor(pos, level) {
        return false;
    }

    let down = pos + IVec2::from((0, 1));
    can_stem(down, level)
}

// Is the position supported, or is a fall guaranteed?
pub fn is_supported_position(pos: IVec2, level: &Level) -> bool {
    if let Some(x) = can_straddle_drop(pos, level) {
        return x;
    }

    true // Not a drop
}

pub fn is_wall(pos: IVec2, level: &Level) -> bool {
    if level.grid.get_cell_at_grid_coords_int(pos).unwrap().cell_type == GridCellType::Wall {
        true
    } else {
        false
    }
}

pub enum MoveDirection {
    WalkForward,
    WalkBackward,
    ClimbUp,
    ClimbDown
}

pub fn try_move(pos: IVec2, dir: MoveDirection, facing: i32, level: &Level) -> Option<IVec2> {
    assert_eq!(facing.abs(), 1);
    match dir {
        MoveDirection::WalkForward => {
            let new_pos = pos - IVec2::from((facing, 0));
            if !is_wall(new_pos, level) {
                Some(new_pos)
            } else {
                None
            }
        }
        MoveDirection::WalkBackward => {
            let new_pos = pos + IVec2::from((facing, 0));
            if !is_wall(new_pos, level) {
                Some(new_pos)
            } else {
                None
            }
        }
        MoveDirection::ClimbUp => {
            if can_climb_up(pos, level) {
                let new_pos = pos + IVec2::from((0, -1));
                Some(new_pos)
            } else {
                None
            }
        }
        MoveDirection::ClimbDown => {
            if can_climb_down(pos, level) || !has_floor(pos, level) {
                let new_pos = pos + IVec2::from((0, 1));
                Some(new_pos)
            } else {
                None
            }
        }
    }
}