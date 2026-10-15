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
pub struct EnemyJaar;

pub struct FiJaarPlugin;

impl Plugin for FiJaarPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(JaarRespawnTimer(Timer::from_seconds(1.0, TimerMode::Once)))
            .init_resource::<JaarSpawnedCount>()
            .add_systems(
                OnEnter(GameScene::FloatingIsland),
                (
                    reset_enemy_jaar_wave,
                    setup_enemy_jaar_animation_graph,
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_enemy_jaar,
                    setup_enemy_jaar_animation_player,
                    provoke_enemy_jaar_when_hurt,
                    enemy_jaar_behavior,
                    update_enemy_jaar_animation,
                    update_jaar_hurt_and_dead,
                    spawn_jaar_punch_hitbox,
                    jaar_punch_hit_player,
                    despawn_jaar_punch_hitbox,
                )
                    .chain()
                    .run_if(in_state(GameScene::FloatingIsland).and(in_state(GameMode::Playing))),
            );
    }
}

const JAAR_SPAWN_POSITION: Vec3 = Vec3::new(0.0, 1.0, 0.0);
const JAAR_STOP_DISTANCE: f32 = 1.0;
const JAAR_MOVE_SPEED: f32 = 3.0;
const JAAR_LOSE_INTEREST_DISTANCE: f32 = 10.0;
const JAAR_PATROL_RADIUS: f32 = 8.0;

fn spawn_enemy_jaar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawned_count: ResMut<JaarSpawnedCount>,
) {
    if spawned_count.0 >= JAAR_SPAWN_LIMIT {
        return;
    }

    let jaar_scene = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("enemy/Fi_Jaar.glb"),
    );

    let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
    let initial_dir = Vec3::new(random_angle.cos(), 0.0, random_angle.sin()).normalize();

    const JAAR_BODY_Y: f32 = 1.0;

    let base_stats = BaseStats::JAAR;

    let enemy_jaar = commands
        .spawn((
            Name::new("Enemy Jaar"),
            Enemy,
            EnemyJaar,
            EnemyState::Patrol,
            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },
            base_stats,
            CombatStats::from(base_stats),
            AtkAndDefElement(Element::Wind),
            ElementExpReward::JAAR,
            RigidBody::Dynamic,
            Collider::capsule(0.45, 1.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            Transform::from_translation(JAAR_SPAWN_POSITION + Vec3::Y * JAAR_BODY_Y),
            DespawnOnExit(GameScene::FloatingIsland),
        ))
        .insert(PassivePatrol {
            direction: initial_dir,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            center: JAAR_SPAWN_POSITION,
            radius: JAAR_PATROL_RADIUS,
        })
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(jaar_scene),
                Transform::from_xyz(0.0, -JAAR_BODY_Y, 0.0),
                ApplyToonMaterial,
            ));
        })
        .id();

    commands.entity(enemy_jaar).insert((
        CombatTarget,
        JaarAttackTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
    ));

    spawn_enemy_health_bar(&mut commands, enemy_jaar);

    spawned_count.0 += 1;
}

fn setup_enemy_jaar_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(3).from_asset("enemy/Fi_Jaar.glb"),
        ),
        1.0,
        graph.root,
    );

    let chase = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(4).from_asset("enemy/Fi_Jaar.glb"),
        ),
        1.0,
        graph.root,
    );

    let attack = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(0).from_asset("enemy/Fi_Jaar.glb"),
        ),
        1.0,
        graph.root,
    );

    let hurt = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(2).from_asset("enemy/Fi_Jaar.glb"),
        ),
        1.0,
        graph.root,
    );

    let dead = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(1).from_asset("enemy/Fi_Jaar.glb"),
        ),
        1.0,
        graph.root,
    );

    commands.insert_resource(EnemyJaarAnimationGraph {
        graph: graphs.add(graph),
        idle,
        chase,
        attack,
        hurt,
        dead,
    });
}

fn setup_enemy_jaar_animation_player(
    mut commands: Commands,
    animation_graph: Res<EnemyJaarAnimationGraph>,
    mut animation_players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    child_of_query: Query<&ChildOf>,
    jaar_query: Query<(), With<EnemyJaar>>,
) {
    for (animation_entity, mut player) in &mut animation_players {
        let Some(jaar_root) = find_enemy_jaar_root(
            animation_entity,
            &child_of_query,
            &jaar_query,
        ) else {
            continue;
        };

        commands.entity(animation_entity).insert((
            AnimationGraphHandle(animation_graph.graph.clone()),
            EnemyJaarAnimationTarget {
                root: jaar_root,
            },
            EnemyJaarAnimState::Idle,
        ));

        player.stop_all();
        player.play(animation_graph.idle).repeat();
    }
}

fn find_enemy_jaar_root(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    jaar_query: &Query<(), With<EnemyJaar>>,
) -> Option<Entity> {
    let mut current = entity;

    loop {
        if jaar_query.get(current).is_ok() {
            return Some(current);
        }

        let Ok(child_of) = child_of_query.get(current) else {
            return None;
        };

        current = child_of.parent();
    }
}

