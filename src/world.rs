use bevy::prelude::*;

use crate::components::*;
use crate::biomes::*;
use crate::npc::guardian::GuardianPlugin;
use crate::enemy::enemy_muamua::*;
use crate::warp_portal::*;
use crate::enemy::enemy_choky::*;
use crate::biomes::mtr_quicksand::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {app
        .init_state::<GameScene>()
        .add_plugins(GuardianPlugin)
        .add_plugins(EnemyMuamuaPlugin)
        .add_plugins(WarpPortalPlugin)
        .add_plugins(EnemyChokyPlugin)
        .add_plugins(MaterialPlugin::<QuicksandMaterial>::default())
        
        .add_systems(OnEnter(GameScene::LoadingHub), spawn_loading_ui)
        .add_systems(Update, go_to_hub.run_if(in_state(GameScene::LoadingHub)))
        .add_systems(OnExit(GameScene::LoadingHub), cleanup_loading_ui)

        .add_systems(OnEnter(GameScene::Hub), (hub::spawn_hub, setup_hub_light, spawn_warp_portal))
        .add_systems(Update, check_warp_to_desert.run_if(in_state(GameScene::Hub)))
        .add_systems(OnExit(GameScene::Hub), cleanup_current_scene)

        .add_systems(OnEnter(GameScene::LoadingDesert), spawn_loading_ui)
        .add_systems(Update, go_to_desert.run_if(in_state(GameScene::LoadingDesert)))
        .add_systems(OnExit(GameScene::LoadingDesert), cleanup_loading_ui)

        .add_systems(OnEnter(GameScene::Desert), (desert::spawn_desert, setup_desert_light))
        .add_systems(Update, check_warp_to_hub.run_if(in_state(GameScene::Desert)))
        .add_systems(Update, check_warp_to_floating_island.run_if(in_state(GameScene::Desert))) // เปลี่ยนให้วาร์ปไป FloatingIsland
        
        .add_systems(OnExit(GameScene::Desert), cleanup_current_scene)

        .add_systems(OnEnter(GameScene::Desert),spawn_quicksand,)
        .add_systems(Update,update_quicksand_shader.run_if(in_state(GameScene::Desert)))

        .add_systems(OnEnter(GameScene::LoadingFloatingIsland), spawn_loading_ui)
        .add_systems(Update, go_to_floating_island.run_if(in_state(GameScene::LoadingFloatingIsland)))
        .add_systems(OnExit(GameScene::LoadingFloatingIsland), cleanup_loading_ui)
        
        .add_systems(OnEnter(GameScene::FloatingIsland), (floating_island::spawn_floating_island, setup_floating_island_light))
        .add_systems(Update, check_warp_to_desert_from_floating.run_if(in_state(GameScene::FloatingIsland))) // วาร์ปกลับไปที่ Desert
        .add_systems(OnExit(GameScene::FloatingIsland), cleanup_current_scene);
    }
}

fn spawn_loading_ui(mut commands: Commands) {
    commands.spawn((
        LoadingUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Loading..."),
            TextFont {font_size: 48.0, ..default()},
            TextColor(Color::WHITE),
        ));
    });
}
fn cleanup_loading_ui(
    mut commands: Commands,
    query: Query<Entity, With<LoadingUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
fn cleanup_current_scene(
    mut commands: Commands,
    query: Query<Entity, With<CurrentScene>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
fn go_to_hub(
    mut next_state: ResMut<NextState<GameScene>>,
) {
    next_state.set(GameScene::Hub);
    print!("Hub");
}
fn go_to_desert(
    mut next_state: ResMut<NextState<GameScene>>,
) {
    next_state.set(GameScene::Desert);
    print!("Desert");
}
fn check_warp_to_desert(
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
fn check_warp_to_hub(
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

fn setup_hub_light(mut commands: Commands) {
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

fn setup_desert_light(mut commands: Commands) {
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
fn go_to_floating_island(
    mut next_state: ResMut<NextState<GameScene>>,
) {
    next_state.set(GameScene::FloatingIsland);
    println!("Entering FloatingIsland");
}

fn check_warp_to_floating_island(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToFloatingIsland>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);
        if distance < 2.0 {
            next_state.set(GameScene::LoadingFloatingIsland);
        }
    }
}

fn check_warp_to_desert_from_floating(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToDesert>>, // ใช้ Component เดิมเพื่อเดินทางกลับ
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);
        if distance < 2.0 {
            next_state.set(GameScene::LoadingDesert);
        }
    }
}

fn setup_floating_island_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 40_000.0, // ปรับค่าความสว่างตามธีมของฉาก Floating Island
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