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

const CHOKY_SPAWN_POSITION: Vec3 = Vec3::new(-15.0, 0.0, -50.0);
const CHOKY_CHASE_RANGE: f32 = 10.0;
const CHOKY_STOP_DISTANCE: f32 = 1.0;
const CHOKY_RETURN_STOP_DISTANCE: f32 = 0.25;
const CHOKY_MOVE_SPEED: f32 = 3.0;
const CHOKY_WARP_POSITION: Vec3 = Vec3::new(17.5, 1.0, -48.0);
const CHOKY_CRITICAL_KNOCKBACK_UP: f32 = 4.5;

pub struct EnemyChokyPlugin;

impl Plugin for EnemyChokyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChokyHasSpawned>()
            .add_systems(
                OnEnter(GameScene::Desert),
                (
                    setup_enemy_choky_animation_graph,
                    reset_choky_spawn_once,
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_enemy_choky,
                    setup_enemy_choky_animation_player,
                    enemy_choky_chase_player,
                    update_enemy_choky_animation,
                    debug_enemy_choky_spawn,
                    update_choky_hurt_and_dead,
                    spawn_choky_punch_hitbox,
                    choky_punch_hit_player,
                    despawn_choky_punch_hitbox,
                    despawn_choky_temporary,
                )
                    .chain()
                    .run_if(in_state(GameScene::Desert).and(in_state(GameMode::Playing))),
            );
    }
}

fn spawn_enemy_choky(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    choky_query: Query<(), With<EnemyChoky>>,
    mut has_spawned: ResMut<ChokyHasSpawned>,
) {
    if has_spawned.0 {
        return;
    }

    // ถ้ามี Choky อยู่ในฉากอยู่แล้ว ไม่ต้องสร้างเพิ่ม
    if !choky_query.is_empty() {
        has_spawned.0 = true;
        return;
    }

    let choky_scene = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("enemy/EnemyChoky.glb"),
    );

    let spawn_position = CHOKY_SPAWN_POSITION;

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
            RigidBody::Kinematic,
            Collider::capsule(0.45, 1.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            Transform::from_translation(spawn_position + Vec3::Y * CHOKY_BODY_Y,),
            DespawnOnExit(GameScene::Desert),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(choky_scene),
                Transform::from_xyz(0.0,-CHOKY_BODY_Y,0.0,),
                ApplyToonMaterial
            ));
        })
        .id();

    commands.entity(enemy_choky).insert((
        CombatTarget,
        ChokyAttackTimer(Timer::from_seconds(0.3,TimerMode::Repeating,),),
    ));

    spawn_enemy_health_bar(
        &mut commands,
        enemy_choky,
    );

    has_spawned.0 = true;
}

