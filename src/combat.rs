use macroquad::math::DVec2;
use crate::mob;
use crate::mob::{MagicColor, MobId, MobType};

pub enum CollisionType {
    Bullet(MobId, MagicColor)
}

pub struct Collision {
    pub collision_type: CollisionType,
}

#[derive(Debug)]
pub struct DamageReport {
    pub remaining_hp: f64,
    pub hit_damage: f64
}

impl Collision {
    pub fn new_with_bullet(target: MobId, color: MagicColor) -> Self {
        Collision {
            collision_type: CollisionType::Bullet(
                target,
                color
            )
        }
    }

    pub fn damage_target(&self, player_hp: &mut f64, max_player_hp: f64, player_color: MagicColor) {
        let player_to_monster_damage: f64 = 33.0;
        let monster_to_player_damage: f64 = 100.0;
        let ct = &self.collision_type;
        match ct {
            CollisionType::Bullet(m, bullet_color) => {
                match m {
                    MobId::NoMob => {}
                    MobId::Mob(mob) => {
                        let mut m = mob.borrow_mut();
                        if m.color != *bullet_color {
                            m.hp -= player_to_monster_damage;
                        } else {
                            m.hp += player_to_monster_damage/2.0;
                        }
                        m.hp = m.hp.clamp(0.0, mob::monster_hp);
                        println!("Monster hp is {}", m.hp);
                        if m.hp <= 0.0 {
                            m.is_alive = false;
                        }
                    }
                    MobId::Player => {
                        println!("Damaging player, player_color {:?}, bullet_color {:?}", player_color, bullet_color);
                        let new_hp = if player_color != *bullet_color {
                            *player_hp - monster_to_player_damage
                        } else {
                            *player_hp + monster_to_player_damage/4.0
                        };
                        *player_hp = new_hp.clamp(0.0, max_player_hp);
                    }
                }
            }
        }
    }
}