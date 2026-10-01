use bevy::prelude::*;
use crate::components::combat_cpn::Element;

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

#[derive(Component)]
pub struct GuardianMenuButton;

#[derive(Component, Clone, Copy)]
pub enum GuardianMenuAction {
    BasicPractice,
    AdvancedPractice,
    FullHpMana,
}

#[derive(Resource, Default)]
pub struct GuardianMenuSelection {
    pub index: usize,
}
// ปุ่ม + / - ประจำแต่ละธาตุในแผงจัดสรร
#[derive(Component, Clone, Copy)]
pub struct AllocButton {
    pub element: Element,
    pub increase: bool,
}

// Text แสดง EXP ของธาตุไหน (ไว้อัพเดตตัวเลขทุกเฟรม)
#[derive(Component, Clone, Copy)]
pub struct AllocElementText(pub Element);

// Text แสดงค่ากลางปัจจุบัน
#[derive(Component)]
pub struct AllocPoolText;

// ปุ่มเลือกธาตุโจมตี/ป้องกัน ในแผงเทพผู้พิทักษ์
#[derive(Component, Clone, Copy)]
pub struct AtkElementButton(pub Element);

// Text แสดงธาตุโจมตีปัจจุบัน
#[derive(Component)]
pub struct AtkElementText;

#[derive(Resource, Default)]
pub struct GuardianDialogOpen(pub bool);

#[derive(Resource, Default)]
pub struct GuardianDialogEscConsumed(pub bool);

#[derive(Resource, Default)]
pub struct GuardianAllocSelection {
    pub index: usize,
}

#[derive(Resource, Default)]
pub struct GuardianAllocPointSelection {
    pub index: usize,
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum GuardianDialogFocus {
    #[default]
    Menu,
    Allocation,
}

#[derive(Component)]
pub struct GuardianMenuPanel;

#[derive(Component)]
pub struct GuardianAllocPanel;

#[derive(Resource)]
pub struct GuardianAllocIncreaseRepeat(pub Timer);

#[derive(Resource)]
pub struct GuardianAllocDecreaseRepeat(pub Timer);

impl Default for GuardianAllocIncreaseRepeat {
    fn default() -> Self {
        // ถ้าอยากให้เร็วขึ้น ลดเลขนี้ เช่น 0.04 หรือ 0.03
        Self(Timer::from_seconds(0.06, TimerMode::Repeating))
    }
}

impl Default for GuardianAllocDecreaseRepeat {
    fn default() -> Self {
        // ถ้าอยากให้เร็วขึ้น ลดเลขนี้ เช่น 0.04 หรือ 0.03
        Self(Timer::from_seconds(0.06, TimerMode::Repeating))
    }
}
#[derive(Default)]
pub struct GuardianAllocPressState {
    pub increase: bool,
    pub decrease: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GuardianControlKey {
    MenuTitle,
    MenuUp,
    MenuDown,
    MenuConfirm,
    MenuSwitchAlloc,
    MenuClose,

    AllocTitle,
    AllocUp,
    AllocDown,
    AllocIncrease,
    AllocDecrease,
    AllocAtkLeft,
    AllocAtkRight,
    AllocConfirm,
    AllocSwitchMenu,
}

#[derive(Component)]
pub struct GuardianControlLabel(pub GuardianControlKey);