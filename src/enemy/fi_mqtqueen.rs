use avian3d::prelude::*;
use bevy::{
    gltf::GltfAssetLabel,
    prelude::*,
};

use crate::camera::*;
use crate::cel_shader::*;
use crate::components::*;
use crate::npc::practice_common::*;
use crate::pause_menu::GameMode;
use crate::player::*;

#[derive(Component, Debug)]
pub struct EnemyMqtqueen;

pub struct FiMqtqueenPlugin;

impl Plugin for FiMqtqueenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MqtqueenRespawnTimer(Timer::from_seconds(
            MQTQUEEN_RESPAWN_DELAY,
            TimerMode::Once,
        )))
        .add_systems(
            OnEnter(GameScene::FloatingIsland),
            (
                reset_enemy_mqtqueen_wave,
                setup_enemy_mqtqueen_animation_graph,
            ),
        )
        .add_systems(
            Update,
            (
                spawn_enemy_mqtqueen,
                setup_enemy_mqtqueen_animation_player,
                enemy_mqtqueen_chase_player,
                update_enemy_mqtqueen_animation,
                update_mqtqueen_hurt_and_dead,
                spawn_mqtqueen_punch_hitbox,
                mqtqueen_punch_hit_player,
                despawn_mqtqueen_punch_hitbox,
            )
                .chain()
                .run_if(in_state(GameScene::FloatingIsland).and(in_state(GameMode::Playing))),
        );
    }
}

const MQTQUEEN_SPAWN_POSITION: Vec3 = Vec3::new(41.08, 37.00, -23.28);
const MQTQUEEN_BODY_Y: f32 = 1.0;

// spawn ได้เรื่อย ๆ แต่ ณ ช่วงเวลาหนึ่งมีได้สูงสุด 1 ตัว
const MQTQUEEN_MAX_ALIVE: usize = 1;
const MQTQUEEN_CHASE_RANGE: f32 = 10.0;
const MQTQUEEN_STOP_DISTANCE: f32 = 1.0;
const MQTQUEEN_MOVE_SPEED: f32 = 3.0;
const MQTQUEEN_PATROL_RADIUS: f32 = 10.0;

// ถ้าอยากให้ spawn ทันทีหลังตัวเก่าตาย/เข้าฉาก ให้แก้เป็น 0.0
const MQTQUEEN_RESPAWN_DELAY: f32 = 1.0;
const MQTQUEEN_ATTACK_INTERVAL: f32 = 0.3;
const MQTQUEEN_PUNCH_HITBOX_LIFETIME: f32 = 0.20;

fn spawn_enemy_mqtqueen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut respawn_timer: ResMut<MqtqueenRespawnTimer>,
    mqtqueen_query: Query<(), With<EnemyMqtqueen>>,
) {
    let alive_count = mqtqueen_query.iter().count();

    // ถ้ายังมี Mqtqueen อยู่ครบจำนวนที่กำหนด ให้รอตัวปัจจุบันตาย/ถูก despawn ก่อน
    if alive_count >= MQTQUEEN_MAX_ALIVE {
        respawn_timer.0.reset();
        return;
    }

    // ถ้าไม่มี Mqtqueen แล้ว ให้นับเวลา respawn
    respawn_timer.0.tick(time.delta());
    if !respawn_timer.0.just_finished() {
        return;
    }

    let mqtqueen_scene = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("enemy/Fi_MqtQueen.glb"),
    );

    let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
    let initial_dir = Vec3::new(random_angle.cos(), 0.0, random_angle.sin()).normalize();

    let base_stats = BaseStats::MQTQUEEN;

    let enemy_mqtqueen = commands
        .spawn((
            Name::new("Enemy MqtQueen"),
            Enemy,
            EnemyMqtqueen,
            EnemyState::Patrol,
            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },
            base_stats,
            CombatStats::from(base_stats),
            AtkAndDefElement(Element::Earth),
            ElementExpReward::MQTQUEEN,
            RigidBody::Dynamic,
            Collider::capsule(0.45, 1.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            Transform::from_translation(MQTQUEEN_SPAWN_POSITION + Vec3::Y * MQTQUEEN_BODY_Y),
            DespawnOnExit(GameScene::FloatingIsland),
        ))
        .insert(MqtqueenPatrol {
            direction: initial_dir,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            center: MQTQUEEN_SPAWN_POSITION,
            radius: MQTQUEEN_PATROL_RADIUS,
        })
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(mqtqueen_scene),
                Transform::from_xyz(0.0, -MQTQUEEN_BODY_Y, 0.0),
                ApplyToonMaterial,
            ));
        })
        .id();

    commands.entity(enemy_mqtqueen).insert((
        CombatTarget,
        MqtqueenAttackTimer(Timer::from_seconds(
            MQTQUEEN_ATTACK_INTERVAL,
            TimerMode::Repeating,
        )),
    ));

    spawn_enemy_health_bar(
        &mut commands,
        enemy_mqtqueen,
    );

    respawn_timer.0.reset();
}