fn reset_choky_spawn_once(mut has_spawned: ResMut<ChokyHasSpawned>) {
    has_spawned.0 = false;
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

fn enemy_choky_chase_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<
        &Transform,
        (With<Player>, Without<EnemyChoky>),
    >,
    mut choky_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut EnemyState,
            Option<&EnemyInvestigateDirection>,
            Option<&ChokyHuntSoundPlayed>,
            Option<&CriticalKnockback>,
        ),
        (With<EnemyChoky>, Without<Player>),
    >,
    muamua_defeated: Res<MuamuaDefeatedCount>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let muamua_completed =
        muamua_defeated.0 >= MUAMUA_DEFEAT_GOAL;

    for (
        choky_entity,
        mut choky_transform,
        mut velocity,
        mut enemy_state,
        investigate,
        hunt_sound_played,
        critical_knockback,
    ) in &mut choky_query
    {
        if investigate.is_some() {
            commands
                .entity(choky_entity)
                .remove::<EnemyInvestigateDirection>();
        }

        // ถ้าเจ็บหรือตายอยู่ ยังไม่ให้ไล่หรือเดินกลับ
        // แต่ถ้ามี CriticalKnockback อยู่ ให้ปล่อยกระเด็นก่อน
        if matches!(
            *enemy_state,
            EnemyState::Hurt | EnemyState::Dead
        ) {
            if critical_knockback.is_none() {
                velocity.x = 0.0;
                velocity.z = 0.0;
                velocity.y = 0.0;
            }

            continue;
        }

        // ระยะจาก Choky ไป Player
        let to_player =
            player_transform.translation - choky_transform.translation;

        let flat_to_player = Vec3::new(
            to_player.x,
            0.0,
            to_player.z,
        );

        let player_distance = flat_to_player.length();

        // ระยะจาก Choky ไปจุดเกิด
        let to_spawn =
            CHOKY_SPAWN_POSITION - choky_transform.translation;

        let flat_to_spawn = Vec3::new(
            to_spawn.x,
            0.0,
            to_spawn.z,
        );

        let spawn_distance = flat_to_spawn.length();

        let player_in_chase_radius =
            player_distance <= CHOKY_CHASE_RANGE;

        // ถ้า Muamua ถูกกำจัดครบแล้ว ให้ไล่ทันทีไม่จำกัดรัศมี
        // ถ้ายังไม่ครบ จะไล่เฉพาะเมื่อผู้เล่นอยู่ในระยะ 10 เมตร
        let should_chase_player =
            muamua_completed || player_in_chase_radius;

        if should_chase_player {
            // เล่นเสียงครั้งเดียวตอนเริ่มล่า
            if hunt_sound_played.is_none() {
                commands.spawn((
                    AudioPlayer::new(
                        asset_server
                            .load("sounds/enemy/choky_start.ogg"),
                    ),
                    PlaybackSettings::DESPAWN,
                ));

                commands
                    .entity(choky_entity)
                    .insert(ChokyHuntSoundPlayed);
            }

            // ถ้าใกล้พอในระยะโจมตี ให้หยุดแล้ว Attack
            if player_distance <= CHOKY_STOP_DISTANCE {
                velocity.x = 0.0;
                velocity.z = 0.0;

                if flat_to_player.length_squared() > 0.0001 {
                    let direction = flat_to_player.normalize();

                    choky_transform.rotation =
                        Quat::from_rotation_y(
                            direction.x.atan2(direction.z),
                        );
                }

                *enemy_state = EnemyState::Attack;
                continue;
            }

            // ไล่ตามผู้เล่น
            let direction = flat_to_player.normalize();

            velocity.x = direction.x * CHOKY_MOVE_SPEED;
            velocity.z = direction.z * CHOKY_MOVE_SPEED;

            choky_transform.rotation =
                Quat::from_rotation_y(
                    direction.x.atan2(direction.z),
                );

            *enemy_state = EnemyState::Chase;
        } else {
            // ผู้เล่นอยู่นอกระยะไล่ล่า และ Muamua ยังไม่ครบ
            // ให้เดินกลับจุดเกิด

            // ถ้าต้องการให้เสียงเริ่มล่าเล่นใหม่ทุกครั้งที่ผู้เล่นเข้าระยะอีกครั้ง
            // ให้ลบ ChokyHuntSoundPlayed ตอนกลับจุดเกิด
            if hunt_sound_played.is_some() {
                commands
                    .entity(choky_entity)
                    .remove::<ChokyHuntSoundPlayed>();
            }

            // ถ้าถึงจุดเกิดแล้ว ให้ยืนเฉย ๆ
            if spawn_distance <= CHOKY_RETURN_STOP_DISTANCE {
                velocity.x = 0.0;
                velocity.z = 0.0;

                *enemy_state = EnemyState::Idle;
                continue;
            }

            // เดินกลับจุดเกิด
            let direction = flat_to_spawn.normalize();

            velocity.x = direction.x * CHOKY_MOVE_SPEED;
            velocity.z = direction.z * CHOKY_MOVE_SPEED;

            choky_transform.rotation =
                Quat::from_rotation_y(
                    direction.x.atan2(direction.z),
                );

            // ใช้ Chase เพื่อให้เล่นอนิเมชันขณะเดินกลับ
            *enemy_state = EnemyState::Chase;
        }
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
                commands.spawn((AudioPlayer::new(asset_server.load("sounds/enemy/choky_hurt.ogg")),PlaybackSettings::DESPAWN));
            }

            EnemyChokyAnimState::Dead => {
                animation_player.play(animation_graph.dead);
                commands.spawn((AudioPlayer::new(asset_server.load("sounds/enemy/choky_dead.ogg")),PlaybackSettings::DESPAWN));
            }
        }

        *current_animation = wanted_animation;
    }
}

