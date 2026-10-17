use bevy::prelude::*;

use crate::components::*;
use crate::biomes::*;
use crate::warp_portal::*;
use crate::biomes::mtr_quicksand::*;
use crate::npc::guardian::*;
use crate::enemy::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {app
        .init_state::<GameScene>()
        //hub
        .add_plugins(GuardianPlugin)
        .add_plugins(WarpPortalPlugin)
        //desert
        .add_plugins(EnemyMuamuaPlugin)
        .add_plugins(EnemyMuanimPlugin)
        .add_plugins(EnemyChokyPlugin)
        .add_plugins(MaterialPlugin::<QuicksandMaterial>::default())
        //flaoting island
        .add_plugins(FiJaarPlugin)
        //lagoon
        .add_plugins(LgFrogzyPlugin)
        //volcano
        .add_plugins(VnDadaPlugin)
        .add_systems(OnEnter(GameScene::LoadingHub), spawn_loading_ui)
        .add_systems(Update, go_to_hub.run_if(in_state(GameScene::LoadingHub)))
        .add_systems(OnExit(GameScene::LoadingHub), cleanup_loading_ui)

        .add_systems(OnEnter(GameScene::Hub), (spawn_hub, setup_hub_light, spawn_warp_portal))
        .add_systems(Update, desert::check_warp_to_desert.run_if(in_state(GameScene::Hub)))
        .add_systems(Update, check_warp_to_lagoon.run_if(in_state(GameScene::Hub)),)
        .add_systems(OnExit(GameScene::Hub), cleanup_current_scene)

        .add_systems(OnEnter(GameScene::LoadingDesert), spawn_loading_ui)
        .add_systems(Update, desert::go_to_desert.run_if(in_state(GameScene::LoadingDesert)))
        .add_systems(OnExit(GameScene::LoadingDesert), cleanup_loading_ui)

        .add_systems(OnEnter(GameScene::Desert), (spawn_desert, setup_desert_light))
        .add_systems(Update, desert::check_warp_to_hub.run_if(in_state(GameScene::Desert)))
        .add_systems(Update, floating_island::check_warp_to_floating_island.run_if(in_state(GameScene::Desert))) 
        
        .add_systems(OnExit(GameScene::Desert), cleanup_current_scene)

        // .add_systems(OnEnter(GameScene::Desert),spawn_quicksand,)
        // .add_systems(Update,update_quicksand_shader.run_if(in_state(GameScene::Desert)))

        .add_systems(OnEnter(GameScene::LoadingFloatingIsland), spawn_loading_ui)
        .add_systems(Update, go_to_floating_island.run_if(in_state(GameScene::LoadingFloatingIsland)))
        .add_systems(OnExit(GameScene::LoadingFloatingIsland), cleanup_loading_ui)
        
        .add_systems(OnEnter(GameScene::FloatingIsland), (spawn_floating_island, setup_floating_island_light))
        .add_systems(Update, check_warp_to_desert_from_floating.run_if(in_state(GameScene::FloatingIsland)))
        .add_systems(OnExit(GameScene::FloatingIsland), cleanup_current_scene)
        //move platforms in floating island
        .add_systems(FixedUpdate, update_auto_platforms)

        .add_systems(Update,check_warp_to_lagoon.run_if(in_state(GameScene::FloatingIsland)),)
        // Loading Lagoon
        .add_systems(OnEnter(GameScene::LoadingLagoon), spawn_loading_ui)
        .add_systems(Update,go_to_lagoon.run_if(in_state(GameScene::LoadingLagoon)),)
        .add_systems(OnExit(GameScene::LoadingLagoon), cleanup_loading_ui)
        // Lagoon Scene
        .add_systems(OnEnter(GameScene::Lagoon),(spawn_lagoon, setup_lagoon_light, spawn_back_to_floating_island_portal, spawn_portal_to_volcano_in_lagoon))
        .add_systems(Update,check_warp_back_to_floating_island.run_if(in_state(GameScene::Lagoon)),)
        .add_systems(Update,check_warp_to_volcano.run_if(in_state(GameScene::Lagoon)),)
        .add_systems(OnExit(GameScene::Lagoon), cleanup_current_scene)
        // Loading Volcano
        .add_systems(OnEnter(GameScene::LoadingVolcano), spawn_loading_ui)
        .add_systems(Update,go_to_volcano.run_if(in_state(GameScene::LoadingVolcano)),)
        .add_systems(OnExit(GameScene::LoadingVolcano), cleanup_loading_ui)
        // Volcano Scene
        .add_systems(OnEnter(GameScene::Volcano),(spawn_volcano,setup_volcano_light,spawn_back_to_lagoon_portal,),)
        .add_systems(Update,check_warp_back_to_lagoon.run_if(in_state(GameScene::Volcano)),)
        .add_systems(OnExit(GameScene::Volcano), cleanup_current_scene);
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
