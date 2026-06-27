use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MoveSpeed(pub f32);

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}