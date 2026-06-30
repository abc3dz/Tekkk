use bevy::window::PresentMode;
use bevy::prelude::*;
use avian3d::prelude::*;
use bevy_wind_waker_shader::prelude::*;
use bevy_edge_detection_outline::EdgeDetectionPlugin;

mod player;
mod camera;
mod world;
mod components;
mod fps;
mod biomes;

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
            PhysicsDebugPlugin,
            WindWakerShaderPlugin::default(),
            EdgeDetectionPlugin::default(),
            
        ))
        .add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            camera::CameraPlugin,
            //fps::FpsPlugin, 
            // enemy::EnemyPlugin,
            // combat::CombatPlugin,
        ))
        .run();
}