use avian3d::prelude::*;
use bevy::{
    gltf::GltfAssetLabel,
    prelude::*,
};

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
    },
};

#[derive(Component, Debug)]
pub struct EnemyMuamua;

pub struct EnemyMuamuaPlugin;

impl Plugin for EnemyMuamuaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameScene::Desert),
            spawn_enemy_muamua,
        )
        .add_systems(
            Update,
            debug_enemy_muamua_spawn
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
            Collider::capsule(0.45, 1.0),

            // Model
            SceneRoot(muamua_scene.clone()),
            Transform::from_translation(position),

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