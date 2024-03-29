use std::cell::RefCell;
use std::ops::Neg;
use std::rc::Rc;

use macroquad::math::{DVec2, IVec2};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::DeserializeOwned;

use crate::grid2d::{Grid2D, WallGridCell};
use crate::level::{apply_boundary_conditions_f64, ucoords_to_dvec2};
use crate::raycaster::cast_ray;

type AliveDead = bool;

pub const MONSTER_HP:f64 = 100.0;
const MONSTER_MOVE_COOLDOWN: f64 = 6.0;
const MONSTER_ATTACK_COOLDOWN: f64 = 6.0;
const MONSTER_COLOR_CHANGE_COOLDOWN: f64 = 4.0;

const MONSTER_LINE_OF_SIGHT: f64 = 12.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MagicColor {
    White,
    Black
}

impl MagicColor {
    pub fn get_opposite(&self) -> Self {
        match self {
            MagicColor::White => {MagicColor::Black}
            MagicColor::Black => {MagicColor::White}
        }
    }
}

pub struct MonsterState{
    last_move_time: f64, // time since last move completed
    last_attack_time: f64,
    last_color_change_time: f64,
}

impl MonsterState {
    pub fn update(&mut self, last_frame_time: f64) {
        self.last_move_time = (self.last_move_time - last_frame_time).clamp(0.0, MONSTER_MOVE_COOLDOWN);
        self.last_attack_time = (self.last_attack_time - last_frame_time).clamp(0.0, MONSTER_ATTACK_COOLDOWN);
        self.last_color_change_time = (self.last_color_change_time - last_frame_time).clamp(0.0, MONSTER_COLOR_CHANGE_COOLDOWN);
    }

    pub fn can_attack(&self) -> bool {
        self.last_attack_time == 0.0
    }

    pub fn can_change_color(&self) -> bool {
        self.last_color_change_time == 0.0
    }

    pub fn can_move(&self) -> bool {
        self.last_move_time == 0.0
    }

    pub fn start_attack_cooldown(&mut self) {
        self.last_attack_time = MONSTER_ATTACK_COOLDOWN;
    }

    pub fn start_color_change_cooldown(&mut self) {
        self.last_color_change_time = MONSTER_COLOR_CHANGE_COOLDOWN;
    }

    pub fn start_move_cooldown(&mut self, modifier: f64) {
        self.last_move_time = MONSTER_MOVE_COOLDOWN * modifier;
    }
}

pub enum MobType {
    Monster(MonsterState),
    Bullet
}
pub struct MobData {
    pub is_alive: AliveDead,
    pub hp: f64,
    pub moving: Option<(DVec2, DVec2, f64)>, // start coord, end coord, lerp
    pub move_speed: f64,
    pos: DVec2,
    pub mob_type: MobType,
    color: MagicColor
}

impl MobData {
    pub fn get_pos(&self) -> DVec2 {
        self.pos
    }

    pub fn set_pos(&mut self, pos: DVec2, world_size: (usize, usize)) {
        self.pos = apply_boundary_conditions_f64(pos, world_size);
    }

    pub fn set_pos_centered(&mut self, pos: DVec2, world_size: (usize, usize)) {
        self.pos = apply_boundary_conditions_f64(pos + DVec2::new(0.5, 0.5), world_size);
    }


    pub fn has_line_of_sight_with_bc<T>(&self, target: DVec2, grid: &Grid2D<T>) -> Option<(IVec2, DVec2)> // hit coord, direction
        where T: Default + Clone + Serialize + DeserializeOwned + Into<WallGridCell>{
        let ws = ucoords_to_dvec2(grid.get_size());
        let target = apply_boundary_conditions_f64(target, grid.get_size());
        let (right, left) = if self.pos.x < target.x { // target to our right
            let target_left = DVec2::new((ws.x - target.x).neg(), target.y);
            ((self.has_line_of_sight(target, grid), target - self.pos), // shoot right
             (self.has_line_of_sight(target_left, grid), target_left - self.pos)) // shot left
        } else { //target to our left;
            let target_right = DVec2::new(ws.x + target.x, target.y);
            ((self.has_line_of_sight(target_right, grid), target_right - self.pos), // shoot right
             (self.has_line_of_sight(target, grid), target - self.pos)) // shoot left
        };

        let left: Option<(IVec2, DVec2)>  = if left.0.is_some() {
            Some((left.0.unwrap(), left.1))
        } else {
            None
        };

        let right: Option<(IVec2, DVec2)> = if right.0.is_some() {
            Some((right.0.unwrap(), right.1))
        } else {
            None
        };

        let mut los = left;

        if let Some(r) = right {
            if let Some(l) = los {
                let dist_to_right = self.pos.distance_squared(r.0.as_dvec2());
                let dist_to_left = self.pos.distance_squared(l.0.as_dvec2());
                if dist_to_left < dist_to_right {
                    los = Some(l);
                } else {
                    los = Some(r);
                }
            } else { // los is None, use right
                los = Some(r);
            }
        }
        los
    }
    pub fn has_line_of_sight<T>(&self, target: DVec2, grid: &Grid2D<T>) -> Option<IVec2>
        where T: Default + Clone + Serialize + DeserializeOwned + Into<WallGridCell> {
        let sight_vector = target - self.pos;
        let dir = sight_vector.normalize();
        let (_, _, _, coord) = cast_ray(grid, &self.pos, &dir, MONSTER_LINE_OF_SIGHT);
        let hit_coord = coord.as_dvec2() + DVec2::new(0.5, 0.5); // Find center of coordinate hit
        let hit_distance= hit_coord.distance(self.pos);
        let sight_distance = sight_vector.length();

        let has_los = if hit_distance <= sight_distance {
            if sight_distance - hit_distance < 0.2 { // fudge factor
                true
            } else{
                false
            }
        } else {
            true
        };
        if has_los {
            Some(coord)
        } else {
            None
        }
    }

