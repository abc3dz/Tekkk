use bevy::prelude::*;
use chrono::Local;

use crate::components::*;
use crate::save_load::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuScreen {
    Main,
    SaveSlots,
    LoadSlots,
}

#[derive(Resource)]
pub struct PauseMenuState {
    pub screen: PauseMenuScreen,
    pub selected: PauseMenuItem,
    pub selected_slot: usize,
}

#[derive(Component)]
struct PauseMenuScreenUI;

impl Default for PauseMenuState {
    fn default() -> Self {
        Self {
            screen: PauseMenuScreen::Main,
            selected: PauseMenuItem::Save,
            selected_slot: 0,
        }
    }
}

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
            .add_systems(Update,toggle_pause,)
            .add_systems(OnEnter(GameMode::Paused),setup_pause_menu)
            .add_systems(Update,(
                    pause_menu_input,
                    refresh_pause_menu,
                    update_pause_menu_selection,
                ).run_if(in_state(GameMode::Paused)))
            // ลบ UI ตอน Resume
            .add_systems(OnExit(GameMode::Paused),cleanup_pause_menu,);
    }
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    state: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    let keyboard_pressed = keyboard.just_pressed(KeyCode::Escape);
    let gamepad_pressed = gamepads.iter().any(|gamepad| {gamepad.just_pressed(GamepadButton::Start)});

    if !keyboard_pressed && !gamepad_pressed {
        return;
    }

    match state.get() {
        GameMode::Playing => {next_state.set(GameMode::Paused);}
        GameMode::Paused => {next_state.set(GameMode::Playing);}
    }
}

fn setup_pause_menu(
    mut commands: Commands,
    fonts: Res<GameFonts>,
    menu_state: Res<PauseMenuState>,
) {
    spawn_pause_ui(
        &mut commands,
        &fonts,
        &menu_state,
    );
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

fn spawn_pause_ui(
    commands: &mut Commands,
    fonts: &GameFonts,
    menu_state: &PauseMenuState,
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
                PauseMenuScreenUI,

                Node {
                    width: Val::Px(500.0),
                    padding: UiRect::all(
                        Val::Px(35.0)
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

                match menu_state.screen {

                    PauseMenuScreen::Main => {
                        spawn_main_menu(
                            menu,
                            fonts,
                        );
                    }

                    PauseMenuScreen::SaveSlots => {
                        spawn_slot_menu(
                            menu,
                            fonts,
                            "SAVE",
                            menu_state.selected_slot,
                        );
                    }

                    PauseMenuScreen::LoadSlots => {
                        spawn_slot_menu(
                            menu,
                            fonts,
                            "LOAD",
                            menu_state.selected_slot,
                        );
                    }
                }
            });
        });
}

fn spawn_main_menu(
    menu: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
) {
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
            )
        ),
    ));

    spawn_pause_item(
        menu,
        fonts,
        "Save",
        PauseMenuItem::Save,
    );

    spawn_pause_item(
        menu,
        fonts,
        "Load",
        PauseMenuItem::Load,
    );

    spawn_pause_item(
        menu,
        fonts,
        "Resume",
        PauseMenuItem::Resume,
    );
}

fn spawn_slot_menu(
    menu: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    title: &str,
    selected_slot: usize,
) {
    menu.spawn((
        Text::new(title),
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
            )
        ),
    ));

    for slot_index in 0..3 {
        let slot = slot_index + 1;

        let selected =
            slot_index == selected_slot;

        let prefix = if selected {
            "> "
        } else {
            "  "
        };

        match read_save_slot(slot) {

            // =========================
            // มี Save
            // =========================
            Some(data) => {

                menu.spawn((
                    Text::new(format!("{}Slot {}",prefix,slot)),
                    TextFont {font: fonts.abc3dz.clone(),font_size: 28.0,..default()},
                    TextColor(
                        if selected {
                            Color::srgb(1.0,0.82,0.20,)
                        } else {
                            Color::WHITE
                        }
                    ),
                ));
                let date_time = if data.saved_at.is_empty() {
                    "Unknown".to_string()
                } else {
                    data.saved_at.clone()
                };
                menu.spawn((
                    Text::new(format!("Date/Time: {}",date_time)),
                    TextFont {font: fonts.abc3dz.clone(),font_size: 19.0,..default()},
                    TextColor(Color::srgb(0.70,0.70,0.75,)),
                ));

                // menu.spawn((
                //     Text::new(
                //         format!(
                //             "    MP: {}",
                //             data.mp
                //         )
                //     ),
                //     TextFont {
                //         font: fonts.abc3dz.clone(),
                //         font_size: 19.0,
                //         ..default()
                //     },
                //     TextColor(
                //         Color::srgb(
                //             0.70,
                //             0.70,
                //             0.75,
                //         )
                //     ),
                // ));
            }

            // =========================
            // Slot ว่าง
            // =========================
            None => {

                menu.spawn((
                    Text::new(
                        format!(
                            "{}Slot {}    [ Empty ]",
                            prefix,
                            slot
                        )
                    ),
                    TextFont {
                        font: fonts.abc3dz.clone(),
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(
                        if selected {
                            Color::srgb(
                                1.0,
                                0.82,
                                0.20,
                            )
                        } else {
                            Color::srgb(
                                0.55,
                                0.55,
                                0.60,
                            )
                        }
                    ),
                ));
            }
        }
    }

    menu.spawn((
        Text::new(
            "↑ ↓ Select    Enter Save    Esc Back"
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
            )
        ),
        Node {
            margin: UiRect::top(
                Val::Px(20.0)
            ),
            ..default()
        },
    ));
}

