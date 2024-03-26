use macroquad::math::{DVec2, IVec2};
use crate::mob::MagicColor::White;

type AliveDead = bool;

pub enum MagicColor {
    White,
    Black
}

pub struct MonsterState{
    is_alive: AliveDead,
    last_move_time: f64, // time since last move completed
    last_attack_time: f64,
    last_color_change_time: f64,
}

pub enum MobType {
    Monster(MonsterState),
    Bullet
}
pub struct Mob {
    pub moving: Option<(DVec2, DVec2, f64)>, // start coord, end coord, lerp
    pub move_speed: f64,
    pub pos: DVec2,
    pub mob_type: MobType,
    pub color: MagicColor
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
    pub fn new_monster(&mut self, pos: IVec2) -> usize {
        let float_speed = 1.0; // In world coordinates per second
        let offset = DVec2::from((0.5, 0.5));
        let real_pos = pos.as_dvec2() + offset;

        let mob = Mob {
            moving: None,
            move_speed: float_speed,
            pos: real_pos,
            color: White,
            mob_type: MobType::Monster(
                MonsterState {
                    is_alive: true,
                    last_move_time: 0.0,
                    last_attack_time: 0.0,
                    last_color_change_time: 0.0
                }
            )
        };

        self.mob_list.push(mob);
        self.mob_list.len() - 1
    }

    pub fn new_bullet(&mut self, pos: DVec2, dir: DVec2, color: MagicColor) -> usize {
        let float_speed = 4.0; // In world coordinates per second
        let max_lifetime = 6.0;
        let end_pos = float_speed*max_lifetime*dir.normalize() + pos;

        let mob = Mob {
            moving: Some((pos, end_pos, 0.0)),
            move_speed: float_speed,
            pos,
            color,
            mob_type: MobType::Bullet
        };

        self.mob_list.push(mob);
        self.mob_list.len() - 1
    }
}