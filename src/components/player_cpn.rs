use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
//use bevy::animation::AnimationEvent;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MoveSpeed(pub f32);

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
    pub slap_r: AnimationNodeIndex,
    pub slap_l: AnimationNodeIndex,
    pub slap_lr: AnimationNodeIndex,
    pub dash: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub hurt: AnimationNodeIndex,
    pub power: AnimationNodeIndex,
    pub dead: AnimationNodeIndex,
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum PlayerAnimState {
    Idle,
    Walk,
    SlapR,
    SlapL,
    SlapLR,
    Dash,
    Jump,
    Hurt,
    Power,
    Dead,
}

#[derive(Component)]
pub struct PlayerDashTimer(pub Timer);

#[derive(Component)]
pub struct PlayerDashMove {
    pub timer: Timer,
    pub direction: Vec3,
    pub speed: f32,
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

#[derive(Component)]
pub struct PlayerDashEffect {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PlayerDashTrailTimer(pub Timer);

#[derive(Component)]
pub struct PlayerJumpTimer(pub Timer);

#[derive(Component)]
pub struct PlayerHurtTimer();

#[derive(Component)]
pub struct PlayerPowerTimer(pub Timer);

#[derive(Component, Default)]
pub struct PlayerFootstepTracker {
    pub previous_time: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerHand {
    Left,
    Right,
}

#[derive(Component)]
pub struct PlayerHandBone(pub PlayerHand);

#[derive(Component)]
pub struct PunchHitboxRequest {
    pub owner: Entity,
    pub hand: PlayerHand,
    pub delay: Timer,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct PlayerPunchHitbox {
    // pub owner: Entity,
    // pub already_hit: Vec<Entity>,
}

#[derive(Component)]
pub struct PlayerPunchHitboxLifetime(pub Timer);

#[derive(Debug, Clone, Copy)]
pub enum FloatingDamageKind {
    EnemyNormal,
    EnemyCritical,
    PlayerHit,
    PlayerDrain,
    Heal,
    Mana,
}

#[derive(Component)]
pub struct PlayerSlapHitbox {
    pub lifetime: Timer,
    pub has_hit: bool,
}

pub const PLAYER_ENERGY_SHADER_PATH: &str = "shaders/player_energy_ball.wgsl";

#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)]
pub struct PlayerEnergyMaterial {}

impl Material for PlayerEnergyMaterial {
    fn fragment_shader() -> ShaderRef {
        PLAYER_ENERGY_SHADER_PATH.into()
    }
}

#[derive(Resource)]
pub struct PlayerEnergyAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<PlayerEnergyMaterial>,
}

#[derive(Component)]
pub struct PlayerEnergyBall {
    pub direction: Vec3,
    pub speed: f32,
    pub lifetime: Timer,
    pub has_hit: bool,
}