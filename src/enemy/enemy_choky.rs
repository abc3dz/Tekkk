use avian3d::prelude::*;
use bevy::{
    gltf::GltfAssetLabel,
    prelude::*,
};
use bevy_wind_waker_shader::prelude::*;

use crate::combat::*;
use crate::components::*;
use crate::npc::practice_common::*;
use crate::player::*;

#[derive(Component, Debug)]
pub struct EnemyChoky;

pub struct EnemyChokyPlugin;

impl Plugin for EnemyChokyPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(
            ChokyRespawnTimer(
                Timer::from_seconds(
                    1.0,
                    TimerMode::Once,
                ),
            ),
        )
        .add_systems(
            OnEnter(GameScene::Desert),
            (
                setup_enemy_choky_animation_graph,
                spawn_enemy_choky,
            ),
        )
        .add_systems(
            Update,
            (
                setup_enemy_choky_animation_player,
                enemy_choky_chase_player,
                update_enemy_choky_animation,
                debug_enemy_choky_spawn,
                update_choky_hurt_and_dead,

                spawn_choky_punch_hitbox,
                choky_punch_hit_player,
                despawn_choky_punch_hitbox,
                respawn_enemy_choky_when_defeated
            )
            .chain()
            .run_if(in_state(GameScene::Desert)),
        );
    }
}

fn spawn_enemy_choky(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    spawn_enemy_choky_entity(
        &mut commands,
        &asset_server,
    );
}

fn spawn_enemy_choky_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    let choky_scene = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("enemy/EnemyChoky.glb"),
    );

    let spawn_position =
        Vec3::new(-13.0, 0.0, -40.0);

    const CHOKY_BODY_Y: f32 = 1.0;

    let base_stats = BaseStats::CHOKY;

    let enemy_choky = commands
        .spawn((
            Name::new("Enemy Choky"),

            Enemy,
            EnemyChoky,
            EnemyState::Idle,

            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },

            base_stats,
            CombatStats::from(base_stats),
            AtkAndDefElement(Element::Earth),
            ElementExpReward::CHOKY,

            RigidBody::Dynamic,
            Collider::capsule(0.45, 1.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,

            Transform::from_translation(
                spawn_position
                    + Vec3::Y * CHOKY_BODY_Y,
            ),

            DespawnOnExit(GameScene::Desert),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(choky_scene),
                Transform::from_xyz(
                    0.0,
                    -CHOKY_BODY_Y,
                    0.0,
                ),
                WindWakerShaderBuilder::default()
                    .time_of_day(TimeOfDay::Day)
                    .weather(Weather::Sunny)
                    .build(),
            ));
        })
        .id();

    commands.entity(enemy_choky).insert((
        CombatTarget,
        ChokyAttackTimer(
            Timer::from_seconds(
                0.3,
                TimerMode::Repeating,
            ),
        ),
    ));

    spawn_enemy_health_bar(commands,enemy_choky);
}

fn debug_enemy_choky_spawn(
    query: Query<
        (
            &Health,
            &CombatStats,
            &AtkAndDefElement,
            &ElementExpReward,
            &EnemyState,
        ),
        Added<EnemyChoky>,
    >,
) {
    for (health, stats, element, reward, state) in &query {
        info!(
            "Choky spawned:
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

fn setup_enemy_choky_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(3)
                .from_asset("enemy/EnemyChoky.glb"),
        ),
        1.0,
        graph.root,
    );
    let chase = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(4)
                .from_asset("enemy/EnemyChoky.glb"),
        ),
        1.0,
        graph.root,
    );
    let attack = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(0)
                .from_asset("enemy/EnemyChoky.glb"),
        ),
        1.0,
        graph.root,
    );
    let hurt = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(2)
                .from_asset("enemy/EnemyChoky.glb"),
        ),
        1.0,
        graph.root,
    );
    let dead = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(1)
                .from_asset("enemy/EnemyChoky.glb"),
        ),
        1.0,
        graph.root,
    );

    commands.insert_resource(EnemyChokyAnimationGraph {
        graph: graphs.add(graph),
        idle,
        chase,
        attack,
        hurt,
        dead
    });
}

fn setup_enemy_choky_animation_player(
    mut commands: Commands,
    animation_graph: Res<EnemyChokyAnimationGraph>,

    mut animation_players: Query<
        (Entity, &mut AnimationPlayer),
        Added<AnimationPlayer>,
    >,

    child_of_query: Query<&ChildOf>,
    choky_query: Query<(), With<EnemyChoky>>,
) {
    for (animation_entity, mut player) in &mut animation_players {
        let Some(choky_root) = find_enemy_choky_root(
            animation_entity,
            &child_of_query,
            &choky_query,
        ) else {
            continue;
        };

        commands.entity(animation_entity).insert((
            AnimationGraphHandle(animation_graph.graph.clone()),

            EnemyChokyAnimationTarget {
                root: choky_root,
            },

            EnemyChokyAnimState::Idle,
        ));

        player.stop_all();
        player.play(animation_graph.idle).repeat();
    }
}

