// element_ui.rs
use bevy::prelude::*;
use crate::components::{
    Health, Mana, Player, GuardianDialogUI,
    AtkAndDefElement, BaseStats, CombatStats, ElementMastery,
};
use crate::ui_cpn::{
    GameFonts, Localization, Language, PreviousLanguage,
    setup_localization, load_fonts,
};

pub struct ElementUiPlugin;

impl Plugin for ElementUiPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, (
            load_fonts,
            setup_localization,
        ))
        .init_resource::<PreviousLanguage>()
        .add_systems(Update, (
            element_status_input,
            update_element_status_ui,
            game_controls_input,
            language_switch_input,
            update_ui_on_language_change,
        ));
    }
}

#[derive(Component)]
struct ElementStatusUi;

#[derive(Component)]
struct ControlsUiRoot;

#[derive(Component, Clone, Copy)]
enum PlayerStatusValueText { Hp, Mp, Attack, Defense, CriticalRate, CriticalDamage, AtkAndDefElement }

#[derive(Component, Clone, Copy)]
enum PlayerStatusBonusText { Hp, Mp, Attack, Defense, CriticalRate, CriticalDamage }

#[derive(Component, Clone, Copy)]
enum ElementExpText { Water, Fire, Wind, Earth, Inw }

// 1. เพิ่ม loc: &Localization
fn spawn_element_status_ui(commands: &mut Commands, fonts: &GameFonts, loc: &Localization) {
    commands
        .spawn((
            ElementStatusUi,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(600.0),
                top: Val::Px(25.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(20.0)),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(loc.get("player_status_title")), // ใช้ loc.get
                TextFont { font: fonts.abc3dz.clone(), font_size: 28.0, ..default() },
                TextColor(Color::srgb(1.0, 0.82, 0.20)),
                Node {
                    width: Val::Percent(100.0),
                    margin: UiRect { bottom: Val::Px(18.0), ..default() },
                    ..default()
                },
            ));

            let status_rows = [
                (loc.get("hp_label"), PlayerStatusValueText::Hp, Some(PlayerStatusBonusText::Hp)),
                (loc.get("mp_label"), PlayerStatusValueText::Mp, Some(PlayerStatusBonusText::Mp)),
                (loc.get("atk_label"), PlayerStatusValueText::Attack, Some(PlayerStatusBonusText::Attack)),
                (loc.get("def_label"), PlayerStatusValueText::Defense, Some(PlayerStatusBonusText::Defense)),
                (loc.get("crit_rate_label"), PlayerStatusValueText::CriticalRate, Some(PlayerStatusBonusText::CriticalRate)),
                (loc.get("crit_dmg_label"), PlayerStatusValueText::CriticalDamage, Some(PlayerStatusBonusText::CriticalDamage)),
                (loc.get("element_label"), PlayerStatusValueText::AtkAndDefElement, None),
            ];

            for (label, value_marker, bonus_marker) in status_rows {
                panel
                    .spawn((
                        Text::new(label),
                        TextFont { font: fonts.abc3dz.clone(), font_size: 23.0, ..default() },
                        TextColor(Color::WHITE),
                    ))
                    .with_children(|text| {
                        text.spawn((
                            TextSpan::default(),
                            TextFont { font: fonts.abc3dz.clone(), font_size: 23.0, ..default() },
                            TextColor(Color::WHITE),
                            value_marker,
                        ));
                        if let Some(bonus_marker) = bonus_marker {
                            text.spawn((
                                TextSpan::default(),
                                TextFont { font: fonts.abc3dz.clone(), font_size: 23.0, ..default() },
                                TextColor(Color::srgb(0.25, 1.0, 0.35)),
                                bonus_marker,
                            ));
                        }
                    });
            }

            panel.spawn((
                Text::new(loc.get("element_exp_title")), // ใช้ loc.get
                TextFont { font: fonts.abc3dz.clone(), font_size: 28.0, ..default() },
                TextColor(Color::srgb(1.0, 0.82, 0.20)),
                Node {
                    width: Val::Percent(100.0),
                    margin: UiRect { top: Val::Px(22.0), bottom: Val::Px(8.0), ..default() },
                    ..default()
                },
            ));

            let element_rows = [
                (loc.get("water_label"), ElementExpText::Water),
                (loc.get("fire_label"), ElementExpText::Fire),
                (loc.get("wind_label"), ElementExpText::Wind),
                (loc.get("earth_label"), ElementExpText::Earth),
                (loc.get("inw_label"), ElementExpText::Inw),
            ];

            for (label, marker) in element_rows {
                panel
                    .spawn((
                        Text::new(label),
                        TextFont { font: fonts.abc3dz.clone(), font_size: 23.0, ..default() },
                        TextColor(Color::WHITE),
                    ))
                    .with_child((
                        TextSpan::default(),
                        TextFont { font: fonts.abc3dz.clone(), font_size: 23.0, ..default() },
                        TextColor(Color::WHITE),
                        marker,
                    ));
            }
        });
}

