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
use crate::camera::*;

#[derive(Component, Debug)]
pub struct EnemyMuamua;

pub struct EnemyMuamuaPlugin;

impl Plugin for EnemyMuamuaPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(
            MuamuaRespawnTimer(
                Timer::from_seconds(
                    1.0,
                    TimerMode::Once,
                ),
            ),
        )
        .add_systems(
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
                update_muamua_hurt_and_dead,

                spawn_muamua_punch_hitbox,
                muamua_punch_hit_player,
                despawn_muamua_punch_hitbox,
                respawn_enemy_muamua_when_defeated
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
    spawn_enemy_muamua_entity(
        &mut commands,
        &asset_server,
    );
}

fn spawn_enemy_muamua_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    let muamua_scene = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("enemy/EnemyMuamua.glb"),
    );

    let spawn_position =
        Vec3::new(5.0, 0.0, 5.0);

    const MUAMUA_BODY_Y: f32 = 1.0;

    let base_stats = BaseStats::MUAMUA;

    let enemy_muamua = commands
        .spawn((
            Name::new("Enemy Muamua"),

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

            RigidBody::Dynamic,
            Collider::capsule(0.45, 1.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,

            Transform::from_translation(
                spawn_position
                    + Vec3::Y * MUAMUA_BODY_Y,
            ),

            DespawnOnExit(GameScene::Desert),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(muamua_scene),
                Transform::from_xyz(
                    0.0,
                    -MUAMUA_BODY_Y,
                    0.0,
                ),
                WindWakerShaderBuilder::default()
                    .time_of_day(TimeOfDay::Day)
                    .weather(Weather::Sunny)
                    .build(),
            ));
        })
        .id();

    commands.entity(enemy_muamua).insert((
        CombatTarget,
        MuamuaAttackTimer(
            Timer::from_seconds(
                0.3,
                TimerMode::Repeating,
            ),
        ),
    ));

    spawn_enemy_health_bar(
        commands,
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
    let attack = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(0)
                .from_asset("enemy/EnemyMuamua.glb"),
        ),
        1.0,
        graph.root,
    );
    let hurt = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(1)
                .from_asset("enemy/EnemyMuamua.glb"),
        ),
        1.0,
        graph.root,
    );
    let dead = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(2)
                .from_asset("enemy/EnemyMuamua.glb"),
        ),
        1.0,
        graph.root,
    );

    commands.insert_resource(EnemyMuamuaAnimationGraph {
        graph: graphs.add(graph),
        idle,
        chase,
        attack,
        hurt,
        dead
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
const MUAMUA_STOP_DISTANCE: f32 = 1.0;
const MUAMUA_MOVE_SPEED: f32 = 3.0;

fn enemy_muamua_chase_player(

    mut commands: Commands,
    time: Res<Time>,

    player_query: Query<
        &Transform,
        (
            With<Player>,
            Without<EnemyMuamua>,
        ),
    >,

    mut muamua_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut EnemyState,
            Option<&mut EnemyInvestigateDirection>,
        ),
        (
            With<EnemyMuamua>,
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
        muamua_entity,
        mut muamua_transform,
        mut velocity,
        mut enemy_state,
        investigate,
    ) in &mut muamua_query
    {
        // Hurt / Dead ยังหยุดเหมือนเดิม
        if matches!(
            *enemy_state,
            EnemyState::Hurt | EnemyState::Dead
        ) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        let to_player =
            player_transform.translation
                - muamua_transform.translation;

        let flat_direction =
            Vec3::new(
                to_player.x,
                0.0,
                to_player.z,
            );

        let distance =
            flat_direction.length();

        // ==========================================
        // Player ยังอยู่นอกระยะมองเห็น
        // แต่ Muamua รู้ว่าลูกพลังมาจากทางไหน
        // ==========================================
        if distance > MUAMUA_CHASE_RANGE {
            if let Some(mut investigate) = investigate {
                investigate.timer.tick(time.delta());

                // หมดเวลาค้นหา
                if investigate.timer.is_finished() {
                    velocity.x = 0.0;
                    velocity.z = 0.0;

                    *enemy_state =
                        EnemyState::Idle;

                    commands
                        .entity(muamua_entity)
                        .remove::<EnemyInvestigateDirection>();

                    continue;
                }

                let direction =
                    investigate.direction;

                // เดินไปทางต้นทางของลูกพลัง
                velocity.x =
                    direction.x
                        * MUAMUA_MOVE_SPEED;

                velocity.z =
                    direction.z
                        * MUAMUA_MOVE_SPEED;

                // หันหน้าไปทางที่เดิน
                if direction.length_squared()
                    > 0.0001
                {
                    muamua_transform.rotation =
                        Quat::from_rotation_y(
                            direction
                                .x
                                .atan2(direction.z),
                        );
                }

                // ใช้ animation Chase เป็นท่าเดิน
                *enemy_state =
                    EnemyState::Chase;

                continue;
            }

            // ไม่ได้โดนยิง
            // และ Player ก็อยู่ไกล
            // = Idle เหมือนเดิม
            velocity.x = 0.0;
            velocity.z = 0.0;

            *enemy_state =
                EnemyState::Idle;

            continue;
        }

        // ==========================================
        // เจอ Player แล้ว
        // ไม่ต้องตามทิศ projectile อีก
        // ==========================================
        commands
            .entity(muamua_entity)
            .remove::<EnemyInvestigateDirection>();

        // Player อยู่ในระยะต่อย
        if distance <= MUAMUA_STOP_DISTANCE {
            velocity.x = 0.0;
            velocity.z = 0.0;

            if flat_direction.length_squared()
                > 0.0001
            {
                let direction =
                    flat_direction.normalize();

                muamua_transform.rotation =
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

        // ==========================================
        // เจอ Player แล้ว → ไล่ Player ตามปกติ
        // ==========================================
        let direction =
            flat_direction.normalize();

        velocity.x =
            direction.x
                * MUAMUA_MOVE_SPEED;

        velocity.z =
            direction.z
                * MUAMUA_MOVE_SPEED;

        muamua_transform.rotation =
            Quat::from_rotation_y(
                direction
                    .x
                    .atan2(direction.z),
            );

        *enemy_state =
            EnemyState::Chase;
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
            muamua_query.get(animation_target.root)
        else {
            continue;
        };

        let wanted_animation = match enemy_state {
            EnemyState::Hurt => EnemyMuamuaAnimState::Hurt,
            EnemyState::Dead => EnemyMuamuaAnimState::Dead,
            EnemyState::Chase => {
                EnemyMuamuaAnimState::Chase
            }

            EnemyState::Attack => {
                EnemyMuamuaAnimState::Attack
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
            EnemyMuamuaAnimState::Attack => {
                animation_player
                    .play(animation_graph.attack)
                    .repeat();
            }
            EnemyMuamuaAnimState::Hurt => {
                animation_player.play(animation_graph.hurt);
                commands.spawn(AudioPlayer::new(asset_server.load("sounds/enemy/404327__pfranzen__male-grunts-and-groans_muamua_hurt.ogg")));
            }

            EnemyMuamuaAnimState::Dead => {
                animation_player.play(animation_graph.dead);
                commands.spawn(AudioPlayer::new(asset_server.load("sounds/enemy/404327__pfranzen__male-grunts-and-groans_muamua_dead.ogg")));
            }
        }

        *current_animation = wanted_animation;
    }
}

fn update_muamua_hurt_and_dead(
    mut commands: Commands,
    time: Res<Time>,

    mut muamua_query: Query<
        (
            Entity,
            &mut EnemyState,
            &mut EnemyStateTimer,
            &mut LinearVelocity,
        ),
        With<EnemyMuamua>,
    >,
) {
    for (
        muamua_entity,
        mut enemy_state,
        mut state_timer,
        mut velocity,
    ) in &mut muamua_query
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
                    .entity(muamua_entity)
                    .remove::<EnemyStateTimer>();
            }

            EnemyState::Dead => {
                commands
                    .entity(muamua_entity)
                    .despawn();
            }

            _ => {
                commands
                    .entity(muamua_entity)
                    .remove::<EnemyStateTimer>();
            }
        }
    }
}

fn spawn_muamua_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,

    mut muamua_query: Query<
        (
            Entity,
            &EnemyState,
            &mut MuamuaAttackTimer,
        ),
        With<EnemyMuamua>,
    >,
) {
    for (
        muamua_entity,
        enemy_state,
        mut attack_timer,
    ) in &mut muamua_query
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
            .entity(muamua_entity)
            .with_children(|parent| {
                parent.spawn((
                    MuamuaPunchHitbox {
                        owner: muamua_entity,
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

fn muamua_punch_hit_player(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut hitbox_query: Query<&mut MuamuaPunchHitbox>,
    muamua_query: Query<(&CombatStats,&AtkAndDefElement),(With<EnemyMuamua>,Without<Player>)>,
    mut player_query: Query<(Entity,&mut Health,&CombatStats,&AtkAndDefElement,&GlobalTransform),
        (
            With<Player>,
            Without<EnemyMuamua>,
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
            muamua_stats,
            muamua_element,
        )) = muamua_query.get(hitbox.owner)
        else {
            continue;
        };

        let (
            base_damage,
            is_critical,
        ) = calculate_combat_damage(
            muamua_stats,
            player_stats,
            &mut rng,
        );

        let element_multiplier =
            elemental_multiplier(
                muamua_element.0,
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

        info!(
            "Muamua hit Player: damage={}, critical={}, HP={}/{}",
            damage,
            is_critical,
            player_health.current,
            player_health.max,
        );
    }
}

fn despawn_muamua_punch_hitbox(
    mut commands: Commands,
    time: Res<Time>,

    mut hitbox_query: Query<
        (
            Entity,
            &mut MuamuaPunchHitbox,
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

fn respawn_enemy_muamua_when_defeated(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,

    mut respawn_timer:
        ResMut<MuamuaRespawnTimer>,

    muamua_query:
        Query<(), With<EnemyMuamua>>,
) {
    // Muamua ยังอยู่ ไม่ต้อง Respawn
    if !muamua_query.is_empty() {
        respawn_timer.0.reset();
        return;
    }

    respawn_timer.0.tick(time.delta());

    if !respawn_timer.0.just_finished() {
        return;
    }

    spawn_enemy_muamua_entity(
        &mut commands,
        &asset_server,
    );

    respawn_timer.0.reset();

    info!("Enemy Muamua respawned");
}