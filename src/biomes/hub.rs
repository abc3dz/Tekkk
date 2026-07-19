use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;
use bevy_wind_waker_shader::prelude::*;

use crate::components::*;
//use crate::npc::guardian::spawn_guardian_npc;

pub fn spawn_hub(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("maps/EvrmHub.glb")
            )
        ),
        WindWakerShaderBuilder::default()
            .time_of_day(TimeOfDay::Day)
            .weather(Weather::Sunny)
            .build(),
        Transform::default(),
        CurrentScene,
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(36.0, 0.1, 39.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    //wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(32.0, 3.0, 0.3),
        Transform::from_xyz(0.0, 1.0, -15.5),
    ));
    //gate
    commands.spawn((
        WarpToDesert,
        CurrentScene,
        Sensor,
        Collider::cuboid(2.0, 2.0, 2.0),
        Transform::from_xyz(0.0, 1.0, -15.0),
    ));
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/BGM_StartScene.ogg")),
        PlaybackSettings{
            mode: bevy::audio::PlaybackMode::Loop,
            volume:  bevy::audio::Volume::Linear(0.3),
            ..default()
        },
    ));
}