    pub fn set_color(&mut self, color: MagicColor) {
        self.color = color;
    }

    pub fn get_color(&self) -> MagicColor {
        self.color
    }
}

pub type Mob = Rc<RefCell<Box<MobData>>>;
#[derive(Default, Clone)]
pub enum MobId {
    #[default]
    NoMob,
    Mob(Mob),
    Player
}

impl From<MobId> for WallGridCell {
    fn from(value: MobId) -> Self {
        match value {
            MobId::NoMob => {WallGridCell::Empty}
            MobId::Mob(_) => {WallGridCell::Wall}
            MobId::Player => {WallGridCell::Wall}
        }
    }
}

impl Serialize for MobId {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error> where S: Serializer {
        todo!()
    }
}

impl<'de> Deserialize<'de> for MobId {
    fn deserialize<D>(_: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        todo!()
    }

    fn deserialize_in_place<D>(_: D, _: &mut Self) -> Result<(), D::Error> where D: Deserializer<'de> {
        todo!()
    }
}



pub struct Mobs {
    pub mob_list: Vec<Mob>
}

impl Mobs {
    pub fn new() -> Self {
        Mobs {
            mob_list: Vec::new()
        }
    }

    pub fn delete_dead_mobs(&mut self, mob_grid: &mut Grid2D<MobId>) {
        let mut dead_mobs: Vec<usize> = Vec::new(); // TODO: create outside game loop
        for (i, mob) in self.mob_list.iter().enumerate() {
            let mob = mob.borrow();
            if !mob.is_alive {
                dead_mobs.push(i);
                match  &mob.mob_type {
                    MobType::Monster(_) => {
                        mob_grid.set_cell_at_grid_coords_int(mob.pos.as_ivec2(), MobId::NoMob);
                    }
                    MobType::Bullet => {}
                }
            }
        }
        dead_mobs.sort();
        for i in dead_mobs.iter().rev() {
            self.mob_list.swap_remove(*i);
        }
    }
    pub fn new_monster(&mut self, pos: IVec2, mob_grid: &mut Grid2D<MobId>, color: MagicColor) -> bool {
        if let Some(m) = mob_grid.get_cell_at_grid_coords_int(pos) {
            match m {
                MobId::NoMob => { // No mob here
                    let float_speed = 1.0; // In world coordinates per second
                    let offset = DVec2::from((0.5, 0.5));
                    let real_pos = pos.as_dvec2() + offset;

                    let mob = MobData {
                        is_alive: true,
                        hp: MONSTER_HP,
                        moving: None,
                        move_speed: float_speed,
                        pos: real_pos,
                        color,
                        mob_type: MobType::Monster(
                            MonsterState {
                                last_move_time: MONSTER_MOVE_COOLDOWN,
                                last_attack_time: MONSTER_ATTACK_COOLDOWN,
                                last_color_change_time: MONSTER_COLOR_CHANGE_COOLDOWN,
                            }
                        )
                    };

                    let new_mob = Rc::new(RefCell::new(Box::new(mob)));
                    mob_grid.set_cell_at_grid_coords_int(pos, MobId::Mob(new_mob.clone()));
                    self.mob_list.push(new_mob);
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn new_bullet(&mut self, pos: DVec2, dir: DVec2, color: MagicColor) -> usize {
        let float_speed = 2.0; // In world coordinates per second
        let max_lifetime = 5.0;
        let dir_vec = dir.normalize();

        let pos =
            pos + 0.25*dir_vec +
                ((0.55f64.powi(2) + 0.55f64.powi(2)).sqrt() * dir_vec); // start out of player room

        let end_pos = float_speed*max_lifetime*dir_vec + pos;

        let mob = MobData {
            is_alive: true,
            hp: 1.0,
            moving: Some((pos, end_pos, 0.0)),
            move_speed: float_speed,
            pos,
            color,
            mob_type: MobType::Bullet
        };

        self.mob_list.push(Rc::new(RefCell::new(Box::new(mob))));
        self.mob_list.len() - 1
    }
}

pub fn mob_at_cell(pos: IVec2, grid: &Grid2D<MobId>) -> MobId {
    if let Some(x) = grid.get_cell_at_grid_coords_int(pos) {
        x.clone()
    } else {
        MobId::NoMob
    }
}