fn enemy_jaar_behavior(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<
        &Transform,
        (
            With<Player>,
            Without<EnemyJaar>,
        ),
    >,
    mut jaar_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut EnemyState,
            Option<&mut PassivePatrol>,
            Option<&Provoked>,
        ),
        (
            With<EnemyJaar>,
            Without<Player>,
        ),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (
        jaar_entity,
        mut jaar_transform,
        mut velocity,
        mut enemy_state,
        patrol_data,
        provoked,
    ) in &mut jaar_query
    {
        if matches!(*enemy_state, EnemyState::Hurt | EnemyState::Dead) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        if provoked.is_none() {
            if let Some(mut patrol) = patrol_data {
                patrol.timer.tick(time.delta());

                let current_position = Vec3::new(
                    jaar_transform.translation.x,
                    0.0,
                    jaar_transform.translation.z,
                );

                let patrol_center = Vec3::new(
                    patrol.center.x,
                    0.0,
                    patrol.center.z,
                );

                let from_center = current_position - patrol_center;
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

                    let mut new_direction = Vec3::new(
                        random_angle.cos(),
                        0.0,
                        random_angle.sin(),
                    )
                    .normalize();

                    if patrol.radius > 0.0 && distance_from_center > patrol.radius * 0.75 {
                        let outward_direction = from_center.normalize();

                        if new_direction.dot(outward_direction) > 0.0 {
                            new_direction = -outward_direction;
                        }
                    }

                    patrol.direction = new_direction;
                }

                let dir = if patrol.direction.length_squared() > 0.0001 {
                    patrol.direction.normalize()
                } else {
                    Vec3::new(0.0, 0.0, 1.0)
                };

                velocity.x = dir.x * JAAR_MOVE_SPEED;
                velocity.z = dir.z * JAAR_MOVE_SPEED;

                if dir.length_squared() > 0.0001 {
                    jaar_transform.rotation =
                        Quat::from_rotation_y(dir.x.atan2(dir.z));
                }

                *enemy_state = EnemyState::Patrol;
            } else {
                velocity.x = 0.0;
                velocity.z = 0.0;
                *enemy_state = EnemyState::Idle;
            }

            continue;
        }

        let to_player = player_transform.translation - jaar_transform.translation;
        let flat_direction = Vec3::new(to_player.x, 0.0, to_player.z);
        let distance = flat_direction.length();

        if distance > JAAR_LOSE_INTEREST_DISTANCE {
            commands.entity(jaar_entity).remove::<Provoked>();

            if let Some(mut patrol) = patrol_data {
                *enemy_state = EnemyState::Patrol;

                patrol.timer.reset();

                let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
                patrol.direction =
                    Vec3::new(random_angle.cos(), 0.0, random_angle.sin()).normalize();

                let dir = patrol.direction;

                velocity.x = dir.x * JAAR_MOVE_SPEED;
                velocity.z = dir.z * JAAR_MOVE_SPEED;

                if dir.length_squared() > 0.0001 {
                    jaar_transform.rotation =
                        Quat::from_rotation_y(dir.x.atan2(dir.z));
                }
            } else {
                *enemy_state = EnemyState::Idle;
                velocity.x = 0.0;
                velocity.z = 0.0;
            }

            continue;
        }

        if distance <= JAAR_STOP_DISTANCE {
            velocity.x = 0.0;
            velocity.z = 0.0;

            if flat_direction.length_squared() > 0.0001 {
                let direction = flat_direction.normalize();
                jaar_transform.rotation =
                    Quat::from_rotation_y(direction.x.atan2(direction.z));
            }

            *enemy_state = EnemyState::Attack;
            continue;
        }

        let direction = if flat_direction.length_squared() > 0.0001 {
            flat_direction.normalize()
        } else {
            Vec3::ZERO
        };

        velocity.x = direction.x * JAAR_MOVE_SPEED;
        velocity.z = direction.z * JAAR_MOVE_SPEED;

        if direction.length_squared() > 0.0001 {
            jaar_transform.rotation =
                Quat::from_rotation_y(direction.x.atan2(direction.z));
        }

        *enemy_state = EnemyState::Chase;
    }
}

fn update_enemy_jaar_animation(
    animation_graph: Res<EnemyJaarAnimationGraph>,
    jaar_query: Query<&EnemyState, With<EnemyJaar>>,
    mut animation_query: Query<
        (
            &EnemyJaarAnimationTarget,
            &mut AnimationPlayer,
            &mut EnemyJaarAnimState,
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
        let Ok(enemy_state) = jaar_query.get(animation_target.root) else {
            continue;
        };

        let wanted_animation = match enemy_state {
            EnemyState::Hurt => EnemyJaarAnimState::Hurt,
            EnemyState::Dead => EnemyJaarAnimState::Dead,
            EnemyState::Chase | EnemyState::Patrol => EnemyJaarAnimState::Chase,
            EnemyState::Attack => EnemyJaarAnimState::Attack,
            _ => EnemyJaarAnimState::Idle,
        };

        if *current_animation == wanted_animation {
            continue;
        }

        animation_player.stop_all();

        match wanted_animation {
            EnemyJaarAnimState::Idle => {
                animation_player.play(animation_graph.idle).repeat();
            }
            EnemyJaarAnimState::Chase => {
                animation_player.play(animation_graph.chase).repeat();
            }
            EnemyJaarAnimState::Attack => {
                animation_player.play(animation_graph.attack).repeat();
            }
            EnemyJaarAnimState::Hurt => {
                animation_player.play(animation_graph.hurt);
                commands.spawn(AudioPlayer::new(
                    asset_server.load("sounds/enemy/jaar_hurt.ogg"),
                ));
            }
            EnemyJaarAnimState::Dead => {
                animation_player.play(animation_graph.dead);
                commands.spawn(AudioPlayer::new(
                    asset_server.load("sounds/enemy/jaar_dead.ogg"),
                ));
            }
        }

        *current_animation = wanted_animation;
    }
}

