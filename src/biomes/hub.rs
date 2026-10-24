use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;

use crate::components::*;
use crate::cel_shader::*;
use crate::musical;

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
    //doorstopper
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(0.3, 5.0),
        //Transform::from_xyz(9.5, 1.0, 16.0),
        Transform::from_xyz(0.0, 1.2, 16.5)
        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
         // หมุน 90° รอบแกน X → cylinder จะนอนตามแกน Z
        DespawnOnExit(GameScene::Hub),
    ));
    //chair wood
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.69, 0.1, 1.84),
        Transform::from_xyz(-14.17, 0.7,-13.2),
        DespawnOnExit(GameScene::Hub),
    ));
    //tree near cw 
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 3.0, 0.3),
        Transform::from_xyz(-12.6, 1.5,-14.0),
        DespawnOnExit(GameScene::Hub),
    ));
    //tree1 back
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 1.5, 0.3),
        Transform::from_xyz(-7.4, 0.75,-14.5),
        DespawnOnExit(GameScene::Hub),
    ));
    //tree2 back
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 1.5, 0.3),
        Transform::from_xyz(-5.8, 0.75,-14.11),
        DespawnOnExit(GameScene::Hub),
    ));
    //tree3 back
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 1.5, 0.3),
        Transform::from_xyz(-4.0, 0.75,-13.65),
        DespawnOnExit(GameScene::Hub),
    ));
    //pillar back 
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.42, 3.0, 0.39),
        Transform::from_xyz(-7.3, 1.5,-9.5),
        DespawnOnExit(GameScene::Hub),
    ));
    //moutain back
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(55.0, 15.0, 1.0),
        Transform::from_xyz(0.0, 0.0,-18.6),
        DespawnOnExit(GameScene::Hub),
    ));
    //moutain left
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 15.0, 55.0),
        Transform::from_xyz(-18.0, 0.0,0.0),
        DespawnOnExit(GameScene::Hub),
    ));
    //moutain right
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 15.0, 55.0),
        Transform::from_xyz(18.0, 0.0,0.0),
        DespawnOnExit(GameScene::Hub),
    ));
    //end scene
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(55.0, 15.0, 1.0),
        Transform::from_xyz(0.0, 0.0,19.35),
        DespawnOnExit(GameScene::Hub),
    ));

    musical::spawn_drum_at(
        &mut commands,
        &asset_server,
        Transform::from_xyz(0.0, 0.1, 7.5),
    );
    musical::spawn_bass_at(
        &mut commands,
        &asset_server,
        Transform::from_xyz(-2.4, 0.1, 8.5),
    );
    musical::spawn_guitar_at(
        &mut commands,
        &asset_server,
        Transform::from_xyz(-1.4, 0.1, 9.5),
    );
    musical::spawn_turntable_at(
        &mut commands,
        &asset_server,
        Transform::from_xyz(2.4, 0.1, 8.5),
    );

    //warp
    commands.spawn((
        WarpToDesert,
        CurrentScene,
        Sensor,
        Collider::cuboid(2.0, 2.0, 1.0),
        Transform::from_xyz(0.0, 1.0, -15.0),
        DespawnOnExit(GameScene::Hub),
    ));
    commands.spawn((
        WarpToLagoon,
        CurrentScene,
        Sensor,
        Collider::cuboid(2.0, 2.0, 1.0),
        Transform::from_xyz(10.0, 1.0, -15.0),
        DespawnOnExit(GameScene::Hub),
    ));
    //sound
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/BGM_StartScene.ogg")),
        PlaybackSettings{mode: bevy::audio::PlaybackMode::Loop, volume:  bevy::audio::Volume::Linear(0.1), ..default()},
        MusicAudio,
        DespawnOnExit(GameScene::Hub),
    ));
    //txt
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
        Text::new("Tekkk V.0.0.1"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        CurrentScene,
        DespawnOnExit(GameScene::Hub),
    ));

}
pub fn go_to_hub(
    mut next_state: ResMut<NextState<GameScene>>,
    mut scene_spawn: ResMut<SceneSpawnPoint>,
) {
    scene_spawn.position = Some(Vec3::new(0.0, 2.0, 0.0));
    next_state.set(GameScene::Hub);
}

pub fn setup_hub_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 50_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.8,
            -0.3,
            0.0,
        )),
        CurrentScene,
    ));
}