fn refresh_pause_menu(
    mut commands: Commands,
    fonts: Res<GameFonts>,
    menu_state: Res<PauseMenuState>,
    query: Query<Entity, With<PauseMenuUI>>,
) {
    if !menu_state.is_changed() {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }

    spawn_pause_ui(
        &mut commands,
        &fonts,
        &menu_state,
    );
}

fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<PauseMenuState>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut save_request: ResMut<SaveRequest>,

    mut load_request: ResMut<LoadRequest>,
) {
    match menu_state.screen {
        // ==========================================
        // MAIN PAUSE MENU
        // ==========================================
        PauseMenuScreen::Main => {

            if keyboard.just_pressed(KeyCode::ArrowUp) {
                menu_state.selected =
                    match menu_state.selected {
                        PauseMenuItem::Save => {
                            PauseMenuItem::Resume
                        }

                        PauseMenuItem::Load => {
                            PauseMenuItem::Save
                        }

                        PauseMenuItem::Resume => {
                            PauseMenuItem::Load
                        }
                    };
            }

            if keyboard.just_pressed(KeyCode::ArrowDown) {
                menu_state.selected =
                    match menu_state.selected {
                        PauseMenuItem::Save => {
                            PauseMenuItem::Load
                        }

                        PauseMenuItem::Load => {
                            PauseMenuItem::Resume
                        }

                        PauseMenuItem::Resume => {
                            PauseMenuItem::Save
                        }
                    };
            }

            // Enter
            if keyboard.just_pressed(KeyCode::Enter) {

                match menu_state.selected {

                    PauseMenuItem::Save => {
                        menu_state.screen =
                            PauseMenuScreen::SaveSlots;

                        menu_state.selected_slot = 0;
                    }

                    PauseMenuItem::Load => {
                        menu_state.screen =
                            PauseMenuScreen::LoadSlots;

                        menu_state.selected_slot = 0;
                    }

                    PauseMenuItem::Resume => {
                        next_state.set(
                            GameMode::Playing
                        );
                    }
                }
            }
        }

        // ==========================================
        // SAVE SLOTS
        // ==========================================
        PauseMenuScreen::SaveSlots => {

            // ↑
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                if menu_state.selected_slot > 0 {
                    menu_state.selected_slot -= 1;
                }
            }

            // ↓
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                if menu_state.selected_slot < 2 {
                    menu_state.selected_slot += 1;
                }
            }

            // ESC = กลับ Main Menu
            if keyboard.just_pressed(KeyCode::Escape) {
                menu_state.screen =
                    PauseMenuScreen::Main;
            }

            // ENTER = SAVE
            if keyboard.just_pressed(KeyCode::Enter) {

                let slot =
                    menu_state.selected_slot + 1;

                save_request.slot = Some(slot);

                println!(
                    "Save requested: Slot {}",
                    slot
                );

                menu_state.screen =
                    PauseMenuScreen::Main;
            }
        }

        // ==========================================
        // LOAD SLOTS
        // ==========================================
        PauseMenuScreen::LoadSlots => {

            if keyboard.just_pressed(KeyCode::ArrowUp) {
                if menu_state.selected_slot > 0 {
                    menu_state.selected_slot -= 1;
                }
            }

            if keyboard.just_pressed(KeyCode::ArrowDown) {
                if menu_state.selected_slot < 2 {
                    menu_state.selected_slot += 1;
                }
            }

            if keyboard.just_pressed(KeyCode::Escape) {
                menu_state.screen =
                    PauseMenuScreen::Main;
            }

            if keyboard.just_pressed(KeyCode::Enter) {
                let slot = menu_state.selected_slot + 1;

                    load_request.slot = Some(slot);

                    println!(
                        "Load requested: Slot {}",
                        slot
                    );

                    menu_state.screen =
                        PauseMenuScreen::Main;
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