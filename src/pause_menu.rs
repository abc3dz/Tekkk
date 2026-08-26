use bevy::prelude::*;

use crate::components::*;
use crate::save_load::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuScreen {
    Main,
    SaveSlots,
    LoadSlots,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum PauseButton {
    Save,
    Load,
    Resume,
    SaveSlot(usize),
    LoadSlot(usize),
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

const COLOR_BUTTON_SELECTED: Color =
    Color::srgb(0.25, 0.25, 0.30);

const COLOR_BUTTON_NORMAL: Color =
    Color::srgb(0.10, 0.10, 0.13);

const COLOR_TEXT: Color =
    Color::srgb(1.0, 1.0, 1.0);

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameMode>()
            .init_resource::<PauseMenuState>()
            .add_systems(Update, toggle_pause)
            .add_systems(OnEnter(GameMode::Paused), setup_pause_menu)
            .add_systems(
                Update,
                (
                    pause_menu_input,
                    pause_button_system,
                    refresh_pause_menu,
                )
                    .run_if(in_state(GameMode::Paused)),
            )
            .add_systems(OnExit(GameMode::Paused), cleanup_pause_menu);
    }
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    state: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    let keyboard_pressed = keyboard.just_pressed(KeyCode::Escape);
    let gamepad_pressed =
        gamepads.iter().any(|gamepad| gamepad.just_pressed(GamepadButton::Start));

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
    menu_state: Res<PauseMenuState>,
) {
    spawn_pause_ui(&mut commands, &fonts, &menu_state);
}

fn spawn_pause_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    label: &str,
    button_type: PauseButton,
    selected: bool,
) {
    let background = if selected {
        Color::srgb(0.25, 0.25, 0.30)
    } else {
        Color::srgb(0.10, 0.10, 0.13)
    };

    parent
        .spawn((
            Button,
            button_type,
            Node {
                width: Val::Px(400.0),
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(background),
            
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font: fonts.abc3dz.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        ))
        .with_children(|root| {
            root.spawn((
                PauseMenuScreenUI,
                Node {
                    width: Val::Px(500.0),
                    padding: UiRect::all(Val::Px(35.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(18.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.96)),
            ))
            .with_children(|menu| match menu_state.screen {
                PauseMenuScreen::Main => {
                    spawn_main_menu(menu, fonts, menu_state.selected);
                }
                PauseMenuScreen::SaveSlots => {
                    spawn_save_slots(menu, fonts, menu_state.selected_slot);
                }
                PauseMenuScreen::LoadSlots => {
                    spawn_load_slots(menu, fonts, menu_state.selected_slot);
                }
            });
        });
}

fn spawn_main_menu(
    menu: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    selected: PauseMenuItem,
) {
    menu.spawn((
        Text::new("PAUSED"),
        TextFont {
            font: fonts.abc3dz.clone(),
            font_size: 42.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.82, 0.20)),
    ));

    spawn_pause_button(
        menu,
        fonts,
        "SAVE",
        PauseButton::Save,
        selected == PauseMenuItem::Save,
    );

    spawn_pause_button(
        menu,
        fonts,
        "LOAD",
        PauseButton::Load,
        selected == PauseMenuItem::Load,
    );

    spawn_pause_button(
        menu,
        fonts,
        "RESUME",
        PauseButton::Resume,
        selected == PauseMenuItem::Resume,
    );
}

fn spawn_save_slot_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    slot: usize,
    selected: bool,
    data: Option<&SaveData>,
) {
    let background = if selected {
        Color::srgb(0.25, 0.25, 0.30)
    } else {
        Color::srgb(0.10, 0.10, 0.13)
    };

    parent
        .spawn((
            Button,
            PauseButton::SaveSlot(slot),
            Node {
                width: Val::Px(450.0),
                min_height: Val::Px(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(4.0),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(background),
        ))
        .with_children(|button| {
            let prefix = if selected { "> " } else { "" };

            button.spawn((
                Text::new(format!("{}Slot {}", prefix, slot)),
                TextFont {
                    font: fonts.abc3dz.clone(),
                    font_size: 27.0,
                    ..default()
                },
                TextColor(if selected {
                    Color::srgb(1.0, 0.82, 0.20)
                } else {
                    Color::WHITE
                }),
            ));

            match data {
                Some(data) => {
                    let date_time = if data.saved_at.is_empty() {
                        "Unknown".to_string()
                    } else {
                        data.saved_at.clone()
                    };

                    button.spawn((
                        Text::new(format!("Date/Time: {}", date_time)),
                        TextFont {
                            font: fonts.abc3dz.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.70, 0.70, 0.75)),
                    ));

                    button.spawn((
                        Text::new(format!("HP: {}    MP: {}", data.hp, data.mp)),
                        TextFont {
                            font: fonts.abc3dz.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.70, 0.70, 0.75)),
                    ));
                }
                None => {
                    button.spawn((
                        Text::new("EMPTY"),
                        TextFont {
                            font: fonts.abc3dz.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.55, 0.55, 0.60)),
                    ));
                }
            }
        });
}

