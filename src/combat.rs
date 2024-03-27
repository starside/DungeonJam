use macroquad::math::DVec2;
use crate::mob::{MagicColor, MobId};

pub enum CollisionType {
    Bullet(MobId, MagicColor)
}

pub struct Collision {
    pub collision_type: CollisionType,
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
}