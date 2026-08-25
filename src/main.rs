use bevy::window::PresentMode;
use bevy::prelude::*;
use avian3d::prelude::*;

mod player;
mod camera;
mod world;
mod components;
use crate::components::*;
//mod combat;
//mod fps;
mod biomes;
mod npc;
mod element_drop;
mod element_ui;
mod enemy;
mod warp_portal;
mod cel_shader;
mod pause_menu;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Tekkk".into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            //PhysicsDebugPlugin,
            cel_shader::CelShaderPlugin,
        ))
        .add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            camera::CameraPlugin,
            //fps::FpsPlugin,
            element_ui::ElementUiPlugin,
            element_drop::ElementDropPlugin,
            pause_menu::PauseMenuPlugin
        ))
        .add_systems(PreStartup, load_fonts)
        .run();
}