fn spawn_save_slots(
    menu: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    selected_slot: usize,
) {
    menu.spawn((
        Text::new("SAVE"),
        TextFont {
            font: fonts.abc3dz.clone(),
            font_size: 42.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.82, 0.20)),
    ));

    for index in 0..3 {
        let slot = index + 1;
        let data = read_save_slot(slot);
        spawn_save_slot_button(menu, fonts, slot, index == selected_slot, data.as_ref());
    }
}

fn spawn_load_slot_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    slot: usize,
    selected: bool,
    data: Option<&SaveData>,
) {
    let background = if selected {
        Color::srgb(0.25, 0.25, 0.30)
    } else {
        Color::srgb(0.10, 0.10, 0.13)
    };

    parent
        .spawn((
            Button,
            PauseButton::LoadSlot(slot),
            Node {
                width: Val::Px(450.0),
                min_height: Val::Px(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(4.0),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(background),
        ))
        .with_children(|button| {
            let prefix = if selected { "> " } else { "" };

            button.spawn((
                Text::new(format!("{}Slot {}", prefix, slot)),
                TextFont {
                    font: fonts.abc3dz.clone(),
                    font_size: 27.0,
                    ..default()
                },
                TextColor(if selected {
                    Color::srgb(1.0, 0.82, 0.20)
                } else {
                    Color::WHITE
                }),
            ));

            match data {
                Some(data) => {
                    let date_time = if data.saved_at.is_empty() {
                        "Unknown".to_string()
                    } else {
                        data.saved_at.clone()
                    };

                    button.spawn((
                        Text::new(format!("Date/Time: {}", date_time)),
                        TextFont {
                            font: fonts.abc3dz.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.70, 0.70, 0.75)),
                    ));

                    button.spawn((
                        Text::new(format!("HP: {}    MP: {}", data.hp, data.mp)),
                        TextFont {
                            font: fonts.abc3dz.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.70, 0.70, 0.75)),
                    ));
                }
                None => {
                    button.spawn((
                        Text::new("EMPTY"),
                        TextFont {
                            font: fonts.abc3dz.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.55, 0.55, 0.60)),
                    ));
                }
            }
        });
}

fn spawn_load_slots(
    menu: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    selected_slot: usize,
) {
    menu.spawn((
        Text::new("LOAD"),
        TextFont {
            font: fonts.abc3dz.clone(),
            font_size: 42.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.82, 0.20)),
    ));

    for index in 0..3 {
        let slot = index + 1;
        let data = read_save_slot(slot);
        spawn_load_slot_button(menu, fonts, slot, index == selected_slot, data.as_ref());
    }
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

    spawn_pause_ui(&mut commands, &fonts, &menu_state);
}

