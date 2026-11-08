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
pub struct EnemyJook;

pub struct FiJookPlugin;

impl Plugin for FiJookPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(JookRespawnTimer(Timer::from_seconds(
            JOOK_RESPAWN_DELAY,
            TimerMode::Once,
        )))
        .add_systems(
            OnEnter(GameScene::FloatingIsland),
            (
                reset_enemy_jook_wave,
                setup_enemy_jook_animation_graph,
            ),
        )
        .add_systems(
            Update,
            (
                spawn_enemy_jook,
                setup_enemy_jook_animation_player,
                enemy_jook_chase_player,
                update_enemy_jook_animation,
                update_jook_hurt_and_dead,
                spawn_jook_punch_hitbox,
                jook_punch_hit_player,
                despawn_jook_punch_hitbox,
            )
                .chain()
                .run_if(in_state(GameScene::FloatingIsland).and(in_state(GameMode::Playing))),
        );
    }
}

const JOOK_SPAWN_POSITION: Vec3 = Vec3::new(8.98, 23.68, -13.25);
const JOOK_BODY_Y: f32 = 1.0;

// spawn ได้เรื่อย ๆ แต่ ณ ช่วงเวลาหนึ่งมีได้สูงสุด 1 ตัว
const JOOK_MAX_ALIVE: usize = 1;

const JOOK_CHASE_RANGE: f32 = 10.0;
const JOOK_STOP_DISTANCE: f32 = 1.0;
const JOOK_MOVE_SPEED: f32 = 3.0;
const JOOK_PATROL_RADIUS: f32 = 10.0;

// ถ้าอยากให้ spawn ทันทีหลังตัวเก่าตาย/เข้าฉาก ให้แก้เป็น 0.0
const JOOK_RESPAWN_DELAY: f32 = 1.0;

const JOOK_ATTACK_INTERVAL: f32 = 0.3;
const JOOK_PUNCH_HITBOX_LIFETIME: f32 = 0.20;

fn spawn_enemy_jook(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut respawn_timer: ResMut<JookRespawnTimer>,
    jook_query: Query<(), With<EnemyJook>>,
) {
    let alive_count = jook_query.iter().count();

    // ถ้ายังมี Jook อยู่ครบจำนวนที่กำหนด ให้รอตัวปัจจุบันตาย/ถูก despawn ก่อน
    if alive_count >= JOOK_MAX_ALIVE {
        respawn_timer.0.reset();
        return;
    }

    // ถ้าไม่มี Jook แล้ว ให้นับเวลา respawn
    respawn_timer.0.tick(time.delta());

    if !respawn_timer.0.just_finished() {
        return;
    }

    let jook_scene = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("enemy/Fi_Jook.glb"),
    );

    let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
    let initial_dir = Vec3::new(random_angle.cos(), 0.0, random_angle.sin()).normalize();

    let base_stats = BaseStats::JOOK;

    let enemy_jook = commands
        .spawn((
            Name::new("Enemy Jook"),
            Enemy,
            EnemyJook,
            EnemyState::Patrol,
            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },
            base_stats,
            CombatStats::from(base_stats),
            AtkAndDefElement(Element::Earth),
            ElementExpReward::JOOK,
            RigidBody::Dynamic,
            Collider::capsule(0.45, 1.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            Transform::from_translation(JOOK_SPAWN_POSITION + Vec3::Y * JOOK_BODY_Y),
            DespawnOnExit(GameScene::FloatingIsland),
        ))
        .insert(JookPatrol {
            direction: initial_dir,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            center: JOOK_SPAWN_POSITION,
            radius: JOOK_PATROL_RADIUS,
        })
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(jook_scene),
                Transform::from_xyz(0.0, -JOOK_BODY_Y, 0.0),
                ApplyToonMaterial,
            ));
        })
        .id();

    commands.entity(enemy_jook).insert((
        CombatTarget,
        JookAttackTimer(Timer::from_seconds(
            JOOK_ATTACK_INTERVAL,
            TimerMode::Repeating,
        )),
    ));

    spawn_enemy_health_bar(
        &mut commands,
        enemy_jook,
    );

    respawn_timer.0.reset();
}

fn setup_enemy_jook_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(3)
                .from_asset("enemy/Fi_Jook.glb"),
        ),
        1.0,
        graph.root,
    );

    let chase = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(4)
                .from_asset("enemy/Fi_Jook.glb"),
        ),
        1.0,
        graph.root,
    );

    let attack = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(0)
                .from_asset("enemy/Fi_Jook.glb"),
        ),
        1.0,
        graph.root,
    );

    let hurt = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(2)
                .from_asset("enemy/Fi_Jook.glb"),
        ),
        1.0,
        graph.root,
    );

    let dead = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(1)
                .from_asset("enemy/Fi_Jook.glb"),
        ),
        1.0,
        graph.root,
    );

    commands.insert_resource(EnemyJookAnimationGraph {
        graph: graphs.add(graph),
        idle,
        chase,
        attack,
        hurt,
        dead,
    });
}

