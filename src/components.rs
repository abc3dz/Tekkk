use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

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
pub struct PlayerHurtTimer(pub Timer);

#[derive(Component)]
pub struct PlayerPowerTimer(pub Timer);

#[derive(Component)]
pub struct PlayerFootstepSound;

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
    pub owner: Entity,
    pub already_hit: Vec<Entity>,
}

#[derive(Component)]
pub struct PlayerPunchHitboxLifetime(pub Timer);

#[derive(Debug, Clone, Copy)]
pub enum FloatingDamageKind {
    EnemyNormal,
    EnemyCritical,
    PlayerHit,
    PlayerDrain,
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

//
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefeatParticleState {
    Rising,
    ChasingPlayer,
}

#[derive(Component)]
pub struct DefeatParticle {
    pub velocity: Vec3,

    // ตอนนี้ลูกกลมอยู่ช่วงไหน
    pub state: DefeatParticleState,

    // เวลาที่ให้ลอยขึ้นก่อนเริ่มวิ่งหา Player
    pub state_timer: Timer,

    // กันลูกกลมค้าง หากหา Player ไม่เจอ
    pub lifetime: Timer,
}

//Enemy
#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug, Default, Clone, Copy)]
pub enum EnemyState {
    #[default]
    Idle,
    Chase,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct EnemyStateTimer(pub Timer);

#[derive(Component, Debug)]
pub struct EnemyInvestigateDirection {
    pub direction: Vec3,
    pub timer: Timer,
}

//Muamua
#[derive(Resource)]
pub struct EnemyMuamuaAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub chase: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurt: AnimationNodeIndex,
    pub dead: AnimationNodeIndex,
}

#[derive(Component)]
pub struct EnemyMuamuaAnimationTarget {
    pub root: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyMuamuaAnimState {
    Idle,
    Chase,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct MuamuaPunchHitbox {
    pub owner: Entity,
    pub has_hit: bool,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct MuamuaAttackTimer(pub Timer);

#[derive(Resource)]
pub struct MuamuaRespawnTimer(pub Timer);


//Choky
#[derive(Resource)]
pub struct EnemyChokyAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub chase: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurt: AnimationNodeIndex,
    pub dead: AnimationNodeIndex,
}

#[derive(Component)]
pub struct EnemyChokyAnimationTarget {
    pub root: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyChokyAnimState {
    Idle,
    Chase,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct ChokyPunchHitbox {
    pub owner: Entity,
    pub has_hit: bool,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct ChokyAttackTimer(pub Timer);

#[derive(Resource)]
pub struct ChokyRespawnTimer(pub Timer);