use bevy::prelude::*;
use rand::Rng;

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
pub struct AtkAndDefElement(pub Element);

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
    pub exp: u32,
}

#[derive(Component, Debug, Default, Clone)]
pub struct ElementMastery {
    pub water: ElementProgress,
    pub fire: ElementProgress,
    pub wind: ElementProgress,
    pub earth: ElementProgress,
    pub inw: ElementProgress,
}

impl ElementMastery {
    pub fn get_mut(
        &mut self,
        element: Element,
    ) -> Option<&mut ElementProgress> {
        match element {
            Element::Water => Some(&mut self.water),
            Element::Fire => Some(&mut self.fire),
            Element::Wind => Some(&mut self.wind),
            Element::Earth => Some(&mut self.earth),
            Element::Inw => Some(&mut self.inw),
            Element::Neutral => None,
        }
    }
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
    pub fn for_element(
        &self,
        element: Element,
    ) -> ExpRange {
        match element {
            Element::Water => self.water,
            Element::Fire => self.fire,
            Element::Wind => self.wind,
            Element::Earth => self.earth,
            Element::Inw => self.inw,
            Element::Neutral => ExpRange::default(),
        }
    }
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

#[derive(Component, Debug, Default)]
pub struct CombatTarget;

#[derive(Debug, Default, Clone, Copy)]
pub struct ElementExpGain {
    pub water: u32,
    pub fire: u32,
    pub wind: u32,
    pub earth: u32,
    pub inw: u32,
}

impl ExpRange {
    pub fn roll(self, rng: &mut impl Rng) -> u32 {
        if self.max < self.min {
            return 0;
        }

        rng.random_range(self.min..=self.max)
    }
}

impl ElementExpReward {
    pub fn grant_all(
        &self,
        mastery: &mut ElementMastery,
        rng: &mut impl Rng,
    ) -> ElementExpGain {
        let gain = ElementExpGain {
            water: self.water.roll(rng),
            fire: self.fire.roll(rng),
            wind: self.wind.roll(rng),
            earth: self.earth.roll(rng),
            inw: self.inw.roll(rng),
        };

        mastery.water.exp =
            mastery.water.exp.saturating_add(gain.water);

        mastery.fire.exp =
            mastery.fire.exp.saturating_add(gain.fire);

        mastery.wind.exp =
            mastery.wind.exp.saturating_add(gain.wind);

        mastery.earth.exp =
            mastery.earth.exp.saturating_add(gain.earth);

        mastery.inw.exp =
            mastery.inw.exp.saturating_add(gain.inw);

        gain
    }
}

pub fn elemental_multiplier(
    attacker: Element,
    defender: Element,
) -> f32 {
    use Element::*;

    match (attacker, defender) {
        // ชนะทาง
        (Water, Fire)
        | (Fire, Wind)
        | (Wind, Earth)
        | (Earth, Water) => 1.5,

        // แพ้ทาง
        (Water, Earth)
        | (Fire, Water)
        | (Wind, Fire)
        | (Earth, Wind) => 0.75,

        // ธาตุเดียวกัน Neutral และ Inw
        _ => 1.0,
    }
}
pub fn calculate_combat_damage(
    attacker: &CombatStats,
    defender: &CombatStats,
    rng: &mut impl rand::Rng,
) -> (i32, bool) {
    let damage_after_defense =
        attacker.attack * 100.0
            / (100.0 + defender.defense.max(0.0));

    // 0.05 หมายถึงโอกาส Critical 5%
    let is_critical =
        rng.random::<f32>()
            < attacker.critical_rate.clamp(0.0, 1.0);

    // ถ้าติด Critical จึงนำ Critical Damage มาคูณ
    let critical_multiplier = if is_critical {
        attacker.critical_damage.max(1.0)
    } else {
        1.0
    };

    let final_damage =
        damage_after_defense * critical_multiplier;

    let damage =
        final_damage.round().max(1.0) as i32;

    (damage, is_critical)
}

pub fn combat_stats_from_element_exp(
    base: &BaseStats,
    mastery: &ElementMastery,
) -> CombatStats {
    // ใช้ EXP / 10.0 โดยตรง
    let water = mastery.water.exp as f32 / 10.0;
    let fire = mastery.fire.exp as f32 / 10.0;
    let wind = mastery.wind.exp as f32 / 10.0;
    let earth = mastery.earth.exp as f32 / 10.0;
    let inw = mastery.inw.exp as f32 / 10.0;

    let hp_bonus =
        earth * 8.0
        + inw * 2.0;

    let mp_bonus =
        water * 10.0
        + inw * 2.0;

    let attack_bonus =
        water * 0.2
        + fire * 0.8
        + inw * 0.15;

    let defense_bonus =
        earth * 0.6
        + inw * 0.15;

    let critical_rate_bonus =
        wind * 0.005
        + inw * 0.001;

    let critical_damage_bonus =
        fire * 0.02
        + wind * 0.01
        + inw * 0.005;

    CombatStats {
        max_hp: base.max_hp + hp_bonus,
        max_mp: base.max_mp + mp_bonus,
        attack: base.attack + attack_bonus,
        defense: base.defense + defense_bonus,
        critical_rate:
            base.critical_rate + critical_rate_bonus,
        critical_damage:
            base.critical_damage + critical_damage_bonus,
    }
}