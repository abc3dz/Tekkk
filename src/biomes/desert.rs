use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;

use crate::components::*;
use crate::cel_shader::*;

pub fn spawn_desert(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
        DespawnOnExit(GameScene::Desert),
    ));
    //warp to hub
    commands.spawn((
        CurrentScene,
        WarpToHub,
        Collider::cuboid(2.0, 2.0, 1.0),
        Transform::from_xyz(0.0, 1.0, 30.0),
        DespawnOnExit(GameScene::Desert),
    ));
    //warp to floating island
    // commands.spawn((
    //     CurrentScene,
    //     WarpToFloatingIsland,
    //     Collider::cuboid(2.0, 2.0, 1.0),
    //     Transform::from_xyz(17.5, 1.0, -48.0),
    //     DespawnOnExit(GameScene::Desert),
    // ));
    
    //mountain left
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 10.0, 100.0),
        Transform::from_xyz(-25.0, 0.0, 0.0),
        DespawnOnExit(GameScene::Desert),
    ));
    //mountain right
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 10.0, 110.0),
        Transform::from_xyz(25.0, 0.0, 0.0),
        DespawnOnExit(GameScene::Desert),
    ));
    //mountain back
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(100.0, 10.0, 0.3),
        Transform::from_xyz(0.0, 0.0, -55.0),
        DespawnOnExit(GameScene::Desert),
    ));
    //pillar down left
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 8.0, 1.0),
        Transform::from_xyz(13.0, 0.0,-47.0),
        DespawnOnExit(GameScene::Desert),
    ));
    //pillar down right
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 8.0, 1.0),
        Transform::from_xyz(19.7, 0.0,-45.6),
        DespawnOnExit(GameScene::Desert),
    ));
    //pillar top left
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 8.0, 1.0),
        Transform::from_xyz(15.2, 0.0,-51.86),
        DespawnOnExit(GameScene::Desert),
    ));
    //pillar top right
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 8.0, 1.0),
        Transform::from_xyz(20.0, 0.0,-52.86),
        DespawnOnExit(GameScene::Desert),
    ));
    //stand
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(2.92, 0.187, 3.11),
        Transform::from_xyz(17.5, 0.0,-48.11),
        DespawnOnExit(GameScene::Desert),
    ));
    //castle base
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(12.3, 2.4, 15.0),
        Transform::from_xyz(-12.51, 0.0,-49.5),
        DespawnOnExit(GameScene::Desert),
    ));
    //castle right wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 8.5, 15.0),
        Transform::from_xyz(-7.2, 0.0,-49.5),
        DespawnOnExit(GameScene::Desert),
    ));
    //castle left wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 8.5, 15.0),
        Transform::from_xyz(-18.2, 0.0,-49.5),
        DespawnOnExit(GameScene::Desert),
    ));
    //castle back wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(12.5, 8.5, 1.0),
        Transform::from_xyz(-12.6, 0.0,-54.0),
        DespawnOnExit(GameScene::Desert),
    ));
    //castle front left wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(4.0, 8.5, 1.0),
        Transform::from_xyz(-14.95, 0.0,-42.57),
        DespawnOnExit(GameScene::Desert),
    ));
    //castle front right wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(4.0, 8.5, 1.0),
        Transform::from_xyz(-9.58, 0.0,-42.57),
        DespawnOnExit(GameScene::Desert),
    ));
    //castle front top wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(2.0, 1.0, 1.0),
        Transform::from_xyz(-12.67, 4.20,-42.57),
        DespawnOnExit(GameScene::Desert),
    ));
    //castle front stair
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.0, 0.9, 2.5),
        Transform::from_xyz(-12.26, 0.2,-41.07).with_rotation(Quat::from_rotation_x(25f32.to_radians())),
        DespawnOnExit(GameScene::Desert),
    ));
    //cactus wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.8, 3.2, 0.9),
        Transform::from_xyz(-18.1, 0.0,3.25),
        DespawnOnExit(GameScene::Desert),
    ));
    //cactus end
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.8, 4.2, 0.9),
        Transform::from_xyz(-7.26, 0.0,8.2),
        DespawnOnExit(GameScene::Desert),
    ));
    //rock end
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.4, 2.9, 1.87),
        Transform::from_xyz(-12.9, 0.0,10.81),
        DespawnOnExit(GameScene::Desert),
    ));
    //end
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(55.0, 10.0, 1.0),
        Transform::from_xyz(0.0, 0.0,36.82),
        DespawnOnExit(GameScene::Desert),
    ));
    
    
    //sound
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/BGM_Dimension.ogg")),
        PlaybackSettings{mode: bevy::audio::PlaybackMode::Loop, volume:  bevy::audio::Volume::Linear(0.1), ..default()},
        MusicAudio,
        DespawnOnExit(GameScene::Desert),
    ));
}
pub fn go_to_desert(
    mut next_state: ResMut<NextState<GameScene>>,
    mut scene_spawn: ResMut<SceneSpawnPoint>,
) {
    scene_spawn.position = Some(Vec3::new(0.0, 2.0, 21.7));
    next_state.set(GameScene::Desert);
}
pub fn check_warp_to_desert(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToDesert>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single()
    else { return };

    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);

        if distance < 2.0 {
            next_state.set(GameScene::LoadingDesert);
        }
    }
}

pub fn check_warp_to_hub(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToHub>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() 
    else { return };

    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);
        if distance < 2.0 {
            next_state.set(GameScene::LoadingHub);
        }
    }
}

pub fn setup_desert_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
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