use bevy::prelude::*;

#[derive(Resource)]
pub struct EnemyDadaAnimationGraph {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub chase: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurt: AnimationNodeIndex,
    pub dead: AnimationNodeIndex,
}

#[derive(Component)]
pub struct EnemyDadaAnimationTarget {
    pub root: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyDadaAnimState {
    Idle,
    Chase,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct DadaPunchHitbox {
    pub owner: Entity,
    pub has_hit: bool,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct DadaAttackTimer(pub Timer);

#[derive(Resource)]
pub struct DadaRespawnTimer(pub Timer);

#[derive(Component)]
pub struct DadaPatrol {
    pub direction: Vec3,
    pub timer: Timer,
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Resource, Default, Debug)]
pub struct DadaSpawnedCount(pub u32);

#[derive(Resource, Default)]
pub struct DadaDefeatedCount(pub u32);

pub const DADA_DEFEAT_GOAL: u32 = 5;

#[derive(Resource)]
pub struct DadaGroupAggro {
    pub active: bool,
    pub timer: Timer,
}

impl Default for DadaGroupAggro {
    fn default() -> Self {
        Self {
            active: false,
            timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

impl DadaGroupAggro {
    pub fn trigger(&mut self, duration: f32) {
        self.active = true;
        self.timer = Timer::from_seconds(duration, TimerMode::Once);
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        if !self.active {
            return;
        }

        self.timer.tick(delta);

        if self.timer.is_finished() {
            self.active = false;
        }
    }
}