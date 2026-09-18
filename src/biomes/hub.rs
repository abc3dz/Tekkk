use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;

use crate::components::*;
use crate::cel_shader::*;

pub fn spawn_hub(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("maps/EvrmHub.glb"))),
        ApplyToonMaterial,
        CurrentScene,

    ));
    //ground
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(36.0, 0.1, 39.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
        DespawnOnExit(GameScene::Hub),
    ));
    //wall back
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(32.0, 2.9, 0.3),
        Transform::from_xyz(0.0, 1.5, -15.7),
        DespawnOnExit(GameScene::Hub),
    ));
    //wall right
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 2.9, 32.0),
        Transform::from_xyz(15.8, 1.5, 0.0),
        DespawnOnExit(GameScene::Hub),
    ));
    //wall left
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 2.9, 32.0),
        Transform::from_xyz(-15.80, 1.5, 0.0),
        DespawnOnExit(GameScene::Hub),
    ));
    //wall front left
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(13.0, 2.9, 0.3),
        Transform::from_xyz(-9.5, 1.5, 16.0),
        DespawnOnExit(GameScene::Hub),
    ));
    //wall front right
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(13.0, 2.9, 0.3),
        Transform::from_xyz(9.5, 1.5, 16.0),
        DespawnOnExit(GameScene::Hub),
    ));
    //wall front right
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(0.3, 5.0),
        //Transform::from_xyz(9.5, 1.0, 16.0),
        Transform::from_xyz(0.0, 1.2, 16.5)
        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
    // หมุน 90° รอบแกน X → cylinder จะนอนตามแกน Z
        DespawnOnExit(GameScene::Hub),
    ));
    //warp
    commands.spawn((
        WarpToDesert,
        CurrentScene,
        Sensor,
        Collider::cuboid(2.0, 2.0, 2.0),
        Transform::from_xyz(0.0, 1.0, -15.0),
        DespawnOnExit(GameScene::Hub),
    ));
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/BGM_StartScene.ogg")),
        PlaybackSettings{mode: bevy::audio::PlaybackMode::Loop, volume:  bevy::audio::Volume::Linear(0.1), ..default()},
        MusicAudio,
        DespawnOnExit(GameScene::Hub),
    ));
}
