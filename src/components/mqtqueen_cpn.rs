use bevy::prelude::*;

#[derive(Resource)]
pub struct EnemyMqtqueenAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub chase: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurt: AnimationNodeIndex,
    pub dead: AnimationNodeIndex,
}

#[derive(Component)]
pub struct EnemyMqtqueenAnimationTarget {
    pub root: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyMqtqueenAnimState {
    Idle,
    Chase,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct MqtqueenPunchHitbox {
    pub owner: Entity,
    pub has_hit: bool,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct MqtqueenAttackTimer(pub Timer);

#[derive(Resource)]
pub struct MqtqueenRespawnTimer(pub Timer);

#[derive(Component)]
pub struct MqtqueenPatrol {
    pub direction: Vec3,
    pub timer: Timer,
    pub center: Vec3,
    pub radius: f32,
}