use bevy::prelude::*;
use crate::components::Player;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, follow_player);
    }
}

#[derive(Component)]
struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    let Ok(mut camera_tf) = camera_query.single_mut() else {
        return;
    };

    let target = player_tf.translation + Vec3::new(0.0, 8.0, 10.0);
    camera_tf.translation = camera_tf.translation.lerp(target, 0.1);
    camera_tf.look_at(player_tf.translation, Vec3::Y);
}