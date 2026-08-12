use bevy::prelude::*;

#[derive(Resource)]
pub struct GuardianAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub welcome: AnimationNodeIndex,
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum GuardianAnimState {
    Idle,
    Welcome,
}

#[derive(Component)]
pub struct GuardianAnimationTarget;

#[derive(Component)]
pub struct Npc;

#[derive(Component)]
pub struct GuardianNpc;

#[derive(Component)]
pub struct GuardianInteractArea;

#[derive(Component)]
pub struct PlayerInGuardianArea;

#[derive(Component)]
pub struct GuardianDialogUI;

#[derive(Component)]
pub struct PracticeEntity;

#[derive(Component)]
pub struct BasicPracticeGun;

#[derive(Resource, Default)]
pub struct BasicPracticeActive(pub bool);

#[derive(Resource)]
pub struct BasicGunRespawnTimer(pub Timer);

#[derive(Component)]
pub struct BasicGunShootTimer(pub Timer);

#[derive(Component)]
pub struct BasicPracticeProjectile {
    pub velocity: Vec3,
    pub hp_damage: i32,
}

#[derive(Component)]
pub struct ProjectileLifetime(pub Timer);

#[derive(Component)]
pub struct GuardianClone;

#[derive(Component)]
pub struct MinionLifeDrainTimer(pub Timer);

#[derive(Resource, Default)]
pub struct AdvancedPracticeActive(pub bool);

#[derive(Resource)]
pub struct AdvancedMinionRespawnTimer(pub Timer);

#[derive(Component)]
pub struct EnemyHealthBar {
    pub target: Entity,
}

#[derive(Component)]
pub struct EnemyHealthBarFill;