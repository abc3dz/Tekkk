use bevy::prelude::*;

use crate::combat::{
    AtkAndDefElement,
    BaseStats,
    CombatStats,
    ElementMastery,
};

use crate::components::{
    Health,
    Mana,
    Player,
    GuardianDialogUI,
    GameFonts,
};

pub struct ElementUiPlugin;

impl Plugin for ElementUiPlugin {
    fn build(&self, app: &mut App) {app
        //.add_systems(Startup,setup_element_status_ui)
        .add_systems(Update,(
            element_status_input,
            update_element_status_ui,
            game_controls_input
        ));
    }
}

#[derive(Component)]
struct ElementStatusUi;

#[derive(Component)]
struct ControlsUiRoot;

#[derive(Component, Clone, Copy)]
enum PlayerStatusValueText {
    Hp,
    Mp,
    Attack,
    Defense,
    CriticalRate,
    CriticalDamage,
    AtkAndDefElement,
}

#[derive(Component, Clone, Copy)]
enum PlayerStatusBonusText {
    Hp,
    Mp,
    Attack,
    Defense,
    CriticalRate,
    CriticalDamage,
}

#[derive(Component, Clone, Copy)]
enum ElementExpText {
    Water,
    Fire,
    Wind,
    Earth,
    Inw,
}

fn spawn_element_status_ui(commands: &mut Commands, fonts: &GameFonts) {
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
        // .with_children(|root| {
        //     root.spawn((
        //         Node {
        //             width: Val::Px(950.0),
        //             min_height: Val::Px(620.0),
        //             flex_direction: FlexDirection::Column,
        //             row_gap: Val::Px(6.0),
        //             padding: UiRect::all(Val::Px(24.0)),
        //             ..default()
        //         },
        //         BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.95)),
        //     ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("PLAYER STATUS"),
                    TextFont { font: fonts.abc3dz.clone(), font_size: 28.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.82, 0.20)),
                    Node {
                        width: Val::Percent(100.0),
                        margin: UiRect { bottom: Val::Px(18.0), ..default() },
                        ..default()
                    },
                ));
                // panel.spawn((
                //     Text::new("PLAYER STATUS"),
                //     TextFont { font: fonts.abc3dz.clone(), font_size: 28.0, ..default() },
                //     TextColor(Color::WHITE),
                //     Node { width: Val::Percent(100.0), margin: UiRect::bottom(Val::Px(12.0)), ..default() },
                // ));

                let status_rows = [
                    ("HP: ", PlayerStatusValueText::Hp, Some(PlayerStatusBonusText::Hp)),
                    ("MP: ", PlayerStatusValueText::Mp, Some(PlayerStatusBonusText::Mp)),
                    ("ATK: ", PlayerStatusValueText::Attack, Some(PlayerStatusBonusText::Attack)),
                    ("DEF: ", PlayerStatusValueText::Defense, Some(PlayerStatusBonusText::Defense)),
                    ("Critical Rate: ", PlayerStatusValueText::CriticalRate, Some(PlayerStatusBonusText::CriticalRate)),
                    ("Critical Damage: ", PlayerStatusValueText::CriticalDamage, Some(PlayerStatusBonusText::CriticalDamage)),
                    ("Element: ", PlayerStatusValueText::AtkAndDefElement, None),
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
                    Text::new("ELEMENT EXP"),
                    TextFont { font: fonts.abc3dz.clone(), font_size: 28.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.82, 0.20)),
                    Node {
                        width: Val::Percent(100.0),
                        margin: UiRect { top: Val::Px(22.0), bottom: Val::Px(8.0), ..default() },
                        ..default()
                    },
                ));

                let element_rows = [
                    ("Water: ", ElementExpText::Water),
                    ("Fire: ", ElementExpText::Fire),
                    ("Wind: ", ElementExpText::Wind),
                    ("Earth: ", ElementExpText::Earth),
                    ("Inw: ", ElementExpText::Inw),
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
        //});
}

fn element_status_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    dialog_query: Query<(), With<GuardianDialogUI>>,
    ui_query: Query<Entity, With<ElementStatusUi>>,
    fonts: Res<GameFonts>,
) {
    if !dialog_query.is_empty() {
        return;
    }

    let keyboard_pause_pressed = keyboard.just_pressed(KeyCode::KeyU);
    let gamepad_pause_pressed = gamepads.iter().any(|gamepad| {gamepad.just_pressed(GamepadButton::RightTrigger)});

    if !keyboard_pause_pressed && !gamepad_pause_pressed{
        return;
    }

    if let Ok(entity) = ui_query.single() {
        // มีอยู่แล้ว -> despawn
        commands.entity(entity).despawn();
    } else {
        // ยังไม่มี -> spawn
        spawn_element_status_ui(&mut commands, &fonts);
    }
}

