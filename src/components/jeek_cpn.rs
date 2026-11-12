use bevy::prelude::*;

#[derive(Resource)]
pub struct EnemyJeekAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub chase: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurt: AnimationNodeIndex,
    pub dead: AnimationNodeIndex,
}

#[derive(Component)]
pub struct EnemyJeekAnimationTarget {
    pub root: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyJeekAnimState {
    Idle,
    Chase,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct JeekPunchHitbox {
    pub owner: Entity,
    pub has_hit: bool,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct JeekAttackTimer(pub Timer);

#[derive(Resource)]
pub struct JeekRespawnTimer(pub Timer);

#[derive(Component)]
pub struct JeekPatrol {
    pub direction: Vec3,
    pub timer: Timer,
    pub center: Vec3,
    pub radius: f32,
}