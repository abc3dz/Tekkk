use avian3d::prelude::*;
use bevy::{
    gltf::GltfAssetLabel,
    prelude::*,
};

use crate::cel_shader::*;
use crate::components::*;
use crate::npc::practice_common::*;
use crate::player::*;
use crate::camera::*;
use crate::pause_menu::GameMode;

#[derive(Component, Debug)]
pub struct EnemyDada;

pub struct VnDadaPlugin;

impl Plugin for VnDadaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DadaRespawnTimer(Timer::from_seconds(1.0, TimerMode::Once)))
            .init_resource::<DadaSpawnedCount>()
            .init_resource::<DadaDefeatedCount>()
            .add_systems(
                OnEnter(GameScene::Volcano),
                (
                    reset_enemy_dada_wave,
                    setup_dada_animation_graph,
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_enemy_dada,
                    setup_dada_animation_player,
                    enemy_dada_chase_player,
                    update_dada_animation,
                    update_dada_hurt_and_dead,
                    spawn_dada_punch_hitbox,
                    dada_punch_hit_player,
                    despawn_dada_punch_hitbox,
                )
                    .chain()
                    .run_if(in_state(GameScene::Volcano).and(in_state(GameMode::Playing))),
            );
    }
}

const DADA_SPAWN_POSITION: Vec3 = Vec3::new(0.0, 1.0, -15.0);
const DADA_MAX_ALIVE: usize = 3;
const DADA_CHASE_RANGE: f32 = 10.0;
const DADA_STOP_DISTANCE: f32 = 1.0;
const DADA_MOVE_SPEED: f32 = 3.0;
const DADA_PATROL_RADIUS: f32 = 8.0;

fn spawn_enemy_dada(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    defeated_count: Res<DadaDefeatedCount>,
    mut spawned_count: ResMut<DadaSpawnedCount>,
    dada_query: Query<(), With<EnemyDada>>,
) {
    // ถ้าแพ้ครบแล้ว ไม่ต้อง spawn เพิ่ม
    if defeated_count.0 >= DADA_DEFEAT_GOAL {
        return;
    }

    // ถ้าเคยเกิดครบตามเป้าไปแล้ว ไม่ต้อง spawn เพิ่ม
    if spawned_count.0 >= DADA_DEFEAT_GOAL {
        return;
    }

    let alive_count = dada_query.iter().count();
    let remaining_to_spawn = DADA_DEFEAT_GOAL.saturating_sub(spawned_count.0) as usize;
    let max_alive = DADA_MAX_ALIVE.min(remaining_to_spawn);

    // ถ้าตัวที่ยังมีชีวิตอยู่เต็มจำนวนแล้ว ยังไม่ต้อง spawn
    if alive_count >= max_alive {
        return;
    }

    let dada_scene = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("enemy/Vn_Dada.glb"),
    );

    let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
    let initial_dir = Vec3::new(random_angle.cos(), 0.0, random_angle.sin()).normalize();

    const DADA_BODY_Y: f32 = 1.0;

    let base_stats = BaseStats::DADA;

    let enemy_dada = commands
        .spawn((
            Name::new("Enemy Dada"),
            Enemy,
            EnemyDada,
            EnemyState::Patrol,
            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },
            base_stats,
            CombatStats::from(base_stats),
            AtkAndDefElement(Element::Fire),
            ElementExpReward::DADA,
            RigidBody::Dynamic,
            Collider::capsule(0.45, 1.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            Transform::from_translation(DADA_SPAWN_POSITION + Vec3::Y * DADA_BODY_Y),
            DespawnOnExit(GameScene::Volcano),
        ))
        .insert(DadaPatrol {
            direction: initial_dir,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            center: DADA_SPAWN_POSITION,
            radius: DADA_PATROL_RADIUS,
        })
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(dada_scene),
                Transform::from_xyz(0.0, -DADA_BODY_Y, 0.0),
                ApplyToonMaterial,
            ));
        })
        .id();

    commands.entity(enemy_dada).insert((
        CombatTarget,
        DadaAttackTimer(Timer::from_seconds(
            0.3,
            TimerMode::Repeating,
        )),
    ));

    spawn_enemy_health_bar(
        &mut commands,
        enemy_dada,
    );

    spawned_count.0 += 1;
}