fn setup_enemy_jook_animation_player(
    mut commands: Commands,
    animation_graph: Res<EnemyJookAnimationGraph>,
    mut animation_players: Query<
        (Entity, &mut AnimationPlayer),
        Added<AnimationPlayer>,
    >,
    child_of_query: Query<&ChildOf>,
    jook_query: Query<(), With<EnemyJook>>,
) {
    for (animation_entity, mut player) in &mut animation_players {
        let Some(jook_root) = find_enemy_jook_root(
            animation_entity,
            &child_of_query,
            &jook_query,
        ) else {
            continue;
        };

        commands.entity(animation_entity).insert((
            AnimationGraphHandle(animation_graph.graph.clone()),
            EnemyJookAnimationTarget {
                root: jook_root,
            },
            EnemyJookAnimState::Idle,
        ));

        player.stop_all();
        player.play(animation_graph.idle).repeat();
    }
}

fn find_enemy_jook_root(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    jook_query: &Query<(), With<EnemyJook>>,
) -> Option<Entity> {
    let mut current = entity;

    loop {
        if jook_query.get(current).is_ok() {
            return Some(current);
        }

        let Ok(child_of) = child_of_query.get(current) else {
            return None;
        };

        current = child_of.parent();
    }
}

fn enemy_jook_chase_player(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<
        &Transform,
        (
            With<Player>,
            Without<EnemyJook>,
        ),
    >,
    mut jook_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut EnemyState,
            Option<&mut EnemyInvestigateDirection>,
            Option<&mut JookPatrol>,
            Option<&CriticalKnockback>,
        ),
        (
            With<EnemyJook>,
            Without<Player>,
        ),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (
        jook_entity,
        mut jook_transform,
        mut velocity,
        mut enemy_state,
        investigate,
        patrol_data,
        critical_knockback,
    ) in &mut jook_query
    {
        // Hurt / Dead ยังหยุดเหมือนเดิม
        if matches!(*enemy_state, EnemyState::Hurt | EnemyState::Dead) {
            if critical_knockback.is_none() {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
            continue;
        }

        let to_player = player_transform.translation - jook_transform.translation;
        let flat_direction = Vec3::new(to_player.x, 0.0, to_player.z);
        let distance = flat_direction.length();

        // ==========================================
        // 1. Player อยู่ในระยะมองเห็น (<= 10.0) -> ไล่ตามทันที
        // ==========================================
        if distance <= JOOK_CHASE_RANGE {
            commands.entity(jook_entity).remove::<EnemyInvestigateDirection>();

            // Player อยู่ในระยะโจมตี
            if distance <= JOOK_STOP_DISTANCE {
                velocity.x = 0.0;
                velocity.z = 0.0;

                if flat_direction.length_squared() > 0.0001 {
                    let direction = flat_direction.normalize();
                    jook_transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
                }

                *enemy_state = EnemyState::Attack;
                continue;
            }

            // ไล่ Player ตามปกติ
            let direction = flat_direction.normalize();
            velocity.x = direction.x * JOOK_MOVE_SPEED;
            velocity.z = direction.z * JOOK_MOVE_SPEED;
            jook_transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
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
                let to_spawn = JOOK_SPAWN_POSITION - jook_transform.translation;
                let flat_to_spawn = Vec3::new(to_spawn.x, 0.0, to_spawn.z);

                if flat_to_spawn.length() <= 0.3 {
                    velocity.x = 0.0;
                    velocity.z = 0.0;
                    *enemy_state = EnemyState::Idle;
                    commands.entity(jook_entity).remove::<EnemyInvestigateDirection>();
                    continue;
                }

                investigate.direction = flat_to_spawn.normalize();
                investigate.timer = Timer::from_seconds(10.0, TimerMode::Once);
            }

            let direction = investigate.direction;
            velocity.x = direction.x * JOOK_MOVE_SPEED;
            velocity.z = direction.z * JOOK_MOVE_SPEED;

            if direction.length_squared() > 0.0001 {
                jook_transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            }

            *enemy_state = EnemyState::Chase;
            continue;
        }

        // 2.2 ไม่ได้โดนยิง และ Player อยู่ไกล -> ลาดตระเวนสุ่มแบบจำกัดพื้นที่
        if let Some(mut patrol) = patrol_data {
            patrol.timer.tick(time.delta());

            // ตัดแกน Y ออก ให้คิดแค่ระนาบ XZ
            let current_pos = Vec3::new(
                jook_transform.translation.x,
                0.0,
                jook_transform.translation.z,
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

            velocity.x = direction.x * JOOK_MOVE_SPEED;
            velocity.z = direction.z * JOOK_MOVE_SPEED;

            if direction.length_squared() > 0.0001 {
                jook_transform.rotation =
                    Quat::from_rotation_y(direction.x.atan2(direction.z));
            }

            *enemy_state = EnemyState::Patrol;
            continue;
        }
    }
}

