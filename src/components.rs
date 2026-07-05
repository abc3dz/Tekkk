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

#[derive(Component)]
pub struct PlayerAnimationTarget;

#[derive(Resource)]
pub struct GuardianAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub welcome: AnimationNodeIndex,
    pub basic_practice: AnimationNodeIndex,
    pub advanced_practice: AnimationNodeIndex,
}

#[derive(Component)]
pub struct GuardianAnimationTarget;

#[derive(Component)]
pub struct CurrentScene;

#[derive(Component)]
pub struct LoadingUI;

#[derive(Component)]
pub struct WarpToDesert;

#[derive(Component)]
pub struct Npc;

#[derive(Component)]
pub struct GuardianNpc;

#[derive(Component)]
pub struct GuardianInteractArea;

#[derive(Component)]
pub struct PlayerInGuardianArea;

// #[derive(Component)]
// pub struct GuardianPopupUI;

#[derive(Component)]
pub struct GuardianMenuUI;

#[derive(Component)]
pub struct HubOnly;