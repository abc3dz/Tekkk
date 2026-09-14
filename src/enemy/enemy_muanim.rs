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
pub struct EnemyMuanim;

pub struct EnemyMuanimPlugin;

impl Plugin for EnemyMuanimPlugin {
    fn build(&self, app: &mut App) { 
        app.insert_resource(MuanimRespawnTimer(Timer::from_seconds(1.0,TimerMode::Once)))
        .init_resource::<MuanimSpawnedCount>()
        .add_systems(
            OnEnter(GameScene::Desert),
            (
                reset_enemy_muanim_wave,
                setup_enemy_muanim_animation_graph,
            ),
        )
        .add_systems(OnEnter(GameScene::Desert),setup_enemy_muanim_animation_graph)
        .add_systems(Update,(
                spawn_enemy_muanim,
                setup_enemy_muanim_animation_player,
                provoke_enemy_muanim_when_hurt,
                enemy_muanim_behavior,
                update_enemy_muanim_animation,
                update_muanim_hurt_and_dead,
                spawn_muanim_punch_hitbox,
                muanim_punch_hit_player,
                despawn_muanim_punch_hitbox,
            )
            .chain()
            .run_if(in_state(GameScene::Desert).and(in_state(GameMode::Playing))),
        );
    }
}

const MUANIM_SPAWN_POSITION: Vec3 = Vec3::new(-10.0, 1.0, -10.0);
fn spawn_enemy_muanim(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawned_count: ResMut<MuanimSpawnedCount>,
) {
    if spawned_count.0 >= MUANIM_SPAWN_LIMIT {
        return;
    }

    let muanim_scene = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("enemy/EnemyMuanim.glb"),
    );
    let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
    let initial_dir = Vec3::new(random_angle.cos(), 0.0, random_angle.sin()).normalize();
    const MUANIM_BODY_Y: f32 = 1.0;
    let base_stats = BaseStats::MUANIM;
    let enemy_muanim = commands.spawn((
            Name::new("Enemy Muanim"),
            Enemy,
            EnemyMuanim,
            EnemyState::Patrol,
            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },
            base_stats,
            CombatStats::from(base_stats),
            AtkAndDefElement(Element::Earth),
            ElementExpReward::MUANIM,
            RigidBody::Dynamic,
            Collider::capsule(0.45, 1.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            Transform::from_translation(MUANIM_SPAWN_POSITION + Vec3::Y * MUANIM_BODY_Y),
            DespawnOnExit(GameScene::Desert),
        ))
        .insert(PassivePatrol {
            direction: initial_dir,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
        })
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(muanim_scene),
                Transform::from_xyz(0.0,-MUANIM_BODY_Y,0.0,),
                ApplyToonMaterial,
            ));
        })
        .id();
    
    commands.entity(enemy_muanim).insert((
        CombatTarget,
        MuanimAttackTimer(Timer::from_seconds(
            0.3,
            TimerMode::Repeating,
        )),
    ));
    spawn_enemy_health_bar(
        &mut commands,
        enemy_muanim,
    );
    //respawn_timer.0.reset();
    spawned_count.0 += 1;
    //info!("Enemy muanim spawned");
}

fn setup_enemy_muanim_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(3)
                .from_asset("enemy/EnemyMuanim.glb"),
        ),
        1.0,
        graph.root,
    );
    let chase = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(4)
                .from_asset("enemy/EnemyMuanim.glb"),
        ),
        1.0,
        graph.root,
    );
    let attack = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(0)
                .from_asset("enemy/EnemyMuanim.glb"),
        ),
        1.0,
        graph.root,
    );
    let hurt = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(2)
                .from_asset("enemy/EnemyMuanim.glb"),
        ),
        1.0,
        graph.root,
    );
    let dead = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(1)
                .from_asset("enemy/EnemyMuanim.glb"),
        ),
        1.0,
        graph.root,
    );

    commands.insert_resource(EnemyMuanimAnimationGraph {
        graph: graphs.add(graph),
        idle,
        chase,
        attack,
        hurt,
        dead
    });
}

fn setup_enemy_muanim_animation_player(
    mut commands: Commands,
    animation_graph: Res<EnemyMuanimAnimationGraph>,

    mut animation_players: Query<
        (Entity, &mut AnimationPlayer),
        Added<AnimationPlayer>,
    >,

    child_of_query: Query<&ChildOf>,
    muanim_query: Query<(), With<EnemyMuanim>>,
) {
    for (animation_entity, mut player) in &mut animation_players {
        let Some(muanim_root) = find_enemy_muanim_root(
            animation_entity,
            &child_of_query,
            &muanim_query,
        ) else {
            continue;
        };

        commands.entity(animation_entity).insert((
            AnimationGraphHandle(animation_graph.graph.clone()),

            EnemyMuanimAnimationTarget {
                root: muanim_root,
            },

            EnemyMuanimAnimState::Idle,
        ));

        player.stop_all();
        player.play(animation_graph.idle).repeat();
    }
}

