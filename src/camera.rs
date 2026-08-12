use bevy::prelude::*;
use crate::components::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    follow_player,
                    update_camera_shake.after(follow_player),
                ),
            );
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct CameraShake {
    timer: Timer,
    duration: f32,
    strength: f32,
    last_offset: Vec3,
}

impl CameraShake {
    pub fn new(duration: f32, strength: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            duration,
            strength,
            last_offset: Vec3::ZERO,
        }
    }
}


fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 2.0, 3.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}


fn follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<
        &mut Transform,
        (With<MainCamera>, Without<Player>, Without<CameraShake>)
    >,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    let Ok(mut camera_tf) = camera_query.single_mut() else {
        return;
    };

    let target =
        player_tf.translation + Vec3::new(0.0, 4.0, 7.0);

    camera_tf.translation =
        camera_tf.translation.lerp(target, 0.1);

    camera_tf.look_at(player_tf.translation, Vec3::Y);
}


pub fn update_camera_shake(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut Transform, &mut CameraShake),
        With<MainCamera>,
    >,
) {
    for (entity, mut transform, mut shake) in &mut query {

        // เอา offset frame เก่าออกก่อน
        transform.translation -= shake.last_offset;

        shake.timer.tick(time.delta());

        if shake.timer.is_finished() {
            commands.entity(entity).remove::<CameraShake>();
            continue;
        }

        let elapsed = shake.timer.elapsed_secs();

        let decay =
            1.0 - (elapsed / shake.duration);

        let x = (elapsed * 75.0).sin();
        let y = (elapsed * 93.0).cos();

        let offset =
            Vec3::new(x, y, 0.0)
            * shake.strength
            * decay;

        transform.translation += offset;

        shake.last_offset = offset;
    }
}