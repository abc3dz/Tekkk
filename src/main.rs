use bevy::prelude::*;
use avian3d::prelude::*;

mod player;
mod camera;
mod world;
mod enemy;
mod combat;
mod components;
mod states;

fn main() {
    App::new()
        
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin,
        ))
        .add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            camera::CameraPlugin,
            // enemy::EnemyPlugin,
            // combat::CombatPlugin,
        ))
        .run();
}