use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, player_movement);
    }
}

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_sever: Res<AssetServer>,
) {
    commands.spawn((
        Player,
        MoveSpeed(6.0),
        Health { current: 100, max: 100 },
        SceneRoot(asset_sever.load("models/PlayerMoya.glb#Scene0")),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 2.0, 0.0),
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity::ZERO,
    ));
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&MoveSpeed, &mut LinearVelocity, &mut Transform), With<Player>>,
) {
    let Ok((speed, mut velocity, mut transform)) = query.single_mut() else {
        return;
    };


    let mut dir = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        dir.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        dir.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }

    if dir.length() > 0.0 {
        dir = dir.normalize();
         transform.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z));
    }

    velocity.x = dir.x * speed.0;
    velocity.z = dir.z * speed.0;
}