fn setup_dada_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(3)
                .from_asset("enemy/Vn_Dada.glb"),
        ),
        1.0,
        graph.root,
    );

    let chase = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(4)
                .from_asset("enemy/Vn_Dada.glb"),
        ),
        1.0,
        graph.root,
    );

    let attack = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(0)
                .from_asset("enemy/Vn_Dada.glb"),
        ),
        1.0,
        graph.root,
    );

    let hurt = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(2)
                .from_asset("enemy/Vn_Dada.glb"),
        ),
        1.0,
        graph.root,
    );

    let dead = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(1)
                .from_asset("enemy/Vn_Dada.glb"),
        ),
        1.0,
        graph.root,
    );

    commands.insert_resource(EnemyDadaAnimationGraph {
        graph: graphs.add(graph),
        idle,
        chase,
        attack,
        hurt,
        dead,
    });
}

fn setup_dada_animation_player(
    mut commands: Commands,
    animation_graph: Res<EnemyDadaAnimationGraph>,
    mut animation_players: Query<
        (Entity, &mut AnimationPlayer),
        Added<AnimationPlayer>,
    >,
    child_of_query: Query<&ChildOf>,
    dada_query: Query<(), With<EnemyDada>>,
) {
    for (animation_entity, mut player) in &mut animation_players {
        let Some(dada_root) = find_enemy_dada_root(
            animation_entity,
            &child_of_query,
            &dada_query,
        ) else {
            continue;
        };

        commands.entity(animation_entity).insert((
            AnimationGraphHandle(animation_graph.graph.clone()),
            EnemyDadaAnimationTarget {
                root: dada_root,
            },
            EnemyDadaAnimState::Idle,
        ));

        player.stop_all();
        player.play(animation_graph.idle).repeat();
    }
}

fn find_enemy_dada_root(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    dada_query: &Query<(), With<EnemyDada>>,
) -> Option<Entity> {
    let mut current = entity;

    loop {
        if dada_query.get(current).is_ok() {
            return Some(current);
        }

        let Ok(child_of) = child_of_query.get(current) else {
            return None;
        };

        current = child_of.parent();
    }
}

