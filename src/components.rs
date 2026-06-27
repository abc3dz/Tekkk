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

#[derive(Resource)]
pub struct PlayerAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum PlayerActionState {
    Idle,
    Walk,
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum PlayerAnimState {
    Idle,
    Walk,
}