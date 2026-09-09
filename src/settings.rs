use bevy::prelude::*;

use crate::components::{MusicAudio, GameFonts};
use crate::pause_menu::{GameMode, PauseMenuScreen, PauseMenuState};

// ==========================================
// SETTINGS
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsItem {
    Resolution,
    DisplayMode,
    MasterVolume,
    Back,
}

#[derive(Resource)]
pub struct SettingsState {
    pub selected: SettingsItem,
    pub resolution_index: usize,
    pub display_mode_index: usize,
    pub master_volume: f32,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            selected: SettingsItem::Resolution,
            resolution_index: 0,
            display_mode_index: 0,
            master_volume: 1.0,
        }
    }
}

// ==========================================
// UI COMPONENT
// ==========================================

#[derive(Component)]
pub struct SettingsUI;

#[derive(Component)]
struct SettingsButton(SettingsItem);

// ==========================================
// VALUES
// ==========================================

const RESOLUTIONS: &[(u32, u32)] = &[
    (1280, 720),
    (1600, 900),
    (1920, 1080),
];

const DISPLAY_MODES: &[&str] = &[
    "Fullscreen",
    "Borderless",
    "Windowed",
];

// ==========================================
// PLUGIN
// ==========================================

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsState>()
            .add_systems(Update,(
                    settings_input,
                    settings_button_system,
                    settings_button_visual_system
                ).after(crate::pause_menu::pause_menu_input).run_if(in_state(GameMode::Paused)),)
            .add_systems(Update,apply_master_volume);
    }
}

// ==========================================
// SPAWN SETTINGS SCREEN
// ==========================================

pub fn spawn_settings_screen(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    settings: &SettingsState,
) {
    parent
        .spawn((
            SettingsUI,
            Node {
                width: Val::Px(600.0),
                padding: UiRect::all(Val::Px(35.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.96)),
        ))
        .with_children(|menu| {
            // ==========================================
            // TITLE
            // ==========================================

            menu.spawn((
                Text::new("SETTINGS"),
                TextFont {
                    font: fonts.abc3dz.clone(),
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.82, 0.20)),
            ));

            // ==========================================
            // RESOLUTION
            // ==========================================

            let (width, height) = RESOLUTIONS[settings.resolution_index];

            spawn_settings_button(
                menu,
                fonts,
                SettingsItem::Resolution,
                &format!("RESOLUTION    {} x {}", width, height),
                settings.selected == SettingsItem::Resolution,
            );

            // ==========================================
            // DISPLAY MODE
            // ==========================================

            spawn_settings_button(
                menu,
                fonts,
                SettingsItem::DisplayMode,
                &format!(
                    "DISPLAY MODE    {}",
                    DISPLAY_MODES[settings.display_mode_index]
                ),
                settings.selected == SettingsItem::DisplayMode,
            );

            // ==========================================
            // MASTER VOLUME
            // ==========================================

            spawn_settings_button(
                menu,
                fonts,
                SettingsItem::MasterVolume,
                &format!(
                    "MASTER VOLUME    {}%",
                    (settings.master_volume * 100.0) as u32
                ),
                settings.selected == SettingsItem::MasterVolume,
            );

            // ==========================================
            // BACK
            // ==========================================

            spawn_settings_button(
                menu,
                fonts,
                SettingsItem::Back,
                "BACK",
                settings.selected == SettingsItem::Back,
            );
        });
}

// ==========================================
// BUTTON
// ==========================================

fn spawn_settings_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    item: SettingsItem,
    label: &str,
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
            SettingsButton(item),
            Node {
                width: Val::Px(520.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(background),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(format!("{}", label)),
                TextFont {
                    font: fonts.abc3dz.clone(),
                    font_size: 25.0,
                    ..default()
                },
                TextColor(if selected {
                    Color::srgb(1.0, 0.82, 0.20)
                } else {
                    Color::WHITE
                }),
            ));
        });
}

// ==========================================
// KEYBOARD / GAMEPAD
// ==========================================

fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut settings: ResMut<SettingsState>,
    mut pause_state: ResMut<PauseMenuState>,
) {
    let up = keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
        || gamepads
            .iter()
            .any(|g| g.just_pressed(GamepadButton::DPadUp));

    let down = keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
        || gamepads
            .iter()
            .any(|g| g.just_pressed(GamepadButton::DPadDown));

    let left = keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
        || gamepads
            .iter()
            .any(|g| g.just_pressed(GamepadButton::DPadLeft));

    let right = keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
        || gamepads
            .iter()
            .any(|g| g.just_pressed(GamepadButton::DPadRight));

    let confirm = keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || gamepads
            .iter()
            .any(|g| g.just_pressed(GamepadButton::South));

    let cancel = keyboard.just_pressed(KeyCode::Escape)
        || gamepads
            .iter()
            .any(|g| g.just_pressed(GamepadButton::East));

    // ==========================================
    // UP
    // ==========================================

    if up {
        settings.selected = match settings.selected {
            SettingsItem::Resolution => SettingsItem::Back,
            SettingsItem::DisplayMode => SettingsItem::Resolution,
            SettingsItem::MasterVolume => SettingsItem::DisplayMode,
            SettingsItem::Back => SettingsItem::MasterVolume,
        };
    }

    // ==========================================
    // DOWN
    // ==========================================

    if down {
        settings.selected = match settings.selected {
            SettingsItem::Resolution => SettingsItem::DisplayMode,
            SettingsItem::DisplayMode => SettingsItem::MasterVolume,
            SettingsItem::MasterVolume => SettingsItem::Back,
            SettingsItem::Back => SettingsItem::Resolution,
        };
    }

    // ==========================================
    // LEFT / RIGHT
    // ==========================================

    match settings.selected {
        SettingsItem::Resolution => {
            if left && settings.resolution_index > 0 {
                settings.resolution_index -= 1;
            }

            if right
                && settings.resolution_index < RESOLUTIONS.len() - 1
            {
                settings.resolution_index += 1;
            }
        }

        SettingsItem::DisplayMode => {
            if left && settings.display_mode_index > 0 {
                settings.display_mode_index -= 1;
            }

            if right
                && settings.display_mode_index < DISPLAY_MODES.len() - 1
            {
                settings.display_mode_index += 1;
            }
        }

        SettingsItem::MasterVolume => {
            if left {
                settings.master_volume =
                    (settings.master_volume - 0.05).max(0.0);
            }

            if right {
                settings.master_volume =
                    (settings.master_volume + 0.05).min(1.0);
            }
        }

        SettingsItem::Back => {}
    }

    // ==========================================
    // CONFIRM
    // ==========================================

    if confirm {
        if settings.selected == SettingsItem::Back {
            settings.selected = SettingsItem::Resolution;
            pause_state.screen = PauseMenuScreen::Main;
        }
    }

    // ==========================================
    // CANCEL
    // ==========================================

    if cancel {
        pause_state.screen = PauseMenuScreen::Main;
        settings.selected = SettingsItem::Resolution; 
    }
}

// ==========================================
// MOUSE
// ==========================================

fn settings_button_system(
    mut interaction_query: Query<
        (&Interaction, &SettingsButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings: ResMut<SettingsState>,
    mut pause_state: ResMut<PauseMenuState>,
) {
    for (interaction, button) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                settings.selected = button.0;
            }

            Interaction::Pressed => {
                match button.0 {
                    SettingsItem::Resolution => { settings.resolution_index = (settings.resolution_index + 1) % RESOLUTIONS.len();}

                    SettingsItem::DisplayMode => { settings.display_mode_index = (settings.display_mode_index + 1) % DISPLAY_MODES.len();}

                    SettingsItem::MasterVolume => { settings.master_volume = (settings.master_volume + 0.05).min(1.0); }

                    SettingsItem::Back => { pause_state.screen = PauseMenuScreen::Main; }
                }
            }

            Interaction::None => {}
        }
    }
}
// ==========================================
// BUTTON VISUAL FEEDBACK (HOVER / PRESS)
// ==========================================
fn settings_button_visual_system(
    mut interaction_query: Query<
        (&Interaction, &SettingsButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    settings: Res<SettingsState>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        // สีพื้นฐานตามสถานะ selected
        let base_color = if settings.selected == button.0 {
            Color::srgb(0.25, 0.25, 0.30)
        } else {
            Color::srgb(0.10, 0.10, 0.13)
        };

        match *interaction {
            Interaction::Pressed => {
                // สีเมื่อคลิกกด
                *color = BackgroundColor(Color::srgb(0.35, 0.35, 0.40));
            }
            Interaction::Hovered => {
                // สีเมื่อเอาเมาส์ไปชี้
                *color = BackgroundColor(Color::srgb(0.20, 0.20, 0.25));
            }
            Interaction::None => {
                // กลับเป็นสีพื้นฐาน
                *color = BackgroundColor(base_color);
            }
        }
    }
}

fn apply_master_volume(
    settings: Res<SettingsState>,
    mut music_query: Query<&mut AudioSink, With<MusicAudio>>,
) {
    if !settings.is_changed() {
        return;
    }

    for mut sink in &mut music_query {
        sink.set_volume(bevy::audio::Volume::Linear(settings.master_volume));
    }
}