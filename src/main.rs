use bevy::window::PresentMode;
use bevy::prelude::*;
use avian3d::prelude::*;
use bevy_wind_waker_shader::prelude::*;

mod player;
mod camera;
mod world;
mod components;
use crate::components::*;
mod combat;
//mod fps;
mod biomes;
mod npc;
mod element_drop;
mod element_ui;
mod enemy;
mod warp_portal;

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
            WindWakerShaderPlugin::default(),
            
        ))
        .add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            camera::CameraPlugin,
            //fps::FpsPlugin, 
            //enemy::EnemyPlugin,
            element_ui::ElementUiPlugin,
            element_drop::ElementDropPlugin,
        ))
        .add_systems(PreStartup, load_fonts)
        .run();
}
