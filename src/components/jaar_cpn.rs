use bevy::prelude::*;

#[derive(Resource)]
pub struct EnemyJaarAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub chase: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurt: AnimationNodeIndex,
    pub dead: AnimationNodeIndex,
}

#[derive(Component)]
pub struct EnemyJaarAnimationTarget {
    pub root: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyJaarAnimState {
    Idle,
    Chase,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct JaarPunchHitbox {
    pub owner: Entity,
    pub has_hit: bool,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct JaarAttackTimer(pub Timer);

#[derive(Resource)]
pub struct JaarRespawnTimer(pub Timer);

pub const JAAR_SPAWN_LIMIT: u32 = 5;

#[derive(Resource, Default, Debug)]
pub struct JaarSpawnedCount(pub u32);