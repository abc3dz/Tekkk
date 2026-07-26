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
        CombatTarget,
        Element,
        ElementExpReward,
    },
    components::{
        Enemy,
        EnemyState,
        GameScene,
        Health,
        Player,
        EnemyMuamuaAnimationGraph,
        EnemyMuamuaAnimationTarget,
        EnemyMuamuaAnimState
    },
};
use crate::npc::practice_common::spawn_enemy_health_bar;

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
                enemy_muamua_chase_player,
                update_enemy_muamua_animation,
                debug_enemy_muamua_spawn,
            )
            .chain()
            .run_if(in_state(GameScene::Desert)),
        );
    }
}

fn spawn_enemy_muamua(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let muamua_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("enemy/EnemyMuamua.glb"));
    let spawn_positions = Vec3::new(5.0, 0.0, 5.0);
    const MUAMUA_BODY_Y: f32 = 1.0;
    let base_stats = BaseStats::MUAMUA;
    let enemy_muamua = commands.spawn((
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
        RigidBody::Dynamic,
        Collider::capsule(0.45, 1.0),
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity::ZERO,

        Transform::from_translation(
            spawn_positions + Vec3::Y * MUAMUA_BODY_Y,
        ),
        
        // ออกจาก Desert แล้วลบ Muamua
        DespawnOnExit(GameScene::Desert),
    )).with_children(|parent| {
            parent.spawn((SceneRoot(muamua_scene.clone()),
                Transform::from_xyz(0.0,-MUAMUA_BODY_Y,0.0,),
                WindWakerShaderBuilder::default().time_of_day(TimeOfDay::Day).weather(Weather::Sunny).build(),
            ));
        }).id();

    commands.entity(enemy_muamua).insert(CombatTarget);
    spawn_enemy_health_bar(
        &mut commands,
        enemy_muamua,
    );
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
    let chase = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(4)
                .from_asset("enemy/EnemyMuamua.glb"),
        ),
        1.0,
        graph.root,
    );

    commands.insert_resource(EnemyMuamuaAnimationGraph {
        graph: graphs.add(graph),
        idle,
        chase,
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
        let Some(muamua_root) = find_enemy_muamua_root(
            animation_entity,
            &child_of_query,
            &muamua_query,
        ) else {
            continue;
        };

        commands.entity(animation_entity).insert((
            AnimationGraphHandle(animation_graph.graph.clone()),

            EnemyMuamuaAnimationTarget {
                root: muamua_root,
            },

            EnemyMuamuaAnimState::Idle,
        ));

        player.stop_all();
        player.play(animation_graph.idle).repeat();
    }
}

fn find_enemy_muamua_root(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    muamua_query: &Query<(), With<EnemyMuamua>>,
) -> Option<Entity> {
    let mut current = entity;

    loop {
        if muamua_query.get(current).is_ok() {
            return Some(current);
        }

        let Ok(child_of) = child_of_query.get(current) else {
            return None;
        };

        current = child_of.parent();
    }
}

const MUAMUA_CHASE_RANGE: f32 = 10.0;
const MUAMUA_STOP_DISTANCE: f32 = 1.5;
const MUAMUA_MOVE_SPEED: f32 = 3.0;

fn enemy_muamua_chase_player(
    player_query: Query<
        &Transform,
        (With<Player>, Without<EnemyMuamua>),
    >,

    mut muamua_query: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut EnemyState,
        ),
        (With<EnemyMuamua>, Without<Player>),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (
        mut muamua_transform,
        mut velocity,
        mut enemy_state,
    ) in &mut muamua_query
    {
        let to_player =
            player_transform.translation
                - muamua_transform.translation;

        // ไม่สนแกน Y เพราะเดินอยู่บนพื้น
        let flat_direction = Vec3::new(
            to_player.x,
            0.0,
            to_player.z,
        );

        let distance = flat_direction.length();

        // Player อยู่นอกระยะตรวจจับ
        if distance > MUAMUA_CHASE_RANGE {
            velocity.x = 0.0;
            velocity.z = 0.0;

            *enemy_state = EnemyState::Idle;
            continue;
        }

        // เข้าใกล้ Player แล้ว ให้หยุดก่อน
        // ภายหลังตรงนี้ค่อยเปลี่ยนเป็น Attack
        if distance <= MUAMUA_STOP_DISTANCE {
            velocity.x = 0.0;
            velocity.z = 0.0;

            *enemy_state = EnemyState::Idle;
            continue;
        }

        let direction = flat_direction.normalize();

        velocity.x = direction.x * MUAMUA_MOVE_SPEED;
        velocity.z = direction.z * MUAMUA_MOVE_SPEED;

        // หันหน้าเข้าหา Player
        muamua_transform.rotation =
            Quat::from_rotation_y(
                direction.x.atan2(direction.z),
            );

        *enemy_state = EnemyState::Chase;
    }
}

fn update_enemy_muamua_animation(
    animation_graph: Res<EnemyMuamuaAnimationGraph>,

    muamua_query: Query<
        &EnemyState,
        With<EnemyMuamua>,
    >,

    mut animation_query: Query<
        (
            &EnemyMuamuaAnimationTarget,
            &mut AnimationPlayer,
            &mut EnemyMuamuaAnimState,
        ),
    >,
) {
    for (
        animation_target,
        mut animation_player,
        mut current_animation,
    ) in &mut animation_query
    {
        let Ok(enemy_state) =
            muamua_query.get(animation_target.root)
        else {
            continue;
        };

        let wanted_animation = match enemy_state {
            EnemyState::Chase => {
                EnemyMuamuaAnimState::Chase
            }

            _ => EnemyMuamuaAnimState::Idle,
        };

        // Animation เดิมกำลังเล่นอยู่ ไม่ต้องเริ่มใหม่ทุก Frame
        if *current_animation == wanted_animation {
            continue;
        }

        animation_player.stop_all();

        match wanted_animation {
            EnemyMuamuaAnimState::Idle => {
                animation_player
                    .play(animation_graph.idle)
                    .repeat();
            }

            EnemyMuamuaAnimState::Chase => {
                animation_player
                    .play(animation_graph.chase)
                    .repeat();
            }
        }

        *current_animation = wanted_animation;
    }
}