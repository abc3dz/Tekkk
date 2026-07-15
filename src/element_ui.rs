use bevy::prelude::*;

use crate::combat::{
    AttackElement,
    CombatStats,
    DefenseElement,
    ElementMastery,
};

use crate::components::{
    Health,
    Mana,
    Player,
};

pub struct ElementUiPlugin;

impl Plugin for ElementUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_element_status_ui,
        )
        .add_systems(
            Update,
            (
                toggle_element_status_ui,
                update_element_status_when_changed,
            )
                .chain(),
        );
    }
}

// Marker ของ UI หลัก
#[derive(Component)]
struct ElementStatusUi;

// Marker ของข้อความภายใน UI
#[derive(Component)]
struct ElementStatusText;

fn setup_element_status_ui(
    mut commands: Commands,
) {
    commands
        .spawn((
            ElementStatusUi,

            // ครอบทั้งหน้าจอ
            Node {
                position_type: PositionType::Absolute,

                width: Val::Percent(100.0),
                height: Val::Percent(100.0),

                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,

                // ซ่อนไว้ตอนเริ่มเกม
                display: Display::None,

                ..default()
            },

            BackgroundColor(
                Color::srgba(0.0, 0.0, 0.0, 0.55),
            ),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(720.0),
                    min_height: Val::Px(460.0),

                    padding: UiRect::all(
                        Val::Px(24.0),
                    ),

                    ..default()
                },

                BackgroundColor(
                    Color::srgba(
                        0.05,
                        0.05,
                        0.08,
                        0.95,
                    ),
                ),
            ))
            .with_children(|panel| {
                panel.spawn((
                    ElementStatusText,

                    Text::new(
                        "Element status",
                    ),

                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },

                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn build_element_status_text(
    health: &Health,
    mana: &Mana,
    combat: &CombatStats,
    mastery: &ElementMastery,
    attack_element: &AttackElement,
    defense_element: &DefenseElement,
) -> String {
    let water_steps =
        mastery.water.bonus_steps() as f32;

    let fire_steps =
        mastery.fire.bonus_steps() as f32;

    let wind_steps =
        mastery.wind.bonus_steps() as f32;

    let earth_steps =
        mastery.earth.bonus_steps() as f32;

    let inw_steps =
        mastery.inw.bonus_steps() as f32;

    format!(
        "\
PLAYER STATUS

HP: {}/{}        MP: {}/{}

ATK: {:.1}       DEF: {:.1}
Critical Rate: {:.1}%
Critical Damage: {:.1}%

Attack Element: {:?}
Defense Element: {:?}


ELEMENT MASTERY

Water
  Level: {}    EXP: {}
  Bonus: ATK +{:.1}, MP +{:.0}

Fire
  Level: {}    EXP: {}
  Bonus: ATK +{:.1}, Critical Damage +{:.1}%

Wind
  Level: {}    EXP: {}
  Bonus: Critical Rate +{:.1}%, Critical Damage +{:.1}%

Earth
  Level: {}    EXP: {}
  Bonus: DEF +{:.1}, HP +{:.0}

Inw
  Level: {}    EXP: {}
  Bonus: ATK +{:.2}, DEF +{:.2}, Crit Rate +{:.1}%
         Crit Damage +{:.1}%, HP +{:.0}, MP +{:.0}

Press U to close",
        health.current,
        health.max,
        mana.current,
        mana.max,

        combat.attack,
        combat.defense,
        combat.critical_rate * 100.0,
        combat.critical_damage * 100.0,

        attack_element.0,
        defense_element.0,

        // Water
        mastery.water.level,
        mastery.water.exp,
        water_steps * 0.2,
        water_steps * 10.0,

        // Fire
        mastery.fire.level,
        mastery.fire.exp,
        fire_steps * 0.8,
        fire_steps * 0.02 * 100.0,

        // Wind
        mastery.wind.level,
        mastery.wind.exp,
        wind_steps * 0.005 * 100.0,
        wind_steps * 0.01 * 100.0,

        // Earth
        mastery.earth.level,
        mastery.earth.exp,
        earth_steps * 0.6,
        earth_steps * 8.0,

        // Inw
        mastery.inw.level,
        mastery.inw.exp,
        inw_steps * 0.15,
        inw_steps * 0.15,
        inw_steps * 0.001 * 100.0,
        inw_steps * 0.005 * 100.0,
        inw_steps * 2.0,
        inw_steps * 2.0,
    )
}

fn toggle_element_status_ui(
    keyboard: Res<ButtonInput<KeyCode>>,

    player_query: Query<
        (
            &Health,
            &Mana,
            &CombatStats,
            &ElementMastery,
            &AttackElement,
            &DefenseElement,
        ),
        With<Player>,
    >,

    mut ui_query: Query<
        &mut Node,
        With<ElementStatusUi>,
    >,

    mut text_query: Query<
        &mut Text,
        With<ElementStatusText>,
    >,
) {
    if !keyboard.just_pressed(KeyCode::KeyU) {
        return;
    }

    let Ok(mut ui_node) = ui_query.single_mut()
    else {
        return;
    };

    let opening =
        matches!(ui_node.display, Display::None);

    ui_node.display = if opening {
        Display::Flex
    } else {
        Display::None
    };

    // ปิด UI แล้วไม่ต้องคำนวณข้อความ
    if !opening {
        return;
    }

    let Ok((
        health,
        mana,
        combat,
        mastery,
        attack_element,
        defense_element,
    )) = player_query.single()
    else {
        return;
    };

    let Ok(mut text) = text_query.single_mut()
    else {
        return;
    };

    *text = Text::new(
        build_element_status_text(
            health,
            mana,
            combat,
            mastery,
            attack_element,
            defense_element,
        ),
    );
}

fn update_element_status_when_changed(
    player_query: Query<
        (
            &Health,
            &Mana,
            &CombatStats,
            &ElementMastery,
            &AttackElement,
            &DefenseElement,
        ),
        (
            With<Player>,
            Or<(
                Changed<Health>,
                Changed<Mana>,
                Changed<CombatStats>,
                Changed<ElementMastery>,
                Changed<AttackElement>,
                Changed<DefenseElement>,
            )>,
        ),
    >,

    ui_query: Query<
        &Node,
        With<ElementStatusUi>,
    >,

    mut text_query: Query<
        &mut Text,
        With<ElementStatusText>,
    >,
) {
    let Ok(ui_node) = ui_query.single() else {
        return;
    };

    // ปิดอยู่ ไม่ต้องอัปเดต Text
    if matches!(ui_node.display, Display::None) {
        return;
    }

    let Ok((
        health,
        mana,
        combat,
        mastery,
        attack_element,
        defense_element,
    )) = player_query.single()
    else {
        return;
    };

    let Ok(mut text) = text_query.single_mut()
    else {
        return;
    };

    *text = Text::new(
        build_element_status_text(
            health,
            mana,
            combat,
            mastery,
            attack_element,
            defense_element,
        ),
    );
}