fn element_status_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    dialog_query: Query<(), With<GuardianDialogUI>>,
    ui_query: Query<Entity, With<ElementStatusUi>>,
    fonts: Res<GameFonts>,
    loc: Res<Localization>, // เพิ่ม Res<Localization>
) {
    if !dialog_query.is_empty() { return; }

    let keyboard_pressed = keyboard.just_pressed(KeyCode::KeyU);
    let gamepad_pressed = gamepads.iter().any(|gamepad| gamepad.just_pressed(GamepadButton::RightTrigger));

    if !keyboard_pressed && !gamepad_pressed { return; }

    if let Ok(entity) = ui_query.single() {
        commands.entity(entity).despawn();
    } else {
        spawn_element_status_ui(&mut commands, &fonts, &loc); // ส่ง loc เข้าไป
    }
}

fn update_element_status_ui(
    player_query: Query<(&Health, &Mana, &BaseStats, &CombatStats, &ElementMastery, &AtkAndDefElement), With<Player>>,
    ui_query: Query<&Node, With<ElementStatusUi>>,
    mut value_query: Query<(&PlayerStatusValueText, &mut TextSpan), (Without<PlayerStatusBonusText>, Without<ElementExpText>)>,
    mut bonus_query: Query<(&PlayerStatusBonusText, &mut TextSpan), (Without<PlayerStatusValueText>, Without<ElementExpText>)>,
    mut exp_query: Query<(&ElementExpText, &mut TextSpan), (Without<PlayerStatusValueText>, Without<PlayerStatusBonusText>)>,
) {
    let Ok(ui_node) = ui_query.single() else { return };
    if matches!(ui_node.display, Display::None) { return; }

    let Ok((health, mana, base, combat, mastery, atk_and_def_element)) = player_query.single() else { return };

    let hp_bonus = combat.max_hp - base.max_hp;
    let mp_bonus = combat.max_mp - base.max_mp;
    let attack_bonus = combat.attack - base.attack;
    let defense_bonus = combat.defense - base.defense;
    let critical_rate_bonus = (combat.critical_rate - base.critical_rate) * 100.0;
    let critical_damage_bonus = (combat.critical_damage - base.critical_damage) * 100.0;

    for (kind, mut span) in &mut value_query {
        // แก้จาก span.0 ใน Bevy 0.15 TextSpan คือ struct TextSpan(pub String)
        span.0 = match kind {
            PlayerStatusValueText::Hp => format!("{} / {:.0}", health.current, combat.max_hp),
            PlayerStatusValueText::Mp => format!("{} / {:.0}", mana.current, combat.max_mp),
            PlayerStatusValueText::Attack => format!("{:.1}", combat.attack),
            PlayerStatusValueText::Defense => format!("{:.1}", combat.defense),
            PlayerStatusValueText::CriticalRate => format!("{:.1}%", combat.critical_rate * 100.0),
            PlayerStatusValueText::CriticalDamage => format!("{:.1}%", combat.critical_damage * 100.0),
            PlayerStatusValueText::AtkAndDefElement => format!("{:?}", atk_and_def_element.0),
        };
    }

    for (kind, mut span) in &mut bonus_query {
        // แก้ **span เป็น span.0
        span.0 = match kind {
            PlayerStatusBonusText::Hp => format!("  (+{:.0})", hp_bonus),
            PlayerStatusBonusText::Mp => format!("  (+{:.0})", mp_bonus),
            PlayerStatusBonusText::Attack => format!("  (+{:.1})", attack_bonus),
            PlayerStatusBonusText::Defense => format!("  (+{:.1})", defense_bonus),
            PlayerStatusBonusText::CriticalRate => format!("  (+{:.1}%)", critical_rate_bonus),
            PlayerStatusBonusText::CriticalDamage => format!("  (+{:.1}%)", critical_damage_bonus),
        };
    }

    for (element, mut span) in &mut exp_query {
        span.0 = match element {
            ElementExpText::Water => mastery.water.exp.to_string(),
            ElementExpText::Fire => mastery.fire.exp.to_string(),
            ElementExpText::Wind => mastery.wind.exp.to_string(),
            ElementExpText::Earth => mastery.earth.exp.to_string(),
            ElementExpText::Inw => mastery.inw.exp.to_string(),
        };
    }
}

