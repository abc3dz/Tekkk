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
    GuardianDialogUI
};

pub struct ElementUiPlugin;

impl Plugin for ElementUiPlugin {
    fn build(&self, app: &mut App) {app
        .add_systems(Startup,setup_element_status_ui,)
        .add_systems(Update,(
                pause_and_status_input,
                update_element_status_ui,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
struct ElementStatusUi;

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

fn setup_element_status_ui(
    mut commands: Commands,
) {
    commands.spawn((
            ElementStatusUi,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(650.0),
                    min_height: Val::Px(620.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.95)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("PAUSED"),
                    TextFont {font_size: 38.0, ..default()},
                    TextColor(Color::srgb(1.0,0.82,0.20,),),
                    Node {
                        width: Val::Percent(100.0),
                        margin: UiRect {
                            bottom: Val::Px(18.0),
                            ..default()
                        },
                        ..default()
                    },
                ));
                panel.spawn((
                    Text::new("PLAYER STATUS"),
                    TextFont {font_size: 28.0, ..default()},
                    TextColor(Color::WHITE),
                    Node {margin: UiRect::bottom(Val::Px(12.0)), ..default()},
                ));
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
                    panel.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 23.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ))
                        .with_children(|text| {
                            text.spawn((
                                TextSpan::default(),
                                TextFont { font_size: 23.0, ..default()},
                                TextColor(Color::WHITE),
                                value_marker,
                            ));

                            if let Some(bonus_marker) = bonus_marker {
                                text.spawn((
                                    TextSpan::default(),
                                    TextFont {font_size: 23.0, ..default()},
                                    TextColor(Color::srgb(0.25,1.0,0.35,)),
                                    bonus_marker,
                                ));
                            }
                        });
                }
                panel.spawn((
                    Text::new("ELEMENT EXP"),
                    TextFont {font_size: 28.0, ..default()},
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect {
                            top: Val::Px(22.0),
                            bottom: Val::Px(8.0),
                            ..default()
                        },
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
                    panel.spawn((
                            Text::new(label),
                            TextFont {font_size: 23.0, ..default()},
                            TextColor(Color::WHITE)))
                        .with_child((
                            TextSpan::default(),
                            TextFont {font_size: 23.0, ..default()},
                            TextColor(Color::WHITE),
                            marker,
                        ));
                }

                panel.spawn((
                    Text::new("Esc: Resume"),
                    TextFont {font_size: 21.0, ..default()},
                    TextColor(Color::srgb(0.75,0.75,0.80,)),
                    Node {margin: UiRect::top(Val::Px(18.0)), ..default()},
                ));
            });
        });
}
fn pause_and_status_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    dialog_query: Query<(), With<GuardianDialogUI>>,
    mut ui_query: Query<&mut Node, With<ElementStatusUi>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if !dialog_query.is_empty() {
        return;
    }

    let keyboard_pause_pressed = keyboard.just_pressed(KeyCode::Escape);
    let gamepad_pause_pressed = gamepads.iter().any(|gamepad| {gamepad.just_pressed(GamepadButton::Start)});

    if !keyboard_pause_pressed && !gamepad_pause_pressed{
        return;
    }

    let Ok(mut ui_node) = ui_query.single_mut()
    else { return };

    if virtual_time.is_paused() {
        virtual_time.unpause();
        ui_node.display = Display::None;
        println!("Game resumed");
    } else {
        virtual_time.pause();
        ui_node.display = Display::Flex;
        println!("Game paused");
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