fn enemy_dada_chase_player(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<
        &Transform,
        (
            With<Player>,
            Without<EnemyDada>,
        ),
    >,
    mut dada_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut EnemyState,
            Option<&mut EnemyInvestigateDirection>,
            Option<&mut DadaPatrol>,
        ),
        (
            With<EnemyDada>,
            Without<Player>,
        ),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (
        dada_entity,
        mut dada_transform,
        mut velocity,
        mut enemy_state,
        investigate,
        patrol_data,
    ) in &mut dada_query
    {
        // Hurt / Dead ยังหยุดเหมือนเดิม
        if matches!(*enemy_state, EnemyState::Hurt | EnemyState::Dead) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        let to_player = player_transform.translation - dada_transform.translation;
        let flat_direction = Vec3::new(to_player.x, 0.0, to_player.z);
        let distance = flat_direction.length();

        // ==========================================
        // 1. Player อยู่ในระยะมองเห็น (<= 10.0) -> ไล่ตามทันที
        // ==========================================
        if distance <= DADA_CHASE_RANGE {
            commands.entity(dada_entity).remove::<EnemyInvestigateDirection>();

            // Player อยู่ในระยะโจมตี
            if distance <= DADA_STOP_DISTANCE {
                velocity.x = 0.0;
                velocity.z = 0.0;

                if flat_direction.length_squared() > 0.0001 {
                    let direction = flat_direction.normalize();
                    dada_transform.rotation =
                        Quat::from_rotation_y(direction.x.atan2(direction.z));
                }

                *enemy_state = EnemyState::Attack;
                continue;
            }

            // ไล่ Player ตามปกติ
            let direction = flat_direction.normalize();
            velocity.x = direction.x * DADA_MOVE_SPEED;
            velocity.z = direction.z * DADA_MOVE_SPEED;

            dada_transform.rotation =
                Quat::from_rotation_y(direction.x.atan2(direction.z));

            *enemy_state = EnemyState::Chase;
            continue;
        }

        // ==========================================
        // 2. Player อยู่นอกระยะ (> 10.0)
        // ==========================================

        // 2.1 โดนยิงมา (มี Investigate) -> เดินกลับไปจุดเกิด
        if let Some(mut investigate) = investigate {
            investigate.timer.tick(time.delta());

            if investigate.timer.is_finished() {
                let to_spawn = DADA_SPAWN_POSITION - dada_transform.translation;
                let flat_to_spawn = Vec3::new(to_spawn.x, 0.0, to_spawn.z);

                if flat_to_spawn.length() <= 0.3 {
                    velocity.x = 0.0;
                    velocity.z = 0.0;
                    *enemy_state = EnemyState::Idle;

                    commands
                        .entity(dada_entity)
                        .remove::<EnemyInvestigateDirection>();

                    continue;
                }

                investigate.direction = flat_to_spawn.normalize();
                investigate.timer = Timer::from_seconds(10.0, TimerMode::Once);
            }

            let direction = investigate.direction;

            velocity.x = direction.x * DADA_MOVE_SPEED;
            velocity.z = direction.z * DADA_MOVE_SPEED;

            if direction.length_squared() > 0.0001 {
                dada_transform.rotation =
                    Quat::from_rotation_y(direction.x.atan2(direction.z));
            }

            *enemy_state = EnemyState::Chase;
            continue;
        }

        // 2.2 ไม่ได้โดนยิง และ Player อยู่ไกล -> ลาดตระเวนสุ่มแบบจำกัดพื้นที่
        if let Some(mut patrol) = patrol_data {
            patrol.timer.tick(time.delta());

            // ตัดแกน Y ออก ให้คิดแค่ระนาบ XZ
            let current_pos = Vec3::new(
                dada_transform.translation.x,
                0.0,
                dada_transform.translation.z,
            );

            let patrol_center = Vec3::new(
                patrol.center.x,
                0.0,
                patrol.center.z,
            );

            let from_center = current_pos - patrol_center;
            let distance_from_center = from_center.length();

            // ==========================================================
            // กรณีออกนอกพื้นที่แล้ว -> บังคับให้หันกลับเข้ากลางทันที
            // ==========================================================
            if distance_from_center > patrol.radius {
                let inward_direction = if from_center.length_squared() > 0.0001 {
                    (-from_center).normalize()
                } else {
                    Vec3::new(0.0, 0.0, 1.0)
                };

                patrol.direction = inward_direction;

                // ถ้ากำลังออกนอกเขต อาจยังไม่จำเป็นต้องสุ่มทิศใหม่ตอนนี้
                patrol.timer.reset();
            }
            // ==========================================================
            // กรณีอยู่ในพื้นที่ และครบเวลา -> สุ่มทิศใหม่
            // ==========================================================
            else if patrol.timer.just_finished() {
                let random_angle = rand::random::<f32>() * std::f32::consts::TAU;

                let mut new_direction = Vec3::new(
                    random_angle.cos(),
                    0.0,
                    random_angle.sin(),
                )
                .normalize();

                // ถ้าอยู่ใกล้ขอบพื้นที่แล้ว อย่าให้สุ่มทิศที่หันออกนอกเขต
                // เช่น อยู่ใกล้ขอบวงกลมแล้ว ถ้าทิศใหม่ชี้ออกนอกวงกลม ให้บังคับกลับเข้ากลาง
                if patrol.radius > 0.0 && distance_from_center > patrol.radius * 0.75 {
                    let outward_direction = from_center.normalize();

                    // dot > 0 แปลว่าทิศใหม่กำลังชี้ออกนอกศูนย์กลาง
                    if new_direction.dot(outward_direction) > 0.0 {
                        new_direction = -outward_direction;
                    }
                }

                patrol.direction = new_direction;
            }

            // ป้องกัน direction เป็น Vec3::ZERO
            let direction = if patrol.direction.length_squared() > 0.0001 {
                patrol.direction.normalize()
            } else {
                Vec3::new(0.0, 0.0, 1.0)
            };

            velocity.x = direction.x * DADA_MOVE_SPEED;
            velocity.z = direction.z * DADA_MOVE_SPEED;

            if direction.length_squared() > 0.0001 {
                dada_transform.rotation =
                    Quat::from_rotation_y(direction.x.atan2(direction.z));
            }

            *enemy_state = EnemyState::Patrol;
            continue;
        }
    }
}