fn find_enemy_muanim_root(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    muanim_query: &Query<(), With<EnemyMuanim>>,
) -> Option<Entity> {
    let mut current = entity;

    loop {
        if muanim_query.get(current).is_ok() {
            return Some(current);
        }

        let Ok(child_of) = child_of_query.get(current) else {
            return None;
        };

        current = child_of.parent();
    }
}

const MUANIM_CHASE_RANGE: f32 = 10.0;
const MUANIM_STOP_DISTANCE: f32 = 1.0;
const MUANIM_MOVE_SPEED: f32 = 3.0;

fn enemy_muanim_behavior(
    time: Res<Time>,
    player_query: Query<
        &Transform,
        (
            With<Player>,
            Without<EnemyMuanim>,
        ),
    >,
    mut muanim_query: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut EnemyState,
            Option<&mut PassivePatrol>,
            Option<&Provoked>,
        ),
        (
            With<EnemyMuanim>,
            Without<Player>,
        ),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (
        mut muanim_transform,
        mut velocity,
        mut enemy_state,
        patrol_data,
        provoked,
    ) in &mut muanim_query
    {
        // ถ้ากำลังเจ็บหรือตาย ให้หยุดก่อน
        if matches!(*enemy_state, EnemyState::Hurt | EnemyState::Dead) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // ==================================================
        // กรณีที่ 1 : ยังไม่ถูกผู้เล่นโจมตี
        // ให้เดินสุ่มอย่างเดียว ไม่สนใจผู้เล่น
        // ==================================================
        if provoked.is_none() {
            if let Some(mut patrol) = patrol_data {
                patrol.timer.tick(time.delta());

                if patrol.timer.just_finished() {
                    let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
                    patrol.direction =
                        Vec3::new(random_angle.cos(), 0.0, random_angle.sin()).normalize();
                }

                let dir = patrol.direction;

                velocity.x = dir.x * MUANIM_MOVE_SPEED;
                velocity.z = dir.z * MUANIM_MOVE_SPEED;

                if dir.length_squared() > 0.0001 {
                    muanim_transform.rotation =
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

        // ==================================================
        // กรณีที่ 2 : ถูกผู้เล่นโจมตีแล้ว
        // ให้ไล่และโจมตีผู้เล่น
        // ==================================================
        let to_player = player_transform.translation - muanim_transform.translation;
        let flat_direction = Vec3::new(to_player.x, 0.0, to_player.z);
        let distance = flat_direction.length();

        // ถ้าใกล้ผู้เล่นมากพอ ให้หยุดแล้วโจมตี
        if distance <= MUANIM_STOP_DISTANCE {
            velocity.x = 0.0;
            velocity.z = 0.0;

            if flat_direction.length_squared() > 0.0001 {
                let direction = flat_direction.normalize();
                muanim_transform.rotation =
                    Quat::from_rotation_y(direction.x.atan2(direction.z));
            }

            *enemy_state = EnemyState::Attack;
            continue;
        }

        // ถ้ายังไม่ใกล้พอ ให้ไล่ผู้เล่น
        let direction = if flat_direction.length_squared() > 0.0001 {
            flat_direction.normalize()
        } else {
            Vec3::ZERO
        };

        velocity.x = direction.x * MUANIM_MOVE_SPEED;
        velocity.z = direction.z * MUANIM_MOVE_SPEED;

        if direction.length_squared() > 0.0001 {
            muanim_transform.rotation =
                Quat::from_rotation_y(direction.x.atan2(direction.z));
        }

        *enemy_state = EnemyState::Chase;
    }
}

fn update_enemy_muanim_animation(
    animation_graph: Res<EnemyMuanimAnimationGraph>,
    muanim_query: Query<&EnemyState,With<EnemyMuanim>>,

    mut animation_query: Query<
        (
            &EnemyMuanimAnimationTarget,
            &mut AnimationPlayer,
            &mut EnemyMuanimAnimState,
        ),
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    for (
        animation_target,
        mut animation_player,
        mut current_animation,
    ) in &mut animation_query
    {
        let Ok(enemy_state) =
            muanim_query.get(animation_target.root)
        else {
            continue;
        };

        let wanted_animation = match enemy_state {
            EnemyState::Hurt => EnemyMuanimAnimState::Hurt,
            EnemyState::Dead => EnemyMuanimAnimState::Dead,
            EnemyState::Chase | EnemyState::Patrol => EnemyMuanimAnimState::Chase,
            EnemyState::Attack => EnemyMuanimAnimState::Attack,
            _ => EnemyMuanimAnimState::Idle,
        };

        // Animation เดิมกำลังเล่นอยู่ ไม่ต้องเริ่มใหม่ทุก Frame
        if *current_animation == wanted_animation {
            continue;
        }

        animation_player.stop_all();

        match wanted_animation {
            EnemyMuanimAnimState::Idle => {
                animation_player
                    .play(animation_graph.idle)
                    .repeat();
            }

            EnemyMuanimAnimState::Chase => {
                animation_player
                    .play(animation_graph.chase)
                    .repeat();
            }
            EnemyMuanimAnimState::Attack => {
                animation_player
                    .play(animation_graph.attack)
                    .repeat();
            }
            EnemyMuanimAnimState::Hurt => {
                animation_player.play(animation_graph.hurt);
                commands.spawn(AudioPlayer::new(asset_server.load("sounds/enemy/muanim_hurt.ogg")));
            }

            EnemyMuanimAnimState::Dead => {
                animation_player.play(animation_graph.dead);
                commands.spawn(AudioPlayer::new(asset_server.load("sounds/enemy/muanim_dead.ogg")));
            }
        }

        *current_animation = wanted_animation;
    }
}

fn update_muanim_hurt_and_dead(
    mut commands: Commands,
    time: Res<Time>,

    mut muanim_query: Query<
        (
            Entity,
            &mut EnemyState,
            &mut EnemyStateTimer,
            &mut LinearVelocity,
        ),
        With<EnemyMuanim>,
    >,
) {
    for (
        muanim_entity,
        mut enemy_state,
        mut state_timer,
        mut velocity,
    ) in &mut muanim_query
    {
        state_timer.0.tick(time.delta());

        match *enemy_state {
            EnemyState::Hurt => {
                // Hurt ธรรมดาไม่กระเด็น
                velocity.x = 0.0;
                velocity.z = 0.0;
            }

            EnemyState::Dead => {
                // ปล่อยให้กระเด็น 0.25 วินาที
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
                    .entity(muanim_entity)
                    .remove::<EnemyStateTimer>();
            }

            EnemyState::Dead => {
                commands
                    .entity(muanim_entity)
                    .despawn();
            }

            _ => {
                commands
                    .entity(muanim_entity)
                    .remove::<EnemyStateTimer>();
            }
        }
    }
}

fn spawn_muanim_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,

    mut muanim_query: Query<
        (
            Entity,
            &EnemyState,
            &mut MuanimAttackTimer,
        ),
        With<EnemyMuanim>,
    >,
) {
    for (
        muanim_entity,
        enemy_state,
        mut attack_timer,
    ) in &mut muanim_query
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
            .entity(muanim_entity)
            .with_children(|parent| {
                parent.spawn((
                    MuanimPunchHitbox {
                        owner: muanim_entity,
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

fn muanim_punch_hit_player(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut hitbox_query: Query<&mut MuanimPunchHitbox>,
    muanim_query: Query<(&CombatStats,&AtkAndDefElement),(With<EnemyMuanim>,Without<Player>)>,
    mut player_query: Query<(Entity,&mut Health,&CombatStats,&AtkAndDefElement,&GlobalTransform),
        (
            With<Player>,
            Without<EnemyMuanim>,
        ),
    >,
    anim_graph: Res<PlayerAnimationGraph>,
    mut anim_query: Query<(&mut AnimationPlayer,&mut PlayerAnimState),With<PlayerAnimationTarget>>,
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
            muanim_stats,
            muanim_element,
        )) = muanim_query.get(hitbox.owner)
        else {
            continue;
        };

        let (
            base_damage,
            is_critical,
        ) = calculate_combat_damage(
            muanim_stats,
            player_stats,
            &mut rng,
        );

        let element_multiplier =
            elemental_multiplier(
                muanim_element.0,
                player_element.0,
            );

        let damage = (
            base_damage as f32
                * element_multiplier
        )
            .round()
            .max(1.0) as i32;

        player_health.current -= damage;

        player_health.current = player_health.current.clamp(0,player_health.max);

        if player_health.current > 0 {
            play_player_hurt_animation(
                &mut commands,
                player_entity,
                &anim_graph,
                &mut anim_query,
                &camera_query,
                &asset_server
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
        commands.spawn(AudioPlayer::new(asset_server.load("sounds/enemy/muanim_atk.ogg")));
        info!(
            "muanim hit Player: damage={}, critical={}, HP={}/{}",
            damage,
            is_critical,
            player_health.current,
            player_health.max,
        );
    }
}

fn despawn_muanim_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,

    mut hitbox_query: Query<
        (
            Entity,
            &mut MuanimPunchHitbox,
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
fn reset_enemy_muanim_wave(
    mut spawned_count: ResMut<MuanimSpawnedCount>,
    mut respawn_timer: ResMut<MuanimRespawnTimer>,
) {
    spawned_count.0 = 0;
    respawn_timer.0.reset();
}
fn provoke_enemy_muanim_when_hurt(
    mut commands: Commands,
    query: Query<(Entity, &EnemyState), With<EnemyMuanim>>,
) {
    for (entity, state) in &query {
        if matches!(*state, EnemyState::Hurt | EnemyState::Dead) {
            commands.entity(entity).insert(Provoked);
        }
    }
}