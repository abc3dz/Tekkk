use bevy::prelude::*;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefeatParticleState {
    Rising,
    ChasingPlayer,
}