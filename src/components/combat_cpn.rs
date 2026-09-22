use bevy::prelude::*;
use rand::Rng;

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

    pub const MUANIM: Self = Self {
        max_hp: 100.0,
        max_mp: 0.0,
        attack: 10.0,
        defense: 5.0,
        critical_rate: 0.0,
        critical_damage: 1.0,
    };
    pub const MUAMUA: Self = Self {
        max_hp: 200.0,
        max_mp: 0.0,
        attack: 14.0,
        defense: 8.0,
        critical_rate: 0.0,
        critical_damage: 1.0,
    };
    pub const CHOKY: Self = Self {
        max_hp: 700.0,
        max_mp: 0.0,
        attack: 18.0,
        defense: 11.0,
        critical_rate: 2.0,
        critical_damage: 5.0,
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
    pub const BASIC_PRACTICE_GUN: Self = Self {
        water: ExpRange::new(2, 3),
        fire: ExpRange::new(2, 3),
        wind: ExpRange::new(2, 3),
        earth: ExpRange::new(2, 3),
        inw: ExpRange::new(1, 1),
    };

    pub const ADVANCED_PRACTICE_MINION: Self = Self {
        water: ExpRange::new(6, 9),
        fire: ExpRange::new(6, 9),
        wind: ExpRange::new(6, 9),
        earth: ExpRange::new(6, 9),
        inw: ExpRange::new(4, 7),
    };

    pub const MUANIM: Self = Self {
        water: ExpRange::new(3, 3),
        fire: ExpRange::new(0, 0),
        wind: ExpRange::new(0, 0),
        earth: ExpRange::new(10, 10),
        inw: ExpRange::new(1, 1),
    };
    pub const MUAMUA: Self = Self {
        water: ExpRange::new(5, 5),
        fire: ExpRange::new(0, 0),
        wind: ExpRange::new(0, 0),
        earth: ExpRange::new(20, 20),
        inw: ExpRange::new(1, 1),
    };
    pub const CHOKY: Self = Self {
        water: ExpRange::new(0, 0),
        fire: ExpRange::new(5, 5),
        wind: ExpRange::new(0, 0),
        earth: ExpRange::new(30, 30),
        inw: ExpRange::new(2, 2),
    };
}

#[derive(Component, Debug, Default)]
pub struct CombatTarget;

impl ExpRange {
    pub fn roll(self, rng: &mut impl Rng) -> u32 {
        if self.max < self.min {
            return 0;
        }
        rng.random_range(self.min..=self.max)
    }
}

pub fn elemental_multiplier(
    attacker: Element,
    defender: Element,
) -> f32 {
    use Element::*;

    match (attacker, defender) {
        // Water
        (Water, Water) => 1.0,
        (Water, Earth) => 0.75,
        (Water, Wind) => 1.0,
        (Water, Fire) => 1.5,
        (Water, Inw) => 0.75,

        // Earth
        (Earth, Water) => 1.5,
        (Earth, Earth) => 1.0,
        (Earth, Wind) => 0.75,
        (Earth, Fire) => 1.0,
        (Earth, Inw) => 0.75,

        // Wind
        (Wind, Water) => 1.0,
        (Wind, Earth) => 1.5,
        (Wind, Wind) => 1.0,
        (Wind, Fire) => 0.75,
        (Wind, Inw) => 0.75,

        // Fire
        (Fire, Water) => 0.75,
        (Fire, Earth) => 1.0,
        (Fire, Wind) => 1.5,
        (Fire, Fire) => 1.0,
        (Fire, Inw) => 0.75,

        // Inw
        (Inw, Water) => 1.5,
        (Inw, Earth) => 1.5,
        (Inw, Wind) => 1.5,
        (Inw, Fire) => 1.5,
        (Inw, Inw) => 1.0,

        // Neutral
        _ => 1.0,
    }
}

pub fn calculate_combat_damage(
    attacker: &CombatStats,
    defender: &CombatStats,
    rng: &mut impl rand::Rng,
) -> (i32, bool) {
    let damage_after_defense = attacker.attack * 100.0 / (100.0 + defender.defense.max(0.0));
    let is_critical = rng.random::<f32>() < attacker.critical_rate.clamp(0.0, 1.0);
    let critical_multiplier = if is_critical { attacker.critical_damage.max(1.0) } 
    else { 1.0 };
    let final_damage = damage_after_defense * critical_multiplier;
    let damage = final_damage.round().max(1.0) as i32;
    (damage, is_critical)
}
pub fn combat_stats_from_element_exp(
    base: &BaseStats,
    mastery: &ElementMastery,
) -> CombatStats {
    let water = mastery.water.exp as f32 / 10.0;
    let fire = mastery.fire.exp as f32 / 10.0;
    let wind = mastery.wind.exp as f32 / 10.0;
    let earth = mastery.earth.exp as f32 / 10.0;
    let inw = mastery.inw.exp as f32 / 10.0;

    let hp_bonus = earth * 8.0 + inw * 2.0;
    let mp_bonus = water * 10.0 + inw * 2.0;
    let attack_bonus = water * 0.2 + fire * 0.8 + inw * 0.15;
    let defense_bonus = earth * 0.6 + inw * 0.15;
    let critical_rate_bonus = wind * 0.005 + inw * 0.001;
    let critical_damage_bonus = fire * 0.02 + wind * 0.01 + inw * 0.005;

    CombatStats {
        max_hp: base.max_hp + hp_bonus,
        max_mp: base.max_mp + mp_bonus,
        attack: base.attack + attack_bonus,
        defense: base.defense + defense_bonus,
        critical_rate: base.critical_rate + critical_rate_bonus,
        critical_damage: base.critical_damage + critical_damage_bonus,
    }
}

/// "ค่ากลาง" (Neutral Pool) — จุดที่ได้จากการกด "-" ลด EXP ธาตุ
/// เก็บไว้ใช้กด "+" เพิ่ม EXP ธาตุอื่นต่อไป
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ElementPointPool {
    pub points: u32,
}

/// รายชื่อธาตุทั้งหมด ใช้วนลูปสร้าง UI
pub const ALL_ELEMENTS: [Element; 5] = [
    Element::Water,
    Element::Fire,
    Element::Wind,
    Element::Earth,
    Element::Inw,
];

impl ElementMastery {
    /// อ่านค่า EXP ธาตุแบบ immutable (ไว้โชว์ UI)
    pub fn get(&self, element: Element) -> Option<&ElementProgress> {
        match element {
            Element::Water => Some(&self.water),
            Element::Fire => Some(&self.fire),
            Element::Wind => Some(&self.wind),
            Element::Earth => Some(&self.earth),
            Element::Inw => Some(&self.inw),
            Element::Neutral => None,
        }
    }
}

/// ปุ่ม "-" : ลด EXP ธาตุ 1 หน่วย -> ค่ากลาง +1
pub fn decrease_element_to_pool(
    mastery: &mut ElementMastery,
    pool: &mut ElementPointPool,
    element: Element,
) -> bool {
    let Some(progress) = mastery.get_mut(element) else {
        return false;
    };
    if progress.exp == 0 {
        return false;
    }
    progress.exp -= 1;
    pool.points += 1;
    true
}

/// ปุ่ม "+" : ใช้ค่ากลาง 1 หน่วย -> EXP ธาตุ +1
pub fn increase_element_from_pool(
    mastery: &mut ElementMastery,
    pool: &mut ElementPointPool,
    element: Element,
) -> bool {
    if pool.points == 0 {
        return false;
    }
    let Some(progress) = mastery.get_mut(element) else {
        return false;
    };
    pool.points -= 1;
    progress.exp += 1;
    true
}