#[allow(clippy::too_many_arguments)]
fn update_choky_hurt_and_dead(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut choky_query: Query<
        (
            Entity,
            &mut Transform,
            &mut EnemyState,
            &mut EnemyStateTimer,
            &mut LinearVelocity,
            Option<&ChokyDeathEffectSpawned>,
            Option<&mut CriticalKnockback>,
        ),
        With<EnemyChoky>,
    >,
    player_query: Query<&GlobalTransform, With<Player>>,
    warp_query: Query<(), With<WarpToFloatingIsland>>,
) {
    for (
        choky_entity,
        mut choky_transform,
        mut enemy_state,
        mut state_timer,
        mut velocity,
        death_effect_spawned,
        mut critical_knockback,
    ) in &mut choky_query
    {
         state_timer.0.tick(time.delta());

        let mut allow_knockback = false;
        let mut finished_ground_y: Option<f32> = None;

        if let Some(mut kb) = critical_knockback {
            kb.timer.tick(time.delta());

            if kb.timer.just_finished() {
                finished_ground_y = Some(kb.ground_y);

                commands
                    .entity(choky_entity)
                    .remove::<CriticalKnockback>();
            } else {
                allow_knockback = true;

                let elapsed = kb.timer.elapsed_secs();
                let duration = kb.timer.duration().as_secs_f32();
                let half_duration = duration * 0.5;

                // ครึ่งแรกให้ลอยขึ้น ครึ่งหลังให้ตกลง
                if elapsed < half_duration {
                    velocity.y = CHOKY_CRITICAL_KNOCKBACK_UP;
                } else {
                    velocity.y = -CHOKY_CRITICAL_KNOCKBACK_UP;
                }

                // ถ้าลงมาถึงพื้นเดิมแล้ว ให้หยุดที่พื้นเดิม
                if choky_transform.translation.y <= kb.ground_y {
                    choky_transform.translation.y = kb.ground_y;
                    velocity.y = 0.0;
                }
            }
        }

        // ถ้าไม่ได้กระเด็นคริติคอล ให้หยุดนิ่ง
        if !allow_knockback {
            velocity.x = 0.0;
            velocity.z = 0.0;
            velocity.y = 0.0;

            if let Some(ground_y) = finished_ground_y {
                choky_transform.translation.y = ground_y;
            }
        }


        // เมื่อ Choky ตาย ให้สร้าง effect ก่อน แล้วค่อยสร้างวาร์ป
        if matches!(*enemy_state, EnemyState::Dead)
            && death_effect_spawned.is_none()
            && warp_query.is_empty()
        {
            // ป้องกันไม่ให้สร้างซ้ำ
            commands
                .entity(choky_entity)
                .insert(ChokyDeathEffectSpawned);

            // ---------------------------------------------------------
            // 1) Cylinder mesh สีจาง ๆ ก่อนสร้างวาร์ป
            // ---------------------------------------------------------
            let cylinder_material = materials.add(StandardMaterial {
                base_color: Color::srgba(0.65, 0.9, 1.0, 0.22),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            commands.spawn((
                Name::new("Choky Warp Cylinder"),
                Mesh3d(meshes.add(Cylinder::new(2.0, 0.05))),
                MeshMaterial3d::<StandardMaterial>(cylinder_material),
                Transform::from_xyz(
                    CHOKY_WARP_POSITION.x,
                    0.05,
                    CHOKY_WARP_POSITION.z,
                ),
                DespawnOnExit(GameScene::Desert),
            ));

            // ---------------------------------------------------------
            // 2) ลูกศรจากผู้เล่น ชี้ไปที่แท่นวาร์ป
            // ---------------------------------------------------------
            if let Some(player_transform) = player_query.iter().next() {
                // จุดเริ่มของลูกศร: เหนือหัวผู้เล่นเล็กน้อย
                let start = player_transform.translation() + Vec3::Y * 2.2;

                // เป้าหมายคือแท่นวาร์ป
                let target = CHOKY_WARP_POSITION;

                let mut direction = target - start;

                if direction.length_squared() > 0.0001 {
                    direction = direction.normalize();

                    // ลูกศรถูกสร้างให้ชี้ตามแกน +Y
                    // จึงหมุนจาก Vec3::Y ไปยังทิศทางเป้าหมาย
                    let rotation = if direction.dot(Vec3::Y) < -0.9999 {
                        Quat::from_rotation_x(std::f32::consts::PI)
                    } else {
                        Quat::from_rotation_arc(Vec3::Y, direction)
                    };

                    let arrow_material = materials.add(StandardMaterial {
                        base_color: Color::srgba(1.0, 0.0, 0.0, 0.95),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    });

                    // ท่อยาว 1.5 เมตร => half_height = 0.75
                    let shaft_mesh = meshes.add(Cylinder::new(0.05, 0.75));

                    // หัวลูกศร ใช้ Cone แทนรูปทรงสามเหลี่ยม/หัวศร
                    let head_mesh = meshes.add(Cone::new(0.18, 0.35));

                    commands
                        .spawn((
                            Name::new("Choky Warp Arrow"),
                            ChokyTemporaryLifetime(Timer::from_seconds(
                                5.0,
                                TimerMode::Once,
                            )),
                            DespawnOnExit(GameScene::Desert),
                            Transform::from_translation(start).with_rotation(rotation),
                        ))
                        .with_children(|parent| {
                            // ตัวท่อ
                            parent.spawn((
                                Name::new("Arrow Shaft"),
                                Mesh3d(shaft_mesh),
                                MeshMaterial3d::<StandardMaterial>(arrow_material.clone()),
                                Transform::from_xyz(0.0, 0.75, 0.0),
                            ));

                            // หัวลูกศร
                            parent.spawn((
                                Name::new("Arrow Head"),
                                Mesh3d(head_mesh),
                                MeshMaterial3d::<StandardMaterial>(arrow_material),
                                Transform::from_xyz(0.0, 1.5 + 0.175, 0.0),
                            ));
                        });
                }
            }

            // ---------------------------------------------------------
            // สร้างวาร์ป หลังสั่งสร้าง cylinder และลูกศร
            // ---------------------------------------------------------
            commands.spawn((
                CurrentScene,
                WarpToFloatingIsland,
                Collider::cuboid(2.0, 2.0, 1.0),
                Transform::from_translation(CHOKY_WARP_POSITION),
                DespawnOnExit(GameScene::Desert),
            ));
        }

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
                commands.entity(choky_entity).despawn();
            }
            _ => {
                commands
                    .entity(choky_entity)
                    .remove::<EnemyStateTimer>();
            }
        }
    }
}

fn despawn_choky_temporary(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ChokyTemporaryLifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0.tick(time.delta());

        if lifetime.0.just_finished() {
            commands.entity(entity).despawn();
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
        commands.spawn((AudioPlayer::new(asset_server.load("sounds/enemy/choky_atk.ogg")),PlaybackSettings::DESPAWN));
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