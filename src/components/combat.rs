use bevy::prelude::*;

#[derive(Component)] //
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)] //
pub struct Mana {
    pub current: i32,
    pub max: i32,
}