fn setup_enemy_mqtqueen_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(3).from_asset("enemy/Fi_MqtQueen.glb"),
        ),
        1.0,
        graph.root,
    );

    let chase = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(4).from_asset("enemy/Fi_MqtQueen.glb"),
        ),
        1.0,
        graph.root,
    );

    let attack = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(0).from_asset("enemy/Fi_MqtQueen.glb"),
        ),
        1.0,
        graph.root,
    );

    let hurt = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(2).from_asset("enemy/Fi_MqtQueen.glb"),
        ),
        1.0,
        graph.root,
    );

    let dead = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(1).from_asset("enemy/Fi_MqtQueen.glb"),
        ),
        1.0,
        graph.root,
    );

    commands.insert_resource(EnemyMqtqueenAnimationGraph {
        graph: graphs.add(graph),
        idle,
        chase,
        attack,
        hurt,
        dead,
    });
}

fn setup_enemy_mqtqueen_animation_player(
    mut commands: Commands,
    animation_graph: Res<EnemyMqtqueenAnimationGraph>,
    mut animation_players: Query<
        (Entity, &mut AnimationPlayer),
        Added<AnimationPlayer>,
    >,
    child_of_query: Query<&ChildOf>,
    mqtqueen_query: Query<(), With<EnemyMqtqueen>>,
) {
    for (animation_entity, mut player) in &mut animation_players {
        let Some(mqtqueen_root) = find_enemy_mqtqueen_root(
            animation_entity,
            &child_of_query,
            &mqtqueen_query,
        ) else {
            continue;
        };

        commands.entity(animation_entity).insert((
            AnimationGraphHandle(animation_graph.graph.clone()),
            EnemyMqtqueenAnimationTarget {
                root: mqtqueen_root,
            },
            EnemyMqtqueenAnimState::Idle,
        ));

        player.stop_all();
        player.play(animation_graph.idle).repeat();
    }
}

fn find_enemy_mqtqueen_root(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    mqtqueen_query: &Query<(), With<EnemyMqtqueen>>,
) -> Option<Entity> {
    let mut current = entity;

    loop {
        if mqtqueen_query.get(current).is_ok() {
            return Some(current);
        }

        let Ok(child_of) = child_of_query.get(current) else {
            return None;
        };

        current = child_of.parent();
    }
}