// 2. เพิ่ม loc: &Localization
fn spawn_controls_ui(commands: &mut Commands, fonts: &GameFonts, loc: &Localization) {
    commands
        .spawn((
            ControlsUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(500.0),
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.10, 0.14, 0.90)),
        ))
        .with_children(|controls| {
            controls.spawn((
                Text::new(loc.get("controls_title")),
                TextFont { font: fonts.abc3dz.clone(), font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
            ));

            let control_rows = [
                loc.get("move_label"), "W A S D / D-Pad", "",
                loc.get("jump_label"), "K / A Button", "",
                loc.get("attack_label"), "J / X Button", "",
                loc.get("dash_label"), "L / B Button", "",
                loc.get("power_label"), "I / Y Button", "",
                loc.get("status_label"), "U / RB", "",
                loc.get("controls_label"), "O / RT", "",
            ];

            for control in control_rows {
                controls.spawn((
                    Text::new(control),
                    TextFont { font: fonts.abc3dz.clone(), font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.85, 0.85, 0.90)),
                ));
            }
        });
}

fn game_controls_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    dialog_query: Query<(), With<GuardianDialogUI>>,
    controls_ui_query: Query<Entity, With<ControlsUiRoot>>,
    fonts: Res<GameFonts>,
    loc: Res<Localization>, // เพิ่ม Res<Localization>
) {
    if !dialog_query.is_empty() { return; }

    let keyboard_pressed = keyboard.just_pressed(KeyCode::KeyO);
    let gamepad_pressed = gamepads.iter().any(|gamepad| gamepad.just_pressed(GamepadButton::RightTrigger2));

    if !keyboard_pressed && !gamepad_pressed { return; }

    if let Ok(entity) = controls_ui_query.single() {
        commands.entity(entity).despawn();
    } else {
        spawn_controls_ui(&mut commands, &fonts, &loc); // ส่ง loc เข้าไป
    }
}

// ระบบสลับภาษา
fn language_switch_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut loc: ResMut<Localization>,
) {
    let kb_pressed = keyboard.just_pressed(KeyCode::KeyE);
    // Bevy 0.15 ใช้ FaceEast แทน East (ปุ่ม B / Circle)
    let gp_pressed = gamepads.iter().any(|g| g.just_pressed(GamepadButton::RightTrigger)); 

    if kb_pressed || gp_pressed {
        loc.current = match loc.current {
            Language::English => Language::Thai,
            Language::Thai => Language::NorthernThai,
            Language::NorthernThai => Language::English,
        };
    }
}

// ระบบตรวจสอบการเปลี่ยนภาษา
fn update_ui_on_language_change(
    loc: Res<Localization>,
    mut prev_lang: ResMut<PreviousLanguage>,
    mut commands: Commands,
    ui_query: Query<Entity, With<ElementStatusUi>>,
    controls_query: Query<Entity, With<ControlsUiRoot>>,
    guardian_query: Query<Entity, With<GuardianDialogUI>>,
    fonts: Res<GameFonts>,
) {
    if loc.current != prev_lang.0 {
        if let Ok(entity) = ui_query.single() {
            commands.entity(entity).despawn();
            spawn_element_status_ui(&mut commands, &fonts, &loc);
        }
        if let Ok(entity) = controls_query.single() {
            commands.entity(entity).despawn();
            spawn_controls_ui(&mut commands, &fonts, &loc);
        }
        if let Ok(entity) = guardian_query.single() {
            commands.entity(entity).despawn();
            // ไม่ต้อง spawn ใหม่ตรงๆ เพราะระบบ show_guardian_dialog จะทำงานในเฟรมถัดไป
            // และมันจะ spawn ใหม่ให้เองโดยอัตโนมัติ (เพราะ dialog_query.is_empty() จะเป็น true)
        }
        prev_lang.0 = loc.current;
    }
}