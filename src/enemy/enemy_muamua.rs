use avian3d::prelude::*;
use bevy::{
    gltf::GltfAssetLabel,
    prelude::*,
};
use bevy_wind_waker_shader::prelude::*;

use crate::{
    combat::{
        AtkAndDefElement,
        BaseStats,
        CombatStats,
        Element,
        ElementExpReward,
    },
    components::{
        Enemy,
        EnemyState,
        GameScene,
        Health,
        EnemyMuamuaAnimationGraph,
        EnemyMuamuaAnimationTarget,
    },
};

#[derive(Component, Debug)]
pub struct EnemyMuamua;

pub struct EnemyMuamuaPlugin;

impl Plugin for EnemyMuamuaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameScene::Desert),
            (
                setup_enemy_muamua_animation_graph,
                spawn_enemy_muamua,
            ),
        )
        .add_systems(
            Update,
            (
                setup_enemy_muamua_animation_player,
                debug_enemy_muamua_spawn,
            )
                .run_if(in_state(GameScene::Desert)),
        );
    }
}

fn spawn_enemy_muamua(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let muamua_scene = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("enemy/EnemyMuamua.glb"),
    );

    let spawn_positions = [
        Vec3::new(5.0, 0.0, 5.0),
    ];

    for position in spawn_positions {
        let base_stats = BaseStats::MUAMUA;
        commands.spawn((
            Name::new("Enemy Muamua"),

            // Marker
            Enemy,
            EnemyMuamua,
            EnemyState::Idle,

            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },

            base_stats,
            CombatStats::from(base_stats),

            AtkAndDefElement(Element::Earth),
            ElementExpReward::MUAMUA,

            // Physics
            RigidBody::Kinematic,
            // Collider::capsule(0.45, 1.0),
            Collider::capsule_endpoints(
                0.45,
                Vec3::new(0.0, 0.45, 0.0),
                Vec3::new(0.0, 1.65, 0.0),
            ),


            // Model
            SceneRoot(muamua_scene.clone()),
            Transform::from_translation(position),
            WindWakerShaderBuilder::default().time_of_day(TimeOfDay::Day).weather(Weather::Sunny).build(),

            // ออกจาก Desert แล้วลบ Muamua
            DespawnOnExit(GameScene::Desert),
        ));
    }
}

fn debug_enemy_muamua_spawn(
    query: Query<
        (
            &Health,
            &CombatStats,
            &AtkAndDefElement,
            &ElementExpReward,
            &EnemyState,
        ),
        Added<EnemyMuamua>,
    >,
) {
    for (health, stats, element, reward, state) in &query {
        info!(
            "Muamua spawned:
            HP = {}/{},
            Element = {:?},
            ATK = {},
            DEF = {},
            Reward = Water {}-{}, Fire {}-{}, Wind {}-{}, Earth {}-{}, Inw {}-{},
            State = {:?}",
            health.current,
            health.max,
            element.0,
            stats.attack,
            stats.defense,
            reward.water.min,
            reward.water.max,
            reward.fire.min,
            reward.fire.max,
            reward.wind.min,
            reward.wind.max,
            reward.earth.min,
            reward.earth.max,
            reward.inw.min,
            reward.inw.max,
            state,
        );
    }
}

fn setup_enemy_muamua_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(3)
                .from_asset("enemy/EnemyMuamua.glb"),
        ),
        1.0,
        graph.root,
    );

    commands.insert_resource(EnemyMuamuaAnimationGraph {
        graph: graphs.add(graph),
        idle,
    });
}

fn setup_enemy_muamua_animation_player(
    mut commands: Commands,
    animation_graph: Res<EnemyMuamuaAnimationGraph>,

    mut animation_players: Query<
        (Entity, &mut AnimationPlayer),
        Added<AnimationPlayer>,
    >,

    child_of_query: Query<&ChildOf>,
    muamua_query: Query<(), With<EnemyMuamua>>,
) {
    for (animation_entity, mut player) in &mut animation_players {
        if !belongs_to_enemy_muamua(
            animation_entity,
            &child_of_query,
            &muamua_query,
        ) {
            continue;
        }

        commands.entity(animation_entity).insert((
            AnimationGraphHandle(animation_graph.graph.clone()),
            EnemyMuamuaAnimationTarget,
        ));

        player.stop_all();
        player.play(animation_graph.idle).repeat();
    }
}

fn belongs_to_enemy_muamua(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    muamua_query: &Query<(), With<EnemyMuamua>>,
) -> bool {
    let mut current = entity;

    loop {
        if muamua_query.get(current).is_ok() {
            return true;
        }

        let Ok(child_of) = child_of_query.get(current) else {
            return false;
        };

        current = child_of.parent();
    }
}