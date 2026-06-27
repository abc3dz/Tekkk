use bevy::window::PresentMode;
use bevy::prelude::*;
use avian3d::prelude::*;

mod player;
mod camera;
mod world;
mod components;
mod fps;

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