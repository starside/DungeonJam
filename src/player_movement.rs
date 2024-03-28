use macroquad::math::{DVec2, IVec2};
use crate::grid2d::{Grid2D, WallGridCell};
use crate::level::Level;
use crate::mob::MobId;
use crate::PlayerMode::Idle;

pub struct PlayerPosition {
    pos: IVec2
}

impl PlayerPosition {
    pub fn new(pos: (usize, usize), mob_grid: &mut Grid2D<MobId>)  -> Self{
        let mut np = PlayerPosition {
            pos: IVec2::from((pos.0 as i32, pos.1 as i32))
        };
        if np.set_pos(np.pos, mob_grid).is_err() {
            panic!("Failed to set initial player position");
        }
        np
    }
    pub fn get_pos(&self) -> IVec2 {
        self.pos
    }

    pub fn get_pos_dvec(&self) -> DVec2 {
        self.pos.as_dvec2()
    }

    pub fn get_pos_ituple(&self) -> (i32, i32) {
        (self.pos.x, self.pos.y)
    }

    pub fn try_set_pos(&self, new_pos: IVec2, mob_grid: &Grid2D<MobId>) -> Result<(), MobId> {
        if self.pos == new_pos {
            return Ok(()) // Didn't move
        }
        let res = match mob_grid.get_cell_at_grid_coords_int(new_pos) {
            None => { Err(MobId::NoMob) }
            Some(mob) => {
                match mob {
                    MobId::NoMob => {Ok(())}
                    MobId::Mob(_) => {Err(mob.clone())}
                    MobId::Player => {panic!("Player is in two places at once")} // Did check at top for move to same location
                }
            }
        };
        res
    }

    pub fn set_pos(&mut self, new_pos: IVec2, mob_grid: &mut Grid2D<MobId>) -> Result<(), MobId> {
        let res = self.try_set_pos(new_pos, mob_grid);
        if res.is_ok() {
            mob_grid.set_cell_at_grid_coords_int(self.pos, MobId::NoMob);
            mob_grid.set_cell_at_grid_coords_int(new_pos, MobId::Player);
            self.pos = new_pos;
        }

        res
    }
}

pub fn has_floor(pos: IVec2, level: &Level ) -> Option<IVec2> {
    let down_pos = pos + IVec2::from((0, 1));
    let cell = level.grid.get_cell_at_grid_coords_int(down_pos);
    match cell {
        None => {Some(down_pos)}
        Some(x) => {
            match x {
                WallGridCell::Empty => {None}
                WallGridCell::Wall => {Some(down_pos)}
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
            match x {
                WallGridCell::Empty => {false}
                WallGridCell::Wall => {true}
            }
        }
    }
}

pub fn can_stem(pos: IVec2, level: &Level) -> bool {
    let left_pos = pos + IVec2::from((-1, 0));
    let right_pos = pos + IVec2::from((1, 0));
    let left_cell = *level.grid.get_cell_at_grid_coords_int(left_pos).unwrap();
    let right_cell = *level.grid.get_cell_at_grid_coords_int(right_pos).unwrap();

    if left_cell == WallGridCell::Wall && right_cell == WallGridCell::Wall {
        true
    } else {
        false
    }
}

pub fn can_straddle_drop(pos: IVec2, level: &Level) -> Option<bool> {
    if has_floor(pos, level).is_some() { // Not a pit
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
    can_stem(up_pos, level) || is_supported_position(up_pos, level)
}

pub fn can_climb_down(pos: IVec2, level: &Level) -> bool {
    if has_floor(pos, level).is_some() {
        return false;
    }

    let down = pos + IVec2::from((0, 1));
    can_stem(down, level)
}

// Is the position supported, or is a fall guaranteed?
pub fn is_supported_position(pos: IVec2, level: &Level) -> bool {
    if let Some(x) = can_straddle_drop(pos, level) {
        return x || can_stem(pos, level);
    }

    true // Not a drop
}

pub fn is_wall(pos: IVec2, level: &Level) -> bool {
    if let Some(x) = level.grid.get_cell_at_grid_coords_int(pos) {
        match x {
            WallGridCell::Empty => {false}
            WallGridCell::Wall => {true}
        }
    } else {
        true
    }
}

pub enum MoveDirection {
    WalkForward,
    WalkBackward,
    ClimbUp,
    ClimbDown
}

pub fn is_room_occupiable(pos: IVec2, mob_grid: &Grid2D<MobId>) -> bool {
    match mob_grid.get_cell_at_grid_coords_int(pos) {
        None => {false}
        Some(x) => {
            match x {
                MobId::NoMob => {true}
                MobId::Mob(_) => {false}
                MobId::Player => {true}
            }
        }
    }
}

pub fn try_move(pos: IVec2, dir: MoveDirection, facing: i32, level: &Level, mob_grid: &Grid2D<MobId>) -> Option<IVec2> {
    assert_eq!(facing.abs(), 1);
    match dir {
        MoveDirection::WalkForward => {
            let new_pos = pos - IVec2::from((facing, 0));
            if !is_wall(new_pos, level) && is_room_occupiable(new_pos, mob_grid) {
                Some(new_pos)
            } else {
                None
            }
        }
        MoveDirection::WalkBackward => {
            let new_pos = pos + IVec2::from((facing, 0));
            if !is_wall(new_pos, level) && is_room_occupiable(new_pos, mob_grid) {
                Some(new_pos)
            } else {
                None
            }
        }
        MoveDirection::ClimbUp => {
            if can_climb_up(pos, level) {
                let new_pos = pos + IVec2::from((0, -1));
                if is_room_occupiable(new_pos, mob_grid) {
                    Some(new_pos)
                } else {
                    None
                }
            } else {
                None
            }
        }
        MoveDirection::ClimbDown => {
            if can_climb_down(pos, level) || !has_floor(pos, level).is_some() {
                let new_pos = pos + IVec2::from((0, 1));
                if is_room_occupiable(new_pos, mob_grid) {
                    Some(new_pos)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}