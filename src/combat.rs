use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash,)]
pub enum Element {
    #[default]
    Neutral,
    Water,
    Fire,
    Wind,
    Earth,
    Inw,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct AttackElement(pub Element);

#[derive(Component, Debug, Clone, Copy)]
pub struct DefenseElement(pub Element);

#[derive(Component, Debug, Clone, Copy)]
pub struct BaseStats {
    pub max_hp: f32,
    pub max_mp: f32,
    pub attack: f32,
    pub defense: f32,
    pub critical_rate: f32,
    pub critical_damage: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct CombatStats {
    pub max_hp: f32,
    pub max_mp: f32,
    pub attack: f32,
    pub defense: f32,
    pub critical_rate: f32,
    pub critical_damage: f32,
}

impl From<BaseStats> for CombatStats {
    fn from(base: BaseStats) -> Self {
        Self {
            max_hp: base.max_hp,
            max_mp: base.max_mp,
            attack: base.attack,
            defense: base.defense,
            critical_rate: base.critical_rate,
            critical_damage: base.critical_damage,
        }
    }
}

impl BaseStats {
    pub const PLAYER: Self = Self {
        max_hp: 300.0,
        max_mp: 300.0,
        attack: 15.0,
        defense: 15.0,
        critical_rate: 0.05,
        critical_damage: 1.5,
    };

    pub const BASIC_PRACTICE_GUN: Self = Self {
        max_hp: 100.0,
        max_mp: 0.0,
        attack: 5.0,
        defense: 2.0,
        critical_rate: 0.0,
        critical_damage: 1.0,
    };

    pub const ADVANCED_PRACTICE_MINION: Self = Self {
        max_hp: 120.0,
        max_mp: 0.0,
        attack: 9.0,
        defense: 8.0,
        critical_rate: 0.0,
        critical_damage: 1.0,
    };
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ElementProgress {
    pub level: u16,
    pub exp: u32,
}

impl ElementProgress {
    pub fn bonus_steps(&self) -> u16 {
        self.level / 10
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct ElementMastery {
    pub water: ElementProgress,
    pub fire: ElementProgress,
    pub wind: ElementProgress,
    pub earth: ElementProgress,
    pub inw: ElementProgress,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ExpRange {
    pub min: u32,
    pub max: u32,
}

impl ExpRange {
    pub const fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ElementExpReward {
    pub water: ExpRange,
    pub fire: ExpRange,
    pub wind: ExpRange,
    pub earth: ExpRange,
    pub inw: ExpRange,
}

impl ElementExpReward {
    pub const BASIC_PRACTICE_GUN: Self = Self {
        water: ExpRange::new(2, 5),
        fire: ExpRange::new(2, 5),
        wind: ExpRange::new(2, 5),
        earth: ExpRange::new(2, 5),
        inw: ExpRange::new(2, 3),
    };

    pub const ADVANCED_PRACTICE_MINION: Self = Self {
        water: ExpRange::new(6, 9),
        fire: ExpRange::new(6, 9),
        wind: ExpRange::new(6, 9),
        earth: ExpRange::new(6, 9),
        inw: ExpRange::new(4, 7),
    };
}