fn update_dada_animation(
    animation_graph: Res<EnemyDadaAnimationGraph>,
    dada_query: Query<&EnemyState, With<EnemyDada>>,
    mut animation_query: Query<
        (
            &EnemyDadaAnimationTarget,
            &mut AnimationPlayer,
            &mut EnemyDadaAnimState,
        ),
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (
        animation_target,
        mut animation_player,
        mut current_animation,
    ) in &mut animation_query
    {
        let Ok(enemy_state) = dada_query.get(animation_target.root) else {
            continue;
        };

        let wanted_animation = match enemy_state {
            EnemyState::Hurt => EnemyDadaAnimState::Hurt,
            EnemyState::Dead => EnemyDadaAnimState::Dead,
            EnemyState::Chase | EnemyState::Patrol => EnemyDadaAnimState::Chase,
            EnemyState::Attack => EnemyDadaAnimState::Attack,
            _ => EnemyDadaAnimState::Idle,
        };

        // Animation เดิมกำลังเล่นอยู่ ไม่ต้องเริ่มใหม่ทุก Frame
        if *current_animation == wanted_animation {
            continue;
        }

        animation_player.stop_all();

        match wanted_animation {
            EnemyDadaAnimState::Idle => {
                animation_player
                    .play(animation_graph.idle)
                    .repeat();
            }
            EnemyDadaAnimState::Chase => {
                animation_player
                    .play(animation_graph.chase)
                    .repeat();
            }
            EnemyDadaAnimState::Attack => {
                animation_player
                    .play(animation_graph.attack)
                    .repeat();
            }
            EnemyDadaAnimState::Hurt => {
                animation_player.play(animation_graph.hurt);

                commands.spawn((AudioPlayer::new(asset_server.load("sounds/enemy/vn/dada_hurt.ogg")), PlaybackSettings::DESPAWN));
            }
            EnemyDadaAnimState::Dead => {
                animation_player.play(animation_graph.dead);

                commands.spawn((AudioPlayer::new(asset_server.load("sounds/enemy/vn/dada_dead.ogg")), PlaybackSettings::DESPAWN));
            }
        }

        *current_animation = wanted_animation;
    }
}

fn spawn_dada_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut dada_query: Query<
        (
            Entity,
            &EnemyState,
            &mut DadaAttackTimer,
        ),
        With<EnemyDada>,
    >,
) {
    for (
        dada_entity,
        enemy_state,
        mut attack_timer,
    ) in &mut dada_query
    {
        if !matches!(
            *enemy_state,
            EnemyState::Attack
        ) {
            attack_timer.0.reset();
            continue;
        }

        attack_timer.0.tick(time.delta());

        if !attack_timer.0.just_finished() {
            continue;
        }

        commands
            .entity(dada_entity)
            .with_children(|parent| {
                parent.spawn((
                    DadaPunchHitbox {
                        owner: dada_entity,
                        has_hit: false,
                        lifetime: Timer::from_seconds(
                            0.20,
                            TimerMode::Once,
                        ),
                    },
                    Collider::sphere(0.35),
                    Sensor,
                    CollisionEventsEnabled,
                    Transform::from_xyz(
                        0.0,
                        0.0,
                        0.85,
                    ),
                ));
            });
    }
}

