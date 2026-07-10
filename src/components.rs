use bevy::prelude::*;

//player

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MoveSpeed(pub f32);

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

#[derive(Component)]
pub struct PlayerStatusUI;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct ManaBarFill;

#[derive(Resource)]
pub struct PlayerAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub punch_r: AnimationNodeIndex,
    pub punch_l: AnimationNodeIndex,
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum PlayerAnimState {
    Idle,
    Walk,
    PunchR,
    PunchL,
}

#[derive(Component)]
pub struct PlayerCombo {
    pub current_index: Option<usize>,
    pub queued_next: bool,
    pub timer: Timer,
}

#[derive(Component)]
pub struct FloatingDamageText {
    pub timer: Timer,
    pub world_position: Vec3,
    pub velocity: Vec3,
}

#[derive(Component)]
pub struct PlayerAnimationTarget;

//guardian

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

#[derive(Component)]
pub struct BasicGunHealthBarFill;

#[derive(Component)]
pub struct BasicGunShootTimer(pub Timer);

#[derive(Component)]
pub struct BasicPracticeProjectile {
    pub velocity: Vec3,
    pub hp_damage: i32,
    pub mana_damage: i32,
}

#[derive(Component)]
pub struct ProjectileLifetime(pub Timer);

#[derive(Component)]
pub struct GuardianClone;

#[derive(Component)]
pub struct MinionLifeDrainTimer(pub Timer);

#[derive(Component)]
pub struct MinionHealthBarFill;

// scenes

#[derive(Component)]
pub struct CurrentScene;

#[derive(Component)]
pub struct LoadingUI;

#[derive(Component)]
pub struct WarpToDesert;

#[derive(Component)]
pub struct WarpToHub;

#[derive(Component)]
pub struct HubOnly;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameScene {
    #[default]
    LoadingHub,
    Hub,
    LoadingDesert,
    Desert,
}