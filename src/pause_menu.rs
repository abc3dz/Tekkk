use bevy::prelude::*;

use crate::components::GameFonts;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameMode {
    #[default]
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuItem {
    Save,
    Load,
    Resume,
}

#[derive(Resource)]
pub struct PauseMenuState {
    pub selected: PauseMenuItem,
}

impl Default for PauseMenuState {
    fn default() -> Self {
        Self {
            selected: PauseMenuItem::Save,
        }
    }
}

#[derive(Component)]
struct PauseMenuUI;

#[derive(Component)]
struct PauseMenuItemText(PauseMenuItem);

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameMode>()
            .init_resource::<PauseMenuState>()

            // ESC เปิด / ปิด Pause
            .add_systems(
                Update,
                toggle_pause,
            )

            // สร้าง UI ตอน Pause
            .add_systems(
                OnEnter(GameMode::Paused),
                setup_pause_menu,
            )

            // Input ของ Pause Menu
            .add_systems(
                Update,
                (
                    pause_menu_input,
                    update_pause_menu_selection,
                )
                    .run_if(in_state(GameMode::Paused)),
            )

            // ลบ UI ตอน Resume
            .add_systems(
                OnExit(GameMode::Paused),
                cleanup_pause_menu,
            );
    }
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    state: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    let keyboard_pressed =
        keyboard.just_pressed(KeyCode::Escape);

    let gamepad_pressed =
        gamepads
            .iter()
            .any(|gamepad| {
                gamepad.just_pressed(GamepadButton::Start)
            });

    if !keyboard_pressed && !gamepad_pressed {
        return;
    }

    match state.get() {
        GameMode::Playing => {
            next_state.set(GameMode::Paused);
        }

        GameMode::Paused => {
            next_state.set(GameMode::Playing);
        }
    }
}

fn setup_pause_menu(
    mut commands: Commands,
    fonts: Res<GameFonts>,
) {
    commands
        .spawn((
            PauseMenuUI,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(
                Color::srgba(
                    0.0,
                    0.0,
                    0.0,
                    0.72,
                ),
            ),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(500.0),
                    padding: UiRect::all(
                        Val::Px(35.0),
                    ),
                    flex_direction:
                        FlexDirection::Column,
                    align_items:
                        AlignItems::Center,
                    row_gap: Val::Px(18.0),
                    ..default()
                },
                BackgroundColor(
                    Color::srgba(
                        0.05,
                        0.05,
                        0.08,
                        0.96,
                    ),
                ),
            ))
            .with_children(|menu| {

                // TITLE
                menu.spawn((
                    Text::new("PAUSED"),
                    TextFont {
                        font: fonts.abc3dz.clone(),
                        font_size: 42.0,
                        ..default()
                    },
                    TextColor(
                        Color::srgb(
                            1.0,
                            0.82,
                            0.20,
                        ),
                    ),
                    Node {
                        margin: UiRect::bottom(
                            Val::Px(20.0),
                        ),
                        ..default()
                    },
                ));

                // SAVE
                spawn_pause_item(
                    menu,
                    &fonts,
                    "Save",
                    PauseMenuItem::Save,
                );

                // LOAD
                spawn_pause_item(
                    menu,
                    &fonts,
                    "Load",
                    PauseMenuItem::Load,
                );

                // RESUME
                spawn_pause_item(
                    menu,
                    &fonts,
                    "Resume",
                    PauseMenuItem::Resume,
                );

                // HELP
                menu.spawn((
                    Text::new(
                        "↑ ↓ Select    Enter Confirm    Esc Resume",
                    ),
                    TextFont {
                        font: fonts.abc3dz.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(
                        Color::srgb(
                            0.75,
                            0.75,
                            0.80,
                        ),
                    ),
                    Node {
                        margin: UiRect::top(
                            Val::Px(20.0),
                        ),
                        ..default()
                    },
                ));
            });
        });
}

fn spawn_pause_item(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    text: &str,
    item: PauseMenuItem,
) {
    parent.spawn((
        PauseMenuItemText(item),
        Text::new(text),
        TextFont {
            font: fonts.abc3dz.clone(),
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            width: Val::Percent(100.0),
            justify_content:
                JustifyContent::Center,
            ..default()
        },
    ));
}

fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<PauseMenuState>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        menu_state.selected =
            match menu_state.selected {
                PauseMenuItem::Save =>
                    PauseMenuItem::Resume,

                PauseMenuItem::Load =>
                    PauseMenuItem::Save,

                PauseMenuItem::Resume =>
                    PauseMenuItem::Load,
            };
    }

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        menu_state.selected =
            match menu_state.selected {
                PauseMenuItem::Save =>
                    PauseMenuItem::Load,

                PauseMenuItem::Load =>
                    PauseMenuItem::Resume,

                PauseMenuItem::Resume =>
                    PauseMenuItem::Save,
            };
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        match menu_state.selected {

            PauseMenuItem::Save => {
                println!("SAVE");
            }

            PauseMenuItem::Load => {
                println!("LOAD");
            }

            PauseMenuItem::Resume => {
                next_state.set(
                    GameMode::Playing
                );
            }
        }
    }
}

fn update_pause_menu_selection(
    menu_state: Res<PauseMenuState>,

    mut query: Query<
        (
            &PauseMenuItemText,
            &mut Text,
            &mut TextColor,
        ),
    >,
) {
    for (
        item,
        mut text,
        mut color,
    ) in &mut query {

        if item.0 == menu_state.selected {

            **text = format!(
                "> {}",
                pause_item_name(item.0)
            );

            color.0 =
                Color::srgb(
                    1.0,
                    0.82,
                    0.20,
                );

        } else {

            **text = format!(
                "  {}",
                pause_item_name(item.0)
            );

            color.0 =
                Color::WHITE;
        }
    }
}

fn pause_item_name(
    item: PauseMenuItem,
) -> &'static str {
    match item {
        PauseMenuItem::Save => "Save",
        PauseMenuItem::Load => "Load",
        PauseMenuItem::Resume => "Resume",
    }
}

fn cleanup_pause_menu(
    mut commands: Commands,
    query: Query<
        Entity,
        With<PauseMenuUI>,
    >,
) {
    for entity in &query {
        commands
            .entity(entity)
            .despawn();
    }
}