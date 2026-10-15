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
        Collider::cuboid(2.0, 2.0, 2.0),
        Transform::from_xyz(5.0, 1.0, 15.0),
        DespawnOnExit(GameScene::Desert),
    ));
    //warp to floating island
    commands.spawn((
        CurrentScene,
        WarpToFloatingIsland,
        Collider::cuboid(2.0, 2.0, 2.0),
        Transform::from_xyz(17.5, 1.0, -48.0),
        DespawnOnExit(GameScene::Desert),
    ));
    
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
}
pub fn go_to_desert(
    mut next_state: ResMut<NextState<GameScene>>,
    mut scene_spawn: ResMut<SceneSpawnPoint>,
) {
    scene_spawn.position = Some(Vec3::new(0.0, 2.0, 0.0));
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