fn enemy_mqtqueen_chase_player(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<
        &Transform,
        (
            With<Player>,
            Without<EnemyMqtqueen>,
        ),
    >,
    mut mqtqueen_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut EnemyState,
            Option<&mut EnemyInvestigateDirection>,
            Option<&mut MqtqueenPatrol>,
            Option<&CriticalKnockback>,
        ),
        (
            With<EnemyMqtqueen>,
            Without<Player>,
        ),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (
        mqtqueen_entity,
        mut mqtqueen_transform,
        mut velocity,
        mut enemy_state,
        investigate,
        patrol_data,
        critical_knockback,
    ) in &mut mqtqueen_query
    {
        // Hurt / Dead ยังหยุดเหมือนเดิม
        if matches!(*enemy_state, EnemyState::Hurt | EnemyState::Dead) {
            if critical_knockback.is_none() {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
            continue;
        }

        let to_player = player_transform.translation - mqtqueen_transform.translation;
        let flat_direction = Vec3::new(to_player.x, 0.0, to_player.z);
        let distance = flat_direction.length();

        // ==========================================
        // เช็คความสูงของ Player ก่อน
        // จะไล่ล่าเฉพาะเมื่อ Player อยู่ในช่วง 34.4 < y < 40.0
        // ==========================================
        let player_y = player_transform.translation.y;
        let player_in_chase_height = player_y > 34.4 && player_y < 40.0;

        // ==========================================
        // 1. Player อยู่ในระยะมองเห็น (<= 10.0)
        // และอยู่ในช่วงความสูงที่กำหนด -> ไล่ตาม
        // ==========================================
        if player_in_chase_height && distance <= MQTQUEEN_CHASE_RANGE {
            commands.entity(mqtqueen_entity).remove::<EnemyInvestigateDirection>();

            // Player อยู่ในระยะโจมตี
            if distance <= MQTQUEEN_STOP_DISTANCE {
                velocity.x = 0.0;
                velocity.z = 0.0;

                if flat_direction.length_squared() > 0.0001 {
                    let direction = flat_direction.normalize();
                    mqtqueen_transform.rotation =
                        Quat::from_rotation_y(direction.x.atan2(direction.z));
                }

                *enemy_state = EnemyState::Attack;
                continue;
            }

            // ไล่ Player ตามปกติ
            let direction = flat_direction.normalize();
            velocity.x = direction.x * MQTQUEEN_MOVE_SPEED;
            velocity.z = direction.z * MQTQUEEN_MOVE_SPEED;

            mqtqueen_transform.rotation =
                Quat::from_rotation_y(direction.x.atan2(direction.z));

            *enemy_state = EnemyState::Chase;
            continue;
        }

        // ==========================================
        // 2. Player อยู่นอกระยะ (> 10.0)
        // หรืออยู่นอกช่วงความสูง -> ไม่ไล่ตาม
        // ==========================================

        // 2.1 โดนยิงมา (มี Investigate) -> เดินกลับไปจุดเกิด
        if let Some(mut investigate) = investigate {
            investigate.timer.tick(time.delta());

            if investigate.timer.is_finished() {
                let to_spawn = MQTQUEEN_SPAWN_POSITION - mqtqueen_transform.translation;
                let flat_to_spawn = Vec3::new(to_spawn.x, 0.0, to_spawn.z);

                if flat_to_spawn.length() <= 0.3 {
                    velocity.x = 0.0;
                    velocity.z = 0.0;
                    *enemy_state = EnemyState::Idle;
                    commands.entity(mqtqueen_entity).remove::<EnemyInvestigateDirection>();
                    continue;
                }

                investigate.direction = flat_to_spawn.normalize();
                investigate.timer = Timer::from_seconds(10.0, TimerMode::Once);
            }

            let direction = investigate.direction;
            velocity.x = direction.x * MQTQUEEN_MOVE_SPEED;
            velocity.z = direction.z * MQTQUEEN_MOVE_SPEED;

            if direction.length_squared() > 0.0001 {
                mqtqueen_transform.rotation =
                    Quat::from_rotation_y(direction.x.atan2(direction.z));
            }

            *enemy_state = EnemyState::Chase;
            continue;
        }

        // 2.2 ไม่ได้โดนยิง และ Player อยู่ไกล -> ลาดตระเวนสุ่มแบบจำกัดพื้นที่
        if let Some(mut patrol) = patrol_data {
            patrol.timer.tick(time.delta());

            let current_pos = Vec3::new(
                mqtqueen_transform.translation.x,
                0.0,
                mqtqueen_transform.translation.z,
            );

            let patrol_center = Vec3::new(
                patrol.center.x,
                0.0,
                patrol.center.z,
            );

            let from_center = current_pos - patrol_center;
            let distance_from_center = from_center.length();

            if distance_from_center > patrol.radius {
                let inward_direction = if from_center.length_squared() > 0.0001 {
                    (-from_center).normalize()
                } else {
                    Vec3::new(0.0, 0.0, 1.0)
                };

                patrol.direction = inward_direction;
                patrol.timer.reset();
            } else if patrol.timer.just_finished() {
                let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
                let mut new_direction =
                    Vec3::new(random_angle.cos(), 0.0, random_angle.sin()).normalize();

                if patrol.radius > 0.0 && distance_from_center > patrol.radius * 0.75 {
                    let outward_direction = from_center.normalize();

                    if new_direction.dot(outward_direction) > 0.0 {
                        new_direction = -outward_direction;
                    }
                }

                patrol.direction = new_direction;
            }

            let direction = if patrol.direction.length_squared() > 0.0001 {
                patrol.direction.normalize()
            } else {
                Vec3::new(0.0, 0.0, 1.0)
            };

            velocity.x = direction.x * MQTQUEEN_MOVE_SPEED;
            velocity.z = direction.z * MQTQUEEN_MOVE_SPEED;

            if direction.length_squared() > 0.0001 {
                mqtqueen_transform.rotation =
                    Quat::from_rotation_y(direction.x.atan2(direction.z));
            }

            *enemy_state = EnemyState::Patrol;
            continue;
        }
    }
}