fn find_enemy_choky_root(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    choky_query: &Query<(), With<EnemyChoky>>,
) -> Option<Entity> {
    let mut current = entity;

    loop {
        if choky_query.get(current).is_ok() {
            return Some(current);
        }

        let Ok(child_of) = child_of_query.get(current) else {
            return None;
        };

        current = child_of.parent();
    }
}

const CHOKY_CHASE_RANGE: f32 = 10.0;
const CHOKY_STOP_DISTANCE: f32 = 1.0;
const CHOKY_MOVE_SPEED: f32 = 3.0;

fn enemy_choky_chase_player(
    player_query: Query<
        &Transform,
        (
            With<Player>,
            Without<EnemyChoky>,
        ),
    >,

    mut choky_query: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut EnemyState,
        ),
        (
            With<EnemyChoky>,
            Without<Player>,
        ),
    >,
) {
    let Ok(player_transform) =
        player_query.single()
    else {
        return;
    };

    for (
        mut choky_transform,
        mut velocity,
        mut enemy_state,
    ) in &mut choky_query
    {
        // ตอนเจ็บหรือตาย ห้ามเดินไล่
        // และห้ามเปลี่ยนกลับเป็น Idle/Chase/Attack
        if matches!(*enemy_state, EnemyState::Hurt | EnemyState::Dead) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        let to_player = player_transform.translation- choky_transform.translation;

        // ไม่ใช้แกน Y เพราะเดินบนพื้น
        let flat_direction = Vec3::new(to_player.x,0.0,to_player.z);

        let distance = flat_direction.length();

        // Player อยู่นอกระยะตรวจจับ
        if distance > CHOKY_CHASE_RANGE {
            velocity.x = 0.0;
            velocity.z = 0.0;

            *enemy_state =
                EnemyState::Idle;

            continue;
        }

        // Player อยู่ในระยะต่อย
        if distance <= CHOKY_STOP_DISTANCE {
            velocity.x = 0.0;
            velocity.z = 0.0;

            // หันหน้าเข้าหา Player
            if flat_direction.length_squared()
                > 0.0001
            {
                let direction =
                    flat_direction.normalize();

                choky_transform.rotation =
                    Quat::from_rotation_y(
                        direction
                            .x
                            .atan2(direction.z),
                    );
            }

            *enemy_state =
                EnemyState::Attack;

            continue;
        }

        // อยู่ในระยะตรวจจับ แต่ยังไม่ถึงระยะต่อย
        let direction =
            flat_direction.normalize();

        velocity.x =
            direction.x * CHOKY_MOVE_SPEED;

        velocity.z =
            direction.z * CHOKY_MOVE_SPEED;

        choky_transform.rotation =
            Quat::from_rotation_y(
                direction.x.atan2(direction.z),
            );

        *enemy_state =
            EnemyState::Chase;
    }
}

fn update_enemy_choky_animation(
    animation_graph: Res<EnemyChokyAnimationGraph>,
    choky_query: Query<&EnemyState, With<EnemyChoky>>,
    mut animation_query: Query<(&EnemyChokyAnimationTarget,&mut AnimationPlayer,&mut EnemyChokyAnimState)>,
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
            choky_query.get(animation_target.root)
        else {
            continue;
        };

        let wanted_animation = match enemy_state {
            EnemyState::Hurt => EnemyChokyAnimState::Hurt,
            EnemyState::Dead => EnemyChokyAnimState::Dead,
            EnemyState::Chase => {
                EnemyChokyAnimState::Chase
            }

            EnemyState::Attack => {
                EnemyChokyAnimState::Attack
            }

            _ => EnemyChokyAnimState::Idle,
        };

        // Animation เดิมกำลังเล่นอยู่ ไม่ต้องเริ่มใหม่ทุก Frame
        if *current_animation == wanted_animation {
            continue;
        }

        animation_player.stop_all();

        match wanted_animation {
            EnemyChokyAnimState::Idle => {
                animation_player
                    .play(animation_graph.idle)
                    .repeat();
            }

            EnemyChokyAnimState::Chase => {
                animation_player
                    .play(animation_graph.chase)
                    .repeat();
            }
            EnemyChokyAnimState::Attack => {
                animation_player
                    .play(animation_graph.attack)
                    .repeat();
            }
            EnemyChokyAnimState::Hurt => {
                animation_player.play(animation_graph.hurt);
                commands.spawn(AudioPlayer::new(asset_server.load("sounds/enemy/567989__ancientwarrior__woundedmaleshort_choky_hurt.ogg")));
            }

            EnemyChokyAnimState::Dead => {
                animation_player.play(animation_graph.dead);
                commands.spawn(AudioPlayer::new(asset_server.load("sounds/enemy/567989__ancientwarrior__woundedmaleshort_choky_dead.ogg")));
            }
        }

        *current_animation = wanted_animation;
    }
}

