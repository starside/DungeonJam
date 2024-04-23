use crate::mob;
use crate::mob::{MagicColor, MobId};

pub const PLAYER_HIT_DISTANCE: f64 = 0.4;

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

pub enum DamageIndicator {
    PlayerHit,
    PlayerHeal,
    Other
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

    pub fn damage_target(&self, player_hp: &mut f64, max_player_hp: f64, player_color: MagicColor) -> DamageIndicator {
        let player_to_monster_damage: f64 = 33.0;
        let monster_to_player_damage: f64 = 100.0;
        let ct = &self.collision_type;
        match ct {
            CollisionType::Bullet(m, bullet_color) => {
                match m {
                    MobId::NoMob => {DamageIndicator::Other}
                    MobId::Mob(mob) => {
                        let mut m = mob.borrow_mut();
                        if m.get_color() != *bullet_color {
                            m.hp -= player_to_monster_damage;
                        } else {
                            m.hp += player_to_monster_damage;
                        }
                        m.hp = m.hp.clamp(0.0, mob::MONSTER_HP);
                        if m.hp <= 0.0 {
                            m.is_alive = false;
                        }
                        DamageIndicator::Other
                    }
                    MobId::Player => {
                        let (new_hp, indicator) = if player_color != *bullet_color {
                            (*player_hp - monster_to_player_damage,
                            DamageIndicator::PlayerHit)
                        } else {
                            (*player_hp + monster_to_player_damage/4.0,
                            DamageIndicator::PlayerHeal)
                        };
                        *player_hp = new_hp.clamp(0.0, max_player_hp);
                        indicator
                    }
                }
            }
        }
    }
}