fn dada_punch_hit_player(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut hitbox_query: Query<&mut DadaPunchHitbox>,
    dada_query: Query<
        (&CombatStats, &AtkAndDefElement),
        (With<EnemyDada>, Without<Player>),
    >,
    mut player_query: Query<
        (
            Entity,
            &mut Health,
            &CombatStats,
            &AtkAndDefElement,
            &GlobalTransform,
        ),
        (
            With<Player>,
            Without<EnemyDada>,
        ),
    >,
    anim_graph: Res<PlayerAnimationGraph>,
    mut anim_query: Query<
        (&mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
    asset_server: Res<AssetServer>,
    camera_query: Query<Entity, With<MainCamera>>,
) {
    let mut rng = rand::rng();

    for event in collision_reader.read() {
        let (
            hitbox_entity,
            other_body,
        ) = if hitbox_query.contains(
            event.collider1,
        ) {
            (
                event.collider1,
                event.body2
                    .unwrap_or(event.collider2),
            )
        } else if hitbox_query.contains(
            event.collider2,
        ) {
            (
                event.collider2,
                event.body1
                    .unwrap_or(event.collider1),
            )
        } else {
            continue;
        };

        let Ok(mut hitbox) = hitbox_query.get_mut(hitbox_entity) else {
            continue;
        };

        if hitbox.has_hit {
            continue;
        }

        let Ok((
            player_entity,
            mut player_health,
            player_stats,
            player_element,
            player_global_transform,
        )) = player_query.get_mut(other_body)
        else {
            continue;
        };

        let Ok((
            dada_stats,
            dada_element,
        )) = dada_query.get(hitbox.owner)
        else {
            continue;
        };

        let (
            base_damage,
            is_critical,
        ) = calculate_combat_damage(
            dada_stats,
            player_stats,
            &mut rng,
        );

        let element_multiplier = elemental_multiplier(
            dada_element.0,
            player_element.0,
        );

        let damage = (
            base_damage as f32
                * element_multiplier
        )
            .round()
            .max(1.0) as i32;

        player_health.current -= damage;
        player_health.current = player_health.current.clamp(0, player_health.max);

        if player_health.current > 0 {
            play_player_hurt_animation(
                &mut commands,
                player_entity,
                &anim_graph,
                &mut anim_query,
                &camera_query,
                &asset_server,
            );
        }

        spawn_floating_damage_text(
            &mut commands,
            damage,
            player_global_transform.translation()
                + Vec3::new(0.0, 2.0, 0.0),
            FloatingDamageKind::PlayerHit,
        );

        hitbox.has_hit = true;

        commands.spawn((AudioPlayer::new(asset_server.load("sounds/enemy/vn/dada_atk.ogg",)), PlaybackSettings::DESPAWN));

        info!(
            "Dada hit Player: damage={}, critical={}, HP={}/{}",
            damage,
            is_critical,
            player_health.current,
            player_health.max,
        );
    }
}

fn despawn_dada_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut hitbox_query: Query<
        (
            Entity,
            &mut DadaPunchHitbox,
        ),
    >,
) {
    for (
        entity,
        mut hitbox,
    ) in &mut hitbox_query
    {
        hitbox.lifetime.tick(time.delta());

        if hitbox.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn reset_enemy_dada_wave(
    mut spawned_count: ResMut<DadaSpawnedCount>,
    mut respawn_timer: ResMut<DadaRespawnTimer>,
    mut defeated_count: ResMut<DadaDefeatedCount>,
) {
    spawned_count.0 = 0;
    respawn_timer.0.reset();
    defeated_count.0 = 0;
}

fn update_dada_hurt_and_dead(
    mut commands: Commands,
    time: Res<Time>,
    mut dada_query: Query<
        (
            Entity,
            &mut EnemyState,
            &mut EnemyStateTimer,
            &mut LinearVelocity,
        ),
        With<EnemyDada>,
    >,
    mut defeated_count: ResMut<DadaDefeatedCount>,
) {
    for (
        dada_entity,
        mut enemy_state,
        mut state_timer,
        mut velocity,
    ) in &mut dada_query
    {
        state_timer.0.tick(time.delta());

        match *enemy_state {
            EnemyState::Hurt => {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
            EnemyState::Dead => {
                // ปล่อยให้กระเด็นนิดนึงก่อน
                if state_timer.0.elapsed_secs() >= 0.25 {
                    velocity.x = 0.0;
                    velocity.z = 0.0;
                }
            }
            _ => {}
        }

        if !state_timer.0.is_finished() {
            continue;
        }

        match *enemy_state {
            EnemyState::Hurt => {
                *enemy_state = EnemyState::Idle;

                commands
                    .entity(dada_entity)
                    .remove::<EnemyStateTimer>();
            }
            EnemyState::Dead => {
                // นับว่า Dada ถูก defeat 1 ตัว
                defeated_count.0 += 1;

                info!(
                    "Dada defeated: {}/{}",
                    defeated_count.0,
                    DADA_DEFEAT_GOAL
                );

                commands
                    .entity(dada_entity)
                    .despawn();
            }
            _ => {
                commands
                    .entity(dada_entity)
                    .remove::<EnemyStateTimer>();
            }
        }
    }
}