use bevy::prelude::*;
use crate::components::combat_cpn::Element;

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

#[derive(Resource, Default)]
pub struct ChokyHasSpawned(pub bool);

/// ปุ่ม + / - ประจำแต่ละธาตุในแผงจัดสรร
#[derive(Component, Clone, Copy)]
pub struct AllocButton {
    pub element: Element,
    pub increase: bool,
}

/// Text แสดง EXP ของธาตุไหน (ไว้อัพเดตตัวเลขทุกเฟรม)
#[derive(Component, Clone, Copy)]
pub struct AllocElementText(pub Element);

/// Text แสดงค่ากลางปัจจุบัน
#[derive(Component)]
pub struct AllocPoolText;