use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;

use crate::components::*;
use crate::cel_shader::*;

pub fn spawn_floating_island(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("maps/EvrmFloatingIsland.glb"))),
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
    //sloped
     commands.spawn((
        RigidBody::Static,
        Collider::cuboid(10.0, 0.1, 100.0),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(35.0_f32.to_radians())),
    ));
    //ground2
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(50.0, 0.1, 30.0),
        Transform::from_xyz(7.0, 20.0, -9.0),
    ));
    //warp
    commands.spawn((
        WarpToDesert,
        CurrentScene,
        Sensor,
        Collider::cuboid(2.0, 2.0, 2.0),
        Transform::from_xyz(0.0, 1.0, -5.0),
    ));
}