use bevy::prelude::*;

#[derive(Resource)]
pub struct EnemyMuanimAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub chase: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurt: AnimationNodeIndex,
    pub dead: AnimationNodeIndex,
}

#[derive(Component)]
pub struct EnemyMuanimAnimationTarget {
    pub root: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyMuanimAnimState {
    Idle,
    Chase,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct MuanimPunchHitbox {
    pub owner: Entity,
    pub has_hit: bool,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct MuanimAttackTimer(pub Timer);

#[derive(Resource)]
pub struct MuanimRespawnTimer(pub Timer);

// #[derive(Component)]
// pub struct MuanimPatrol {
//     pub direction: Vec3,
//     pub timer: Timer,
// }

pub const MUANIM_SPAWN_LIMIT: u32 = 10;

#[derive(Resource, Default, Debug)]
pub struct MuanimSpawnedCount(pub u32);