fn update_element_status_ui(
    player_query: Query<(&Health, &Mana, &BaseStats, &CombatStats, &ElementMastery, &AtkAndDefElement), With<Player>>,
    ui_query: Query<&Node, With<ElementStatusUi>>,
    mut value_query: Query<(&PlayerStatusValueText, &mut TextSpan),(Without<PlayerStatusBonusText>, Without<ElementExpText>)>,
    mut bonus_query: Query<(&PlayerStatusBonusText, &mut TextSpan),(Without<PlayerStatusValueText>, Without<ElementExpText>)>,
    mut exp_query: Query<(&ElementExpText, &mut TextSpan),(Without<PlayerStatusValueText>, Without<PlayerStatusBonusText>)>,
) {
    let Ok(ui_node) = ui_query.single() else { return };

    if matches!(ui_node.display, Display::None) {
        return;
    }

    let Ok((
        health,
        mana,
        base,
        combat,
        mastery,
        atk_and_def_element,
    )) = player_query.single()
    else { return };

    let hp_bonus = combat.max_hp - base.max_hp;
    let mp_bonus = combat.max_mp - base.max_mp;
    let attack_bonus = combat.attack - base.attack;
    let defense_bonus = combat.defense - base.defense;
    let critical_rate_bonus = (combat.critical_rate - base.critical_rate) * 100.0;
    let critical_damage_bonus = (combat.critical_damage - base.critical_damage) * 100.0;

    for (kind, mut span) in &mut value_query {
        span.0 = match kind {
            PlayerStatusValueText::Hp => {format!("{} / {:.0}", health.current, combat.max_hp)}
            PlayerStatusValueText::Mp => {format!("{} / {:.0}", mana.current, combat.max_mp)}
            PlayerStatusValueText::Attack => {format!("{:.1}", combat.attack)}
            PlayerStatusValueText::Defense => {format!("{:.1}", combat.defense)}
            PlayerStatusValueText::CriticalRate => {format!("{:.1}%", combat.critical_rate * 100.0)}
            PlayerStatusValueText::CriticalDamage => {format!("{:.1}%", combat.critical_damage * 100.0)}
            PlayerStatusValueText::AtkAndDefElement => {format!("{:?}", atk_and_def_element.0)}
        };
    }

    for (kind, mut span) in &mut bonus_query {
        **span = match kind {
            PlayerStatusBonusText::Hp => {format!("  (+{:.0})", hp_bonus)}
            PlayerStatusBonusText::Mp => {format!("  (+{:.0})", mp_bonus)}
            PlayerStatusBonusText::Attack => {format!("  (+{:.1})", attack_bonus)}
            PlayerStatusBonusText::Defense => {format!("  (+{:.1})", defense_bonus)}
            PlayerStatusBonusText::CriticalRate => {format!("  (+{:.1}%)",critical_rate_bonus)}
            PlayerStatusBonusText::CriticalDamage => {format!("  (+{:.1}%)",critical_damage_bonus)}
        };
    }

    for (element, mut span) in &mut exp_query {
        **span = match element {
            ElementExpText::Water => {mastery.water.exp.to_string()}
            ElementExpText::Fire => {mastery.fire.exp.to_string()}
            ElementExpText::Wind => {mastery.wind.exp.to_string()}
            ElementExpText::Earth => {mastery.earth.exp.to_string()}
            ElementExpText::Inw => {mastery.inw.exp.to_string()}
        };
    }
}

fn spawn_controls_ui(commands: &mut Commands, fonts: &GameFonts) {
    commands
        .spawn((
            ControlsUiRoot, // <-- marker ใส่ตรงนี้
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(330.0),
                top: Val::Px(95.0),
                right: Val::Px(24.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.10, 0.14, 0.90)),
        ))
        .with_children(|controls| {
            controls.spawn((
                Text::new("CONTROLS"),
                TextFont {
                    font: fonts.abc3dz.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            let control_rows = [
                "Move", "  W A S D / D-Pad", "",
                "Jump", "  K / Gamepad South", "",
                "Slap", "  J / Gamepad West", "",
                "Dash", "  L / Gamepad East", "",
                "Power", " I / Gamepad North", "",
            ];

            for control in control_rows {
                controls.spawn((
                    Text::new(control),
                    TextFont {
                        font: fonts.abc3dz.clone(),
                        font_size: 20.0,
                        ..default()
                    },
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
) {
    if !dialog_query.is_empty() {
        return;
    }

    let keyboard_pressed = keyboard.just_pressed(KeyCode::KeyO);
    let gamepad_pressed = gamepads
        .iter()
        .any(|gamepad| gamepad.just_pressed(GamepadButton::RightTrigger2));

    if !keyboard_pressed && !gamepad_pressed {
        return;
    }

    if let Ok(entity) = controls_ui_query.single() {
        commands.entity(entity).despawn();
    } else {
        spawn_controls_ui(&mut commands, &fonts);
    }
}