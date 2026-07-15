use bevy::prelude::*;
//use avian3d::prelude::*;

use crate::components::*;
use crate::biomes::{hub, desert};
use crate::npc::guardian::GuardianPlugin;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameScene>()
            .add_plugins(GuardianPlugin)

            .add_systems(OnEnter(GameScene::LoadingHub), spawn_loading_ui)
            .add_systems(Update, go_to_hub.run_if(in_state(GameScene::LoadingHub)))
            .add_systems(OnExit(GameScene::LoadingHub), cleanup_loading_ui)

            .add_systems(OnEnter(GameScene::Hub), hub::spawn_hub)
            .add_systems(Update, check_warp_to_desert.run_if(in_state(GameScene::Hub)))
            .add_systems(OnExit(GameScene::Hub), cleanup_current_scene)

            .add_systems(OnEnter(GameScene::LoadingDesert), spawn_loading_ui)
            .add_systems(Update, go_to_desert.run_if(in_state(GameScene::LoadingDesert)))
            .add_systems(OnExit(GameScene::LoadingDesert), cleanup_loading_ui)

            .add_systems(OnEnter(GameScene::Desert), desert::spawn_desert)
            .add_systems(Update, check_warp_to_hub.run_if(in_state(GameScene::Desert)))
            .add_systems(OnExit(GameScene::Desert), cleanup_current_scene);
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
            TextFont {
                font_size: 48.0,
                ..default()
            },
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
    let Ok(player_tf) = player_query.single() else {
        return;
    };

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
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);

        if distance < 2.0 {
            next_state.set(GameScene::LoadingHub);
        }
    }
}