fn update_enemy_mqtqueen_animation(
    animation_graph: Res<EnemyMqtqueenAnimationGraph>,
    mqtqueen_query: Query<&EnemyState, With<EnemyMqtqueen>>,
    mut animation_query: Query<
        (
            &EnemyMqtqueenAnimationTarget,
            &mut AnimationPlayer,
            &mut EnemyMqtqueenAnimState,
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
        let Ok(enemy_state) =
            mqtqueen_query.get(animation_target.root)
        else {
            continue;
        };

        let wanted_animation = match enemy_state {
            EnemyState::Hurt => EnemyMqtqueenAnimState::Hurt,
            EnemyState::Dead => EnemyMqtqueenAnimState::Dead,
            EnemyState::Chase | EnemyState::Patrol => EnemyMqtqueenAnimState::Chase,
            EnemyState::Attack => EnemyMqtqueenAnimState::Attack,
            _ => EnemyMqtqueenAnimState::Idle,
        };

        // Animation เดิมกำลังเล่นอยู่ ไม่ต้องเริ่มใหม่ทุก frame
        if *current_animation == wanted_animation {
            continue;
        }

        animation_player.stop_all();

        match wanted_animation {
            EnemyMqtqueenAnimState::Idle => {
                animation_player
                    .play(animation_graph.idle)
                    .repeat();
            }
            EnemyMqtqueenAnimState::Chase => {
                animation_player
                    .play(animation_graph.chase)
                    .repeat();
            }
            EnemyMqtqueenAnimState::Attack => {
                animation_player
                    .play(animation_graph.attack)
                    .repeat();
            }
            EnemyMqtqueenAnimState::Hurt => {
                animation_player.play(animation_graph.hurt);

                commands.spawn((
                    AudioPlayer::new(asset_server.load("sounds/enemy/mqtqueen_hurt.ogg")),
                    PlaybackSettings::DESPAWN,
                ));
            }
            EnemyMqtqueenAnimState::Dead => {
                animation_player.play(animation_graph.dead);

                commands.spawn((
                    AudioPlayer::new(asset_server.load("sounds/enemy/mqtqueen_dead.ogg")),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }

        *current_animation = wanted_animation;
    }
}

fn spawn_mqtqueen_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut mqtqueen_query: Query<
        (
            Entity,
            &EnemyState,
            &mut MqtqueenAttackTimer,
        ),
        With<EnemyMqtqueen>,
    >,
) {
    for (
        mqtqueen_entity,
        enemy_state,
        mut attack_timer,
    ) in &mut mqtqueen_query
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
            .entity(mqtqueen_entity)
            .with_children(|parent| {
                parent.spawn((
                    MqtqueenPunchHitbox {
                        owner: mqtqueen_entity,
                        has_hit: false,
                        lifetime: Timer::from_seconds(
                            MQTQUEEN_PUNCH_HITBOX_LIFETIME,
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

fn mqtqueen_punch_hit_player(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut hitbox_query: Query<&mut MqtqueenPunchHitbox>,
    mqtqueen_query: Query<
        (&CombatStats, &AtkAndDefElement),
        (
            With<EnemyMqtqueen>,
            Without<Player>,
        ),
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
            Without<EnemyMqtqueen>,
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

        let Ok(mut hitbox) =
            hitbox_query.get_mut(hitbox_entity)
        else {
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
            mqtqueen_stats,
            mqtqueen_element,
        )) = mqtqueen_query.get(hitbox.owner)
        else {
            continue;
        };

        let (
            base_damage,
            is_critical,
        ) = calculate_combat_damage(
            mqtqueen_stats,
            player_stats,
            &mut rng,
        );

        let element_multiplier =
            elemental_multiplier(
                mqtqueen_element.0,
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

        commands.spawn((
            AudioPlayer::new(asset_server.load("sounds/enemy/mqtqueen_atk.ogg")),
            PlaybackSettings::DESPAWN,
        ));

        info!(
            "Mqtqueen hit Player: damage={}, critical={}, HP={}/{}",
            damage,
            is_critical,
            player_health.current,
            player_health.max,
        );
    }
}

fn despawn_mqtqueen_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut hitbox_query: Query<
        (
            Entity,
            &mut MqtqueenPunchHitbox,
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

fn reset_enemy_mqtqueen_wave(
    mut respawn_timer: ResMut<MqtqueenRespawnTimer>,
) {
    respawn_timer.0.reset();
}

fn update_mqtqueen_hurt_and_dead(
    mut commands: Commands,
    time: Res<Time>,
    mut mqtqueen_query: Query<
        (
            Entity,
            &mut EnemyState,
            Option<&mut EnemyStateTimer>,
            &mut LinearVelocity,
            Option<&mut CriticalKnockback>,
        ),
        With<EnemyMqtqueen>,
    >,
) {
    for (
        mqtqueen_entity,
        mut enemy_state,
        state_timer,
        mut velocity,
        mut critical_knockback,
    ) in &mut mqtqueen_query
    {
        // ถ้ากำลัง Hurt/Dead แต่ไม่มี timer ให้สร้าง timer ใหม่
        let Some(mut state_timer) = state_timer else {
            if matches!(*enemy_state, EnemyState::Hurt | EnemyState::Dead) {
                let duration = if matches!(*enemy_state, EnemyState::Dead) {
                    1.0
                } else {
                    0.25
                };

                commands.entity(mqtqueen_entity).insert(
                    EnemyStateTimer(Timer::from_seconds(duration, TimerMode::Once)),
                );
            }
            continue;
        };

        state_timer.0.tick(time.delta());

        let mut allow_knockback = false;

        if let Some(mut kb) = critical_knockback {
            kb.timer.tick(time.delta());

            if kb.timer.just_finished() {
                commands
                    .entity(mqtqueen_entity)
                    .remove::<CriticalKnockback>();
            } else {
                allow_knockback = true;
            }
        }

        match *enemy_state {
            EnemyState::Hurt => {
                if !allow_knockback {
                    velocity.x = 0.0;
                    velocity.z = 0.0;
                }
            }
            EnemyState::Dead => {
                if state_timer.0.elapsed_secs() >= 0.25 && !allow_knockback {
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
                    .entity(mqtqueen_entity)
                    .remove::<EnemyStateTimer>();
            }
            EnemyState::Dead => {
                info!("Mqtqueen defeated");
                commands.entity(mqtqueen_entity).despawn();
            }
            _ => {
                commands
                    .entity(mqtqueen_entity)
                    .remove::<EnemyStateTimer>();
            }
        }
    }
}