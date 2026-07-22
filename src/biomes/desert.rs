use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;
use bevy_wind_waker_shader::prelude::*;

use crate::components::*;

pub fn spawn_desert(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("maps/EvrmDesert.glb"))),
        WindWakerShaderBuilder::default().time_of_day(TimeOfDay::Day).weather(Weather::Sunny).build(),
        Transform::default(),
        CurrentScene,
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(50.0, 0.1, 50.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        CurrentScene,
        WarpToHub,
        Transform::from_xyz(0.0, 0.5, -8.0),
        GlobalTransform::default(),
    ));

    if let Ok(mut player_tf) = player_query.single_mut() {
        player_tf.translation = Vec3::new(0.0, 2.0, 0.0);
    }
}