fn update_enemy_jook_animation(
    animation_graph: Res<EnemyJookAnimationGraph>,
    jook_query: Query<&EnemyState, With<EnemyJook>>,
    mut animation_query: Query<
        (
            &EnemyJookAnimationTarget,
            &mut AnimationPlayer,
            &mut EnemyJookAnimState,
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
            jook_query.get(animation_target.root)
        else {
            continue;
        };

        let wanted_animation = match enemy_state {
            EnemyState::Hurt => EnemyJookAnimState::Hurt,
            EnemyState::Dead => EnemyJookAnimState::Dead,
            EnemyState::Chase | EnemyState::Patrol => EnemyJookAnimState::Chase,
            EnemyState::Attack => EnemyJookAnimState::Attack,
            _ => EnemyJookAnimState::Idle,
        };

        // Animation เดิมกำลังเล่นอยู่ ไม่ต้องเริ่มใหม่ทุก frame
        if *current_animation == wanted_animation {
            continue;
        }

        animation_player.stop_all();

        match wanted_animation {
            EnemyJookAnimState::Idle => {
                animation_player
                    .play(animation_graph.idle)
                    .repeat();
            }
            EnemyJookAnimState::Chase => {
                animation_player
                    .play(animation_graph.chase)
                    .repeat();
            }
            EnemyJookAnimState::Attack => {
                animation_player
                    .play(animation_graph.attack)
                    .repeat();
            }
            EnemyJookAnimState::Hurt => {
                animation_player.play(animation_graph.hurt);
                commands.spawn((
                    AudioPlayer::new(asset_server.load("sounds/enemy/jook_hurt.ogg")),
                    PlaybackSettings::DESPAWN,
                ));
            }
            EnemyJookAnimState::Dead => {
                animation_player.play(animation_graph.dead);
                commands.spawn((
                    AudioPlayer::new(asset_server.load("sounds/enemy/jook_dead.ogg")),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }

        *current_animation = wanted_animation;
    }
}

fn spawn_jook_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut jook_query: Query<
        (
            Entity,
            &EnemyState,
            &mut JookAttackTimer,
        ),
        With<EnemyJook>,
    >,
) {
    for (
        jook_entity,
        enemy_state,
        mut attack_timer,
    ) in &mut jook_query
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
            .entity(jook_entity)
            .with_children(|parent| {
                parent.spawn((
                    JookPunchHitbox {
                        owner: jook_entity,
                        has_hit: false,
                        lifetime: Timer::from_seconds(
                            JOOK_PUNCH_HITBOX_LIFETIME,
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

fn jook_punch_hit_player(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut hitbox_query: Query<&mut JookPunchHitbox>,
    jook_query: Query<
        (&CombatStats, &AtkAndDefElement),
        (
            With<EnemyJook>,
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
            Without<EnemyJook>,
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
            jook_stats,
            jook_element,
        )) = jook_query.get(hitbox.owner)
        else {
            continue;
        };

        let (
            base_damage,
            is_critical,
        ) = calculate_combat_damage(
            jook_stats,
            player_stats,
            &mut rng,
        );

        let element_multiplier =
            elemental_multiplier(
                jook_element.0,
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
            AudioPlayer::new(asset_server.load("sounds/enemy/jook_atk.ogg")),
            PlaybackSettings::DESPAWN,
        ));

        info!(
            "Jook hit Player: damage={}, critical={}, HP={}/{}",
            damage,
            is_critical,
            player_health.current,
            player_health.max,
        );
    }
}

fn despawn_jook_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut hitbox_query: Query<
        (
            Entity,
            &mut JookPunchHitbox,
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

fn reset_enemy_jook_wave(
    mut respawn_timer: ResMut<JookRespawnTimer>,
) {
    respawn_timer.0.reset();
}

fn update_jook_hurt_and_dead(
    mut commands: Commands,
    time: Res<Time>,
    mut jook_query: Query<
        (
            Entity,
            &mut EnemyState,
            Option<&mut EnemyStateTimer>,
            &mut LinearVelocity,
            Option<&mut CriticalKnockback>,
        ),
        With<EnemyJook>,
    >,
) {
    for (
        jook_entity,
        mut enemy_state,
        state_timer,
        mut velocity,
        mut critical_knockback,
    ) in &mut jook_query
    {
        // ถ้ากำลัง Hurt/Dead แต่ไม่มี timer ให้สร้าง timer ใหม่
        let Some(mut state_timer) = state_timer else {
            if matches!(*enemy_state, EnemyState::Hurt | EnemyState::Dead) {
                let duration = if matches!(*enemy_state, EnemyState::Dead) {
                    1.0
                } else {
                    0.25
                };

                commands.entity(jook_entity).insert(
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
                    .entity(jook_entity)
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
                    .entity(jook_entity)
                    .remove::<EnemyStateTimer>();
            }
            EnemyState::Dead => {
                info!("Jook defeated");
                commands.entity(jook_entity).despawn();
            }
            _ => {
                commands
                    .entity(jook_entity)
                    .remove::<EnemyStateTimer>();
            }
        }
    }
}