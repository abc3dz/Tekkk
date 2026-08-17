use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;

use crate::components::*;
use crate::cel_shader::*;

pub fn spawn_desert(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("maps/EvrmDesert.glb"))),
        ApplyToonMaterial,
        Transform::from_xyz(0.0, -0.1, 0.0),
        CurrentScene,
    ));
    //ground
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(50.0, 0.1, 109.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    //warp
    commands.spawn((
        CurrentScene,
        WarpToHub,
        Collider::cuboid(2.0, 2.0, 2.0),
        Transform::from_xyz(5.0, 1.0, -15.0),
    ));

    if let Ok(mut player_tf) = player_query.single_mut() {
        player_tf.translation = Vec3::new(0.0, 2.0, 0.0);
    }
}