// ==========================================
// MOUSE INTERACTION SYSTEM
// ==========================================
fn pause_button_system(
    mut interaction_query: Query<
        (&Interaction, &PauseButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut menu_state: ResMut<PauseMenuState>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut save_request: ResMut<SaveRequest>,
    mut load_request: ResMut<LoadRequest>,
) {
    for (interaction, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match button {
                PauseButton::Save => {
                    menu_state.screen = PauseMenuScreen::SaveSlots;
                    menu_state.selected_slot = 0;
                }
                PauseButton::Load => {
                    menu_state.screen = PauseMenuScreen::LoadSlots;
                    menu_state.selected_slot = 0;
                }
                PauseButton::Resume => {
                    next_state.set(GameMode::Playing);
                }
                PauseButton::SaveSlot(slot) => {
                    save_request.slot = Some(*slot);
                    menu_state.screen = PauseMenuScreen::Main;
                }
                PauseButton::LoadSlot(slot) => {
                    load_request.slot = Some(*slot);
                    menu_state.screen = PauseMenuScreen::Main;
                }
            },
            Interaction::Hovered => match button {
                PauseButton::Save => menu_state.selected = PauseMenuItem::Save,
                PauseButton::Load => menu_state.selected = PauseMenuItem::Load,
                PauseButton::Resume => menu_state.selected = PauseMenuItem::Resume,
                PauseButton::SaveSlot(slot) => menu_state.selected_slot = slot - 1,
                PauseButton::LoadSlot(slot) => menu_state.selected_slot = slot - 1,
            },
            Interaction::None => {}
        }
    }
}

// ==========================================
// KEYBOARD & GAMEPAD INPUT SYSTEM
// ==========================================
fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut menu_state: ResMut<PauseMenuState>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut save_request: ResMut<SaveRequest>,
    mut load_request: ResMut<LoadRequest>,
) {
    let up = keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
        || gamepads.iter().any(|g| g.just_pressed(GamepadButton::DPadUp));

    let down = keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
        || gamepads.iter().any(|g| g.just_pressed(GamepadButton::DPadDown));

    let confirm = keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || gamepads.iter().any(|g| g.just_pressed(GamepadButton::South));

    let cancel = keyboard.just_pressed(KeyCode::Escape)
        || gamepads.iter().any(|g| g.just_pressed(GamepadButton::East));

    match menu_state.screen {
        PauseMenuScreen::Main => {
            if up {
                menu_state.selected = match menu_state.selected {
                    PauseMenuItem::Save => PauseMenuItem::Resume,
                    PauseMenuItem::Load => PauseMenuItem::Save,
                    PauseMenuItem::Resume => PauseMenuItem::Load,
                };
            }

            if down {
                menu_state.selected = match menu_state.selected {
                    PauseMenuItem::Save => PauseMenuItem::Load,
                    PauseMenuItem::Load => PauseMenuItem::Resume,
                    PauseMenuItem::Resume => PauseMenuItem::Save,
                };
            }

            if confirm {
                match menu_state.selected {
                    PauseMenuItem::Save => {
                        menu_state.screen = PauseMenuScreen::SaveSlots;
                        menu_state.selected_slot = 0;
                    }
                    PauseMenuItem::Load => {
                        menu_state.screen = PauseMenuScreen::LoadSlots;
                        menu_state.selected_slot = 0;
                    }
                    PauseMenuItem::Resume => {
                        next_state.set(GameMode::Playing);
                    }
                }
            }
        }

        PauseMenuScreen::SaveSlots => {
            if up && menu_state.selected_slot > 0 {
                menu_state.selected_slot -= 1;
            }

            if down && menu_state.selected_slot < 2 {
                menu_state.selected_slot += 1;
            }

            if cancel {
                menu_state.screen = PauseMenuScreen::Main;
            }

            if confirm {
                let slot = menu_state.selected_slot + 1;
                save_request.slot = Some(slot);
                menu_state.screen = PauseMenuScreen::Main;
            }
        }

        PauseMenuScreen::LoadSlots => {
            if up && menu_state.selected_slot > 0 {
                menu_state.selected_slot -= 1;
            }

            if down && menu_state.selected_slot < 2 {
                menu_state.selected_slot += 1;
            }

            if cancel {
                menu_state.screen = PauseMenuScreen::Main;
            }

            if confirm {
                let slot = menu_state.selected_slot + 1;
                load_request.slot = Some(slot);
                menu_state.screen = PauseMenuScreen::Main;
            }
        }
    }
}

fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}