fn update_jaar_hurt_and_dead(
    mut commands: Commands,
    time: Res<Time>,
    mut jaar_query: Query<
        (
            Entity,
            &mut EnemyState,
            &mut EnemyStateTimer,
            &mut LinearVelocity,
        ),
        With<EnemyJaar>,
    >,
) {
    for (
        jaar_entity,
        mut enemy_state,
        mut state_timer,
        mut velocity,
    ) in &mut jaar_query
    {
        state_timer.0.tick(time.delta());

        match *enemy_state {
            EnemyState::Hurt => {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
            EnemyState::Dead => {
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
                commands.entity(jaar_entity).remove::<EnemyStateTimer>();
            }
            EnemyState::Dead => {
                commands.entity(jaar_entity).despawn();
            }
            _ => {
                commands.entity(jaar_entity).remove::<EnemyStateTimer>();
            }
        }
    }
}

fn spawn_jaar_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut jaar_query: Query<
        (
            Entity,
            &EnemyState,
            &mut JaarAttackTimer,
        ),
        With<EnemyJaar>,
    >,
) {
    for (
        jaar_entity,
        enemy_state,
        mut attack_timer,
    ) in &mut jaar_query
    {
        if !matches!(*enemy_state, EnemyState::Attack) {
            attack_timer.0.reset();
            continue;
        }

        attack_timer.0.tick(time.delta());

        if !attack_timer.0.just_finished() {
            continue;
        }

        commands
            .entity(jaar_entity)
            .with_children(|parent| {
                parent.spawn((
                    JaarPunchHitbox {
                        owner: jaar_entity,
                        has_hit: false,
                        lifetime: Timer::from_seconds(0.20, TimerMode::Once),
                    },
                    Collider::sphere(0.35),
                    Sensor,
                    CollisionEventsEnabled,
                    Transform::from_xyz(0.0, 0.0, 0.85),
                ));
            });
    }
}

fn jaar_punch_hit_player(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut hitbox_query: Query<&mut JaarPunchHitbox>,
    jaar_query: Query<
        (&CombatStats, &AtkAndDefElement),
        (
            With<EnemyJaar>,
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
            Without<EnemyJaar>,
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
        let (hitbox_entity, other_body) =
            if hitbox_query.contains(event.collider1) {
                (
                    event.collider1,
                    event.body2.unwrap_or(event.collider2),
                )
            } else if hitbox_query.contains(event.collider2) {
                (
                    event.collider2,
                    event.body1.unwrap_or(event.collider1),
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

        let Ok((jaar_stats, jaar_element)) = jaar_query.get(hitbox.owner) else {
            continue;
        };

        let (base_damage, is_critical) = calculate_combat_damage(
            jaar_stats,
            player_stats,
            &mut rng,
        );

        let element_multiplier = elemental_multiplier(
            jaar_element.0,
            player_element.0,
        );

        let damage = (base_damage as f32 * element_multiplier)
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
            player_global_transform.translation() + Vec3::new(0.0, 2.0, 0.0),
            FloatingDamageKind::PlayerHit,
        );

        hitbox.has_hit = true;

        commands.spawn(AudioPlayer::new(
            asset_server.load("sounds/enemy/jaar_atk.ogg"),
        ));

        info!(
            "Jaar hit Player: damage={}, critical={}, HP={}/{}",
            damage,
            is_critical,
            player_health.current,
            player_health.max,
        );
    }
}

fn despawn_jaar_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut hitbox_query: Query<(Entity, &mut JaarPunchHitbox)>,
) {
    for (entity, mut hitbox) in &mut hitbox_query {
        hitbox.lifetime.tick(time.delta());

        if hitbox.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn reset_enemy_jaar_wave(
    mut spawned_count: ResMut<JaarSpawnedCount>,
    mut respawn_timer: ResMut<JaarRespawnTimer>,
) {
    spawned_count.0 = 0;
    respawn_timer.0.reset();
}

fn provoke_enemy_jaar_when_hurt(
    mut commands: Commands,
    query: Query<(Entity, &EnemyState), With<EnemyJaar>>,
) {
    for (entity, state) in &query {
        if matches!(*state, EnemyState::Hurt | EnemyState::Dead) {
            commands.entity(entity).insert(Provoked);
        }
    }
}