fn update_choky_hurt_and_dead(
    mut commands: Commands,
    time: Res<Time>,

    mut choky_query: Query<
        (
            Entity,
            &mut EnemyState,
            &mut EnemyStateTimer,
            &mut LinearVelocity,
        ),
        With<EnemyChoky>,
    >,
) {
    for (
        choky_entity,
        mut enemy_state,
        mut state_timer,
        mut velocity,
    ) in &mut choky_query
    {
        velocity.x = 0.0;
        velocity.z = 0.0;

        state_timer.0.tick(time.delta());

        if !state_timer.0.is_finished() {
            continue;
        }

        match *enemy_state {
            EnemyState::Hurt => {
                *enemy_state = EnemyState::Idle;

                commands
                    .entity(choky_entity)
                    .remove::<EnemyStateTimer>();
            }

            EnemyState::Dead => {
                commands
                    .entity(choky_entity)
                    .despawn();
            }

            _ => {
                commands
                    .entity(choky_entity)
                    .remove::<EnemyStateTimer>();
            }
        }
    }
}

fn spawn_choky_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,

    mut choky_query: Query<
        (
            Entity,
            &EnemyState,
            &mut ChokyAttackTimer,
        ),
        With<EnemyChoky>,
    >,
) {
    for (
        choky_entity,
        enemy_state,
        mut attack_timer,
    ) in &mut choky_query
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
            .entity(choky_entity)
            .with_children(|parent| {
                parent.spawn((
                    ChokyPunchHitbox {
                        owner: choky_entity,
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

fn choky_punch_hit_player(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut hitbox_query: Query<&mut ChokyPunchHitbox>,
    choky_query: Query<(&CombatStats,&AtkAndDefElement),(With<EnemyChoky>,Without<Player>)>,
    mut player_query: Query<(Entity,&mut Health,&CombatStats,&AtkAndDefElement,&GlobalTransform),
        (
            With<Player>,
            Without<EnemyChoky>,
        ),
    >,
    anim_graph: Res<PlayerAnimationGraph>,
    mut player_anim_query: Query<(&mut AnimationPlayer,&mut PlayerAnimState),With<PlayerAnimationTarget>>,
    asset_server: Res<AssetServer>
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
            choky_stats,
            choky_element,
        )) = choky_query.get(hitbox.owner)
        else {
            continue;
        };

        let (
            base_damage,
            is_critical,
        ) = calculate_combat_damage(
            choky_stats,
            player_stats,
            &mut rng,
        );

        let element_multiplier =
            elemental_multiplier(
                choky_element.0,
                player_element.0,
            );

        let damage = (
            base_damage as f32
                * element_multiplier
        )
            .round()
            .max(1.0) as i32;

        player_health.current -= damage;

        player_health.current =
            player_health.current.clamp(
                0,
                player_health.max,
            );
        if player_health.current > 0 {
            play_player_hurt_animation(
                &mut commands,
                player_entity,
                &anim_graph,
                &mut player_anim_query,
            );
        }
        commands.spawn(AudioPlayer::new(asset_server.load("sounds/331935__pyro13djt__hit_hurt.ogg")));
        spawn_floating_damage_text(
            &mut commands,
            damage,
            player_global_transform.translation()
                + Vec3::new(0.0, 2.0, 0.0),
            FloatingDamageKind::PlayerHit,
        );
        hitbox.has_hit = true;

        info!(
            "choky hit Player: damage={}, critical={}, HP={}/{}",
            damage,
            is_critical,
            player_health.current,
            player_health.max,
        );
    }
}

fn despawn_choky_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,

    mut hitbox_query: Query<
        (
            Entity,
            &mut ChokyPunchHitbox,
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

fn respawn_enemy_choky_when_defeated(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,

    mut respawn_timer:
        ResMut<ChokyRespawnTimer>,

    choky_query:
        Query<(), With<EnemyChoky>>,
) {
    // choky ยังอยู่ ไม่ต้อง Respawn
    if !choky_query.is_empty() {
        respawn_timer.0.reset();
        return;
    }

    respawn_timer.0.tick(time.delta());

    if !respawn_timer.0.just_finished() {
        return;
    }

    spawn_enemy_choky_entity(
        &mut commands,
        &asset_server,
    );

    respawn_timer.0.reset();

    info!("Enemy choky respawned");
}