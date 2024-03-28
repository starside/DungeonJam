use std::cell::RefCell;
use std::rc::Rc;
use macroquad::math::{DVec2, IVec2};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::DeserializeOwned;
use crate::grid2d::{Grid2D, WallGridCell};
use crate::level::Level;
use crate::mob::MagicColor::White;
use crate::raycaster::cast_ray;

type AliveDead = bool;

pub const monster_hp:f64 = 100.0;
const monster_move_cooldown: f64 = 1.0;
const monster_color_change_cooldown: f64 = 1.0;

const monster_attack_cooldown: f64 = 1.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MagicColor {
    White,
    Black
}

pub struct MonsterState{
    last_move_time: f64, // time since last move completed
    last_attack_time: f64,
    last_color_change_time: f64,
}

impl MonsterState {
    pub fn update(&mut self, last_frame_time: f64) {
        self.last_move_time = (self.last_move_time - last_frame_time).clamp(0.0, monster_move_cooldown);
        self.last_attack_time = (self.last_attack_time - last_frame_time).clamp(0.0, monster_attack_cooldown);
        self.last_color_change_time = (self.last_color_change_time - last_frame_time).clamp(0.0, monster_color_change_cooldown);
    }

    pub fn can_attack(&self) -> bool {
        self.last_attack_time == 0.0
    }

    pub fn start_attack_cooldown(&mut self) {
        self.last_attack_time = monster_attack_cooldown;
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
    pub pos: DVec2,
    pub mob_type: MobType,
    pub color: MagicColor
}

impl MobData {
    pub fn has_line_of_sight<T>(&self, target: DVec2, grid: &Grid2D<T>)
        where T: Default + Clone + Serialize + DeserializeOwned + Into<WallGridCell> {
        let dir = (target - self.pos).normalize();
         cast_ray(grid, &self.pos, &dir, 10.0);
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        todo!()
    }
}

impl<'de> Deserialize<'de> for MobId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        todo!()
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error> where D: Deserializer<'de> {
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
    pub fn new_monster(&mut self, pos: IVec2, mob_grid: &mut Grid2D<MobId>) -> bool {
        if let Some(m) = mob_grid.get_cell_at_grid_coords_int(pos) {
            match m {
                MobId::NoMob => { // No mob here
                    let float_speed = 1.0; // In world coordinates per second
                    let offset = DVec2::from((0.5, 0.5));
                    let real_pos = pos.as_dvec2() + offset;

                    let mob = MobData {
                        is_alive: true,
                        hp: monster_hp,
                        moving: None,
                        move_speed: float_speed,
                        pos: real_pos,
                        color: White,
                        mob_type: MobType::Monster(
                            MonsterState {
                                last_move_time: monster_move_cooldown,
                                last_attack_time: monster_attack_cooldown,
                                last_color_change_time: monster_color_change_cooldown,
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