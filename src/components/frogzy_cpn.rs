use bevy::prelude::*;

#[derive(Resource)]
pub struct EnemyFrogzyAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub chase: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurt: AnimationNodeIndex,
    pub dead: AnimationNodeIndex,
}

#[derive(Component)]
pub struct EnemyFrogzyAnimationTarget {
    pub root: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyFrogzyAnimState {
    Idle,
    Chase,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct FrogzyPunchHitbox {
    pub owner: Entity,
    pub has_hit: bool,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct FrogzyAttackTimer(pub Timer);

#[derive(Resource)]
pub struct FrogzyRespawnTimer(pub Timer);

pub const FROGZY_SPAWN_LIMIT: u32 = 5;

#[derive(Resource, Default, Debug)]
pub struct FrogzySpawnedCount(pub u32);