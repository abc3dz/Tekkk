use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::animation::graph::AnimationGraph;
use bevy::animation::AnimationPlayer;
use avian3d::prelude::*;
use bevy_wind_waker_shader::prelude::*;

use crate::components::*;
use crate::combat::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            setup_player_animation_graph, 
            spawn_player,
            setup_player_status_ui
        ))
            .add_systems(Update, (
                setup_player_animation_player,
                player_movement,
                player_jump_input,
                player_jump_update,
                player_combo_input,
                player_dash_move,
                spawn_player_dash_trail_during_dash,
                player_combo_update,
                player_dash_input,
                player_dash_update,
                update_player_dash_effect,
                update_player_animation,
                update_player_status_ui,
                player_punch_damage,
                rebuild_player_combat_stats_from_exp,
                update_player_status_ui,
                update_floating_damage_text,
                update_basic_gun_defeat_particles,
                player_return_after_hurt,
            ).chain()
        );
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let base_stats = BaseStats::PLAYER;

    commands
    .spawn((
        Player,
        MoveSpeed(6.0),
        Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },
        Mana {
            current: base_stats.max_mp as i32,
            max: base_stats.max_mp as i32,
        },
        base_stats,
        CombatStats::from(base_stats),
        AtkAndDefElement(Element::Inw),
        ElementMastery::default(),
        PlayerCombo {
            current_index: None,
            queued_next: false,
            timer: Timer::from_seconds(0.0, TimerMode::Once),
        },
        Transform::from_xyz(0.0, 2.0, 0.0),
        RigidBody::Dynamic,
        Collider::capsule(0.28, 1.0),
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity::ZERO,
        
    ))
    .with_children(|parent| {
        parent.spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("player/PlayerMoya.glb")
                )
            ),
            Transform::from_xyz(0.0, -0.83, 0.0),
            WindWakerShaderBuilder::default()
            .time_of_day(TimeOfDay::Day)
            .weather(Weather::Sunny)
            .build(),
        ));
    });
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &MoveSpeed,
        &mut LinearVelocity,
        &mut Transform,
    ), With<Player>>,
) {
    let Ok((speed, mut velocity, mut transform)) = query.single_mut() else {
        return;
    };

    let mut dir = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) { dir.z -= 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { dir.z += 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { dir.x += 1.0; }

    if dir.length_squared() > 0.0 {
        dir = dir.normalize();

        velocity.x = dir.x * speed.0;
        velocity.z = dir.z * speed.0;

        transform.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z));
    } else {
        velocity.x = 0.0;
        velocity.z = 0.0;
    }

    if transform.translation.y < -5.0 {
        transform.translation = Vec3::new(0.0, 2.0, 0.0);
        velocity.x = 0.0;
        velocity.y = 0.0;
        velocity.z = 0.0;
    }
}

fn setup_player_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(1).from_asset("player/PlayerMoya.glb")),
        1.0,
        graph.root,
    );

    let walk = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(4).from_asset("player/PlayerMoya.glb")),
        1.0,
        graph.root,
    );
    let slap_r = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(3).from_asset("player/PlayerMoya.glb")),
        1.0,
        graph.root,
    );
    let slap_l = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(2).from_asset("player/PlayerMoya.glb")),
        1.0,
        graph.root,
    );
    let dash = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("player/PlayerMoya.glb")),
        1.0,
        graph.root,
    );
    let jump = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(6).from_asset("player/PlayerMoya.glb")),
        1.0,
        graph.root,
    );
    let hurt = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(7).from_asset("player/PlayerMoya.glb")),
        1.0,
        graph.root,
    );
    let slap_lr = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(5).from_asset("player/PlayerMoya.glb")),
        1.0,
        graph.root,
    );

    let graph_handle = graphs.add(graph);

    commands.insert_resource(PlayerAnimationGraph {
        graph: graph_handle,
        idle,
        walk,
        slap_r,
        slap_l,
        slap_lr,
        dash,
        jump,
        hurt,
    });
}

fn setup_player_animation_player(
    mut commands: Commands,
    anim_graph: Res<PlayerAnimationGraph>,
    player_root_query: Query<&Children, With<Player>>,
    children_query: Query<&Children>,
    mut anim_query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    let Ok(player_children) = player_root_query.single() else {
        return;
    };

    let mut player_anim_entity: Option<Entity> = None;

    for child in player_children.iter() {
        find_animation_player_recursive(
            child,
            &children_query,
            &anim_query,
            &mut player_anim_entity,
        );
    }

    let Some(target_entity) = player_anim_entity else {
        return;
    };

    if let Ok((entity, mut player)) = anim_query.get_mut(target_entity) {
        commands.entity(entity).insert((
            AnimationGraphHandle(anim_graph.graph.clone()),
            PlayerAnimState::Idle,
            PlayerAnimationTarget,
        ));

        player.play(anim_graph.idle).repeat();
    }
}

pub fn player_dash_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    anim_graph: Res<PlayerAnimationGraph>,
    player_query: Query<(Entity, &Transform, &PlayerCombo), With<Player>>,
    mut anim_query: Query<
        (Entity, &mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
) {
    if !keyboard.just_pressed(KeyCode::KeyL) {
        return;
    }

    let Ok((player_entity, player_tf, combo)) = player_query.single() else {
        return;
    };

    if combo.current_index.is_some() {
        return;
    }

    let Ok((entity, mut anim_player, mut anim_state)) = anim_query.single_mut() else {
        return;
    };

    println!("Player dash");

    anim_player.stop_all();
    anim_player.play(anim_graph.dash);

    *anim_state = PlayerAnimState::Dash;


    commands.entity(entity).insert(
        PlayerDashTimer(Timer::from_seconds(0.45, TimerMode::Once))
    );

    let dash_direction = player_tf.rotation * Vec3::Z;

    commands.entity(player_entity).insert((
        PlayerDashMove {
            timer: Timer::from_seconds(0.25, TimerMode::Once),
            direction: dash_direction.normalize(),
            speed: 14.0,
        },
        PlayerDashTrailTimer(
            Timer::from_seconds(0.05, TimerMode::Repeating)
        ),
    ));
}
pub fn player_dash_update(
    mut commands: Commands,
    time: Res<Time>,
    anim_graph: Res<PlayerAnimationGraph>,
    mut anim_query: Query<
        (Entity, &mut AnimationPlayer, &mut PlayerAnimState, &mut PlayerDashTimer),
        With<PlayerAnimationTarget>,
    >,
) {
    for (entity, mut anim_player, mut anim_state, mut dash_timer) in &mut anim_query {
        dash_timer.0.tick(time.delta());

        if dash_timer.0.is_finished() {
            anim_player.stop_all();
            anim_player.play(anim_graph.idle).repeat();

            *anim_state = PlayerAnimState::Idle;

            commands.entity(entity).remove::<PlayerDashTimer>();
        }
    }
}
pub fn player_jump_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    anim_graph: Res<PlayerAnimationGraph>,
    combo_query: Query<&PlayerCombo, With<Player>>,
    mut player_query: Query<&mut LinearVelocity, With<Player>>,
    mut anim_query: Query<
        (Entity, &mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
) {
    if !keyboard.just_pressed(KeyCode::KeyK) {
        return;
    }

    let Ok(combo) = combo_query.single() else {
        return;
    };

    // ถ้ากำลัง combo อยู่ ไม่ให้ jump ทับ punch
    if combo.current_index.is_some() {
        return;
    }

    let Ok(mut velocity) = player_query.single_mut() else {
        return;
    };

    let Ok((anim_entity, mut anim_player, mut anim_state)) = anim_query.single_mut() else {
        return;
    };

    // กันกระโดดรัวกลางอากาศแบบง่าย ๆ
    if velocity.y.abs() > 0.1 {
        return;
    }

    velocity.y = 7.0;

    anim_player.stop_all();
    anim_player.play(anim_graph.jump);

    *anim_state = PlayerAnimState::Jump;

    commands.entity(anim_entity).insert(
        PlayerJumpTimer(Timer::from_seconds(0.6, TimerMode::Once))
    );

    println!("Player jump");
}
pub fn player_jump_update(
    mut commands: Commands,
    time: Res<Time>,
    anim_graph: Res<PlayerAnimationGraph>,
    player_query: Query<&LinearVelocity, With<Player>>,
    mut anim_query: Query<
        (
            Entity,
            &mut AnimationPlayer,
            &mut PlayerAnimState,
            &mut PlayerJumpTimer,
        ),
        With<PlayerAnimationTarget>,
    >,
) {
    let Ok(velocity) = player_query.single() else {
        return;
    };

    let is_moving = velocity.x.abs() > 0.01 || velocity.z.abs() > 0.01;

    for (entity, mut anim_player, mut anim_state, mut jump_timer) in &mut anim_query {
        jump_timer.0.tick(time.delta());

        if !jump_timer.0.is_finished() {
            continue;
        }

        anim_player.stop_all();

        if is_moving {
            anim_player.play(anim_graph.walk).repeat();
            *anim_state = PlayerAnimState::Walk;
        } else {
            anim_player.play(anim_graph.idle).repeat();
            *anim_state = PlayerAnimState::Idle;
        }

        commands.entity(entity).remove::<PlayerJumpTimer>();
    }
}
pub fn player_dash_move(
    mut commands: Commands,
    time: Res<Time>,
    mut player_query: Query<(Entity, &mut LinearVelocity, &mut PlayerDashMove), With<Player>>,
) {
    for (entity, mut velocity, mut dash_move) in &mut player_query {
        dash_move.timer.tick(time.delta());

        velocity.x = dash_move.direction.x * dash_move.speed;
        velocity.z = dash_move.direction.z * dash_move.speed;

        if dash_move.timer.is_finished() {
            commands.entity(entity).remove::<PlayerDashMove>();
            commands.entity(entity).remove::<PlayerDashTrailTimer>();

            velocity.x = 0.0;
            velocity.z = 0.0;
        }
    }
}
pub fn update_player_dash_effect(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_query: Query<(Entity, &mut PlayerDashEffect)>,
) {
    for (entity, mut effect) in &mut effect_query {
        effect.timer.tick(time.delta());

        if effect.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
pub fn spawn_player_dash_trail_during_dash(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<
        (&Transform, &mut PlayerDashTrailTimer),
        With<PlayerDashMove>,
    >,
) {
    for (player_tf, mut trail_timer) in &mut player_query {
        trail_timer.0.tick(time.delta());

        if !trail_timer.0.just_finished() {
            continue;
        }

        commands.spawn((
            PlayerDashEffect {
                timer: Timer::from_seconds(0.35, TimerMode::Once),
            },
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("player/PlayerMoyaDash.glb")
                )
            ),
            Transform {
                translation: player_tf.translation + Vec3::Y * -1.0,
                rotation: player_tf.rotation,
                scale: Vec3::splat(1.0),
            },
            GlobalTransform::default(),
        ));
    }
}
pub fn play_player_hurt_animation(
    commands: &mut Commands,
    player_entity: Entity,
    anim_graph: &PlayerAnimationGraph,
    anim_query: &mut Query<
        (&mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
) {
    let Ok((mut anim_player, mut anim_state)) = anim_query.single_mut() else {
        return;
    };

    // ถ้ากำลัง Hurt อยู่ ไม่ต้องเริ่มใหม่ทุกเฟรม
    if *anim_state == PlayerAnimState::Hurt {
        return;
    }

    anim_player.stop_all();
    anim_player.play(anim_graph.hurt);

    *anim_state = PlayerAnimState::Hurt;

    commands.entity(player_entity).insert(
        PlayerHurtTimer(Timer::from_seconds(
            0.5,
            TimerMode::Once,
        )),
    );
}
pub fn player_return_after_hurt(
    mut commands: Commands,
    anim_graph: Res<PlayerAnimationGraph>,

    player_query: Query<
        Entity,
        (With<Player>, With<PlayerHurtTimer>),
    >,

    mut anim_query: Query<
        (&mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    let Ok((mut anim_player, mut anim_state)) =
        anim_query.single_mut()
    else {
        return;
    };

    if *anim_state != PlayerAnimState::Hurt {
        return;
    }

    let hurt_finished = anim_player
        .animation(anim_graph.hurt)
        .is_some_and(|animation| animation.is_finished());

    if !hurt_finished {
        return;
    }

    anim_player.stop_all();
    anim_player.play(anim_graph.idle).repeat();

    *anim_state = PlayerAnimState::Idle;

    commands
        .entity(player_entity)
        .remove::<PlayerHurtTimer>();
}

pub fn player_combo_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    anim_graph: Res<PlayerAnimationGraph>,
    mut combo_query: Query<&mut PlayerCombo, With<Player>>,
    mut anim_query: Query<
        (&mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
) {
    if !keyboard.just_pressed(KeyCode::KeyJ) {
        return;
    }

    let Ok(mut combo) = combo_query.single_mut() else {
        return;
    };

    let Ok((mut anim_player, mut anim_state)) = anim_query.single_mut() else {
        return;
    };

    if *anim_state == PlayerAnimState::Hurt {
        return;
    }

    if combo.current_index.is_none() {
        start_player_combo_attack(
            0,
            &anim_graph,
            &mut anim_player,
            &mut anim_state,
            &mut combo,
        );
    } else {
        combo.queued_next = true;
        println!("Queue next combo");
    }
}

pub fn player_combo_update(
    time: Res<Time>,
    anim_graph: Res<PlayerAnimationGraph>,
    mut combo_query: Query<&mut PlayerCombo, With<Player>>,
    mut anim_query: Query<
        (&mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
) {
    let Ok(mut combo) = combo_query.single_mut() else {
        return;
    };

    let Some(current_index) = combo.current_index else {
        return;
    };

    combo.timer.tick(time.delta());

    if !combo.timer.is_finished() {
        return;
    }

    let Ok((mut anim_player, mut anim_state)) = anim_query.single_mut() else {
        return;
    };

    let next_index = current_index + 1;

    if combo.queued_next && next_index < PLAYER_COMBO_COUNT {
        start_player_combo_attack(
            next_index,
            &anim_graph,
            &mut anim_player,
            &mut anim_state,
            &mut combo,
        );
    } else {
        anim_player.stop_all();
        anim_player.play(anim_graph.idle).repeat();

        *anim_state = PlayerAnimState::Idle;

        combo.current_index = None;
        combo.queued_next = false;

        println!("Combo finished");
    }
}

fn find_animation_player_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    anim_query: &Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    result: &mut Option<Entity>,
) {
    if result.is_some() {
        return;
    }

    if anim_query.get(entity).is_ok() {
        *result = Some(entity);
        return;
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            find_animation_player_recursive(child, children_query, anim_query, result);
        }
    }
}

fn update_player_animation(
    anim_graph: Res<PlayerAnimationGraph>,
    player_query: Query<&LinearVelocity, With<Player>>,
    mut anim_query: Query<
        (&mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
) {
    let Ok(velocity) = player_query.single() else {
        return;
    };

    let is_moving = velocity.x.abs() > 0.01 || velocity.z.abs() > 0.01;

    for (mut player, mut anim_state) in &mut anim_query {
        if matches!(
            *anim_state,
            PlayerAnimState::SlapR | PlayerAnimState::SlapL | PlayerAnimState::SlapLR | PlayerAnimState::Jump | PlayerAnimState::Hurt
        ) {
            continue;
        }

        if is_moving {
            if *anim_state != PlayerAnimState::Walk {
                player.stop_all();
                player.play(anim_graph.walk).repeat();
                *anim_state = PlayerAnimState::Walk;
            }
        } else {
            if *anim_state != PlayerAnimState::Idle {
                player.stop_all();
                player.play(anim_graph.idle).repeat();
                *anim_state = PlayerAnimState::Idle;
            }
        }
    }
}

pub fn setup_player_status_ui(mut commands: Commands) {
    commands
        .spawn((
            PlayerStatusUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(260.0),
                height: Val::Px(70.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Health bar background
            parent
                .spawn((
                    Node {
                        width: Val::Px(240.0),
                        height: Val::Px(24.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        HealthBarFill,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.8, 0.1, 0.1)),
                    ));
                });

            // Mana bar background
            parent
                .spawn((
                    Node {
                        width: Val::Px(240.0),
                        height: Val::Px(24.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        ManaBarFill,
                        Node {
                            width: Val::Percent(50.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.2, 0.9)),
                    ));
                });
        });
}
pub fn update_player_status_ui(
    player_query: Query<(&Health, &Mana), With<Player>>,
    mut health_bar_query: Query<&mut Node, (With<HealthBarFill>, Without<ManaBarFill>)>,
    mut mana_bar_query: Query<&mut Node, (With<ManaBarFill>, Without<HealthBarFill>)>,
) {
    let Ok((health, mana)) = player_query.single() else {
        return;
    };

    let health_percent = health.current as f32 / health.max as f32 * 100.0;
    let mana_percent = mana.current as f32 / mana.max as f32 * 100.0;

    for mut node in &mut health_bar_query {
        node.width = Val::Percent(health_percent.clamp(0.0, 100.0));
    }

    for mut node in &mut mana_bar_query {
        node.width = Val::Percent(mana_percent.clamp(0.0, 100.0));
    }
}

const PLAYER_COMBO_COUNT: usize = 3;

fn combo_duration(index: usize) -> f32 {
    match index {
        0 => 0.45, // SlapR
        1 => 0.45, // SlapL
        2 => 0.65, // SlapLR
        _ => 0.45,
    }
}

fn combo_anim_node(
    anim_graph: &PlayerAnimationGraph,
    index: usize,
) -> AnimationNodeIndex {
    match index {
        0 => anim_graph.slap_r,
        1 => anim_graph.slap_l,
        2 => anim_graph.slap_lr,
        _ => anim_graph.slap_r,
    }
}

fn combo_anim_state(index: usize) -> PlayerAnimState {
    match index {
        0 => PlayerAnimState::SlapR,
        1 => PlayerAnimState::SlapL,
        2 => PlayerAnimState::SlapLR,
        _ => PlayerAnimState::SlapR,
    }
}

fn start_player_combo_attack(
    index: usize,
    anim_graph: &PlayerAnimationGraph,
    anim_player: &mut AnimationPlayer,
    anim_state: &mut PlayerAnimState,
    combo: &mut PlayerCombo,
) {
    anim_player.stop_all();
    anim_player.play(combo_anim_node(anim_graph, index));

    *anim_state = combo_anim_state(index);

    combo.current_index = Some(index);
    combo.queued_next = false;
    combo.timer = Timer::from_seconds(combo_duration(index), TimerMode::Once);

    //println!("Combo attack {}", index);
}
fn spawn_floating_damage_text(
    commands: &mut Commands,
    damage: i32,
    position: Vec3,
) {
    commands.spawn((
        FloatingDamageText {
            timer: Timer::from_seconds(0.8, TimerMode::Once),
            world_position: position,
            velocity: Vec3::new(0.0, 1.5, 0.0),
        },
        Text::new(format!("-{}", damage)),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.2, 0.1)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
    ));
}

pub fn update_floating_damage_text(
    mut commands: Commands,
    time: Res<Time>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut text_query: Query<(Entity, &mut Node, &mut FloatingDamageText)>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for (entity, mut node, mut floating_text) in &mut text_query {
        floating_text.timer.tick(time.delta());

        let velocity = floating_text.velocity;
        let delta_seconds = time.delta_secs();

        floating_text.world_position += velocity * delta_seconds;

        if floating_text.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let Ok(screen_pos) = camera.world_to_viewport(
            camera_transform,
            floating_text.world_position,
        ) else {
            continue;
        };

        node.left = Val::Px(screen_pos.x);
        node.top = Val::Px(screen_pos.y);
    }
}

fn pseudo_random(seed: f32) -> f32 {
    (seed.sin() * 43758.5453).fract().abs()
}

fn spawn_defeat_particles(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    seed: f32,
) {
    for i in 0..12 {
        let r1 = pseudo_random(seed + i as f32 * 1.37);
        let r2 = pseudo_random(seed + i as f32 * 2.11);
        let r3 = pseudo_random(seed + i as f32 * 3.73);

        let size = 0.08 + r1 * 0.22;

        let offset = Vec3::new(
            (r2 - 0.5) * 1.2,
            0.3 + r1 * 0.4,
            (r3 - 0.5) * 1.2,
        );

        let velocity = Vec3::new(
            (r2 - 0.5) * 0.6,
            1.2 + r1 * 1.4,
            (r3 - 0.5) * 0.6,
        );

        commands.spawn((
            BasicGunDefeatParticle {
                velocity,
                lifetime: Timer::from_seconds(0.8 + r3 * 0.5, TimerMode::Once),
            },
            Mesh3d(meshes.add(Sphere::new(size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, 0.45),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(position + offset),
            GlobalTransform::default(),
        ));
    }
}
pub fn update_basic_gun_defeat_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particle_query: Query<(Entity, &mut Transform, &mut BasicGunDefeatParticle)>,
) {
    for (entity, mut transform, mut particle) in &mut particle_query {
        particle.lifetime.tick(time.delta());

        transform.translation += particle.velocity * time.delta_secs();

        if particle.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
fn calculate_combat_damage(
    attacker: &CombatStats,
    defender: &CombatStats,
) -> i32 {
    let damage = attacker.attack
        * 100.0
        / (100.0 + defender.defense.max(0.0));

    damage.round().max(1.0) as i32
}
pub fn player_punch_damage(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut player_query: Query<
        (
            &Transform,
            &CombatStats,
            &mut ElementMastery,
        ),
        (
            With<Player>,
            Without<CombatTarget>,
        ),
    >,

    mut target_query: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            &CombatStats,
            Option<&ElementExpReward>,
        ),
        (
            With<CombatTarget>,
            Without<Player>,
        ),
    >,
) {
    if !keyboard.just_pressed(KeyCode::KeyJ) {
        return;
    }

    let Ok((
        player_transform,
        player_stats,
        mut element_mastery,
    )) = player_query.single_mut()
    else {
        return;
    };

    let mut rng = rand::rng();

    for (
        target_entity,
        target_transform,
        mut target_health,
        target_stats,
        reward,
    ) in &mut target_query
    {
        // กัน Entity ที่ถูก defeat แล้ว
        if target_health.current <= 0 {
            continue;
        }

        let distance = player_transform
            .translation
            .distance(target_transform.translation);

        if distance > 2.0 {
            continue;
        }

        let damage = calculate_combat_damage(
            player_stats,
            target_stats,
        );

        target_health.current -= damage;
        target_health.current =
            target_health.current.clamp(
                0,
                target_health.max,
            );

        spawn_floating_damage_text(
            &mut commands,
            damage,
            target_transform.translation
                + Vec3::new(0.0, 2.0, 0.0),
        );

        // ยังไม่ถูก defeat
        if target_health.current > 0 {
            continue;
        }

        println!("Combat target defeated");

        // แจก Reward หาก Entity มี ElementExpReward
        if let Some(reward) = reward {
            let gain = reward.grant_all(
                &mut element_mastery,
                &mut rng,
            );

            println!(
                "Element EXP: Water +{}, Fire +{}, Wind +{}, Earth +{}, Inw +{}",
                gain.water,
                gain.fire,
                gain.wind,
                gain.earth,
                gain.inw,
            );
        }

        spawn_defeat_particles(
            &mut commands,
            &mut meshes,
            &mut materials,
            target_transform.translation + Vec3::Y,
            time.elapsed_secs(),
        );

        commands.entity(target_entity).despawn();
    }
}
pub fn rebuild_player_combat_stats_from_exp(
    mut player_query: Query<
        (
            &BaseStats,
            &ElementMastery,
            &mut CombatStats,
            &mut Health,
            &mut Mana,
        ),
        (
            With<Player>,
            Changed<ElementMastery>,
        ),
    >,
) {
    for (
        base,
        mastery,
        mut combat,
        mut health,
        mut mana,
    ) in &mut player_query
    {
        let new_stats =
            combat_stats_from_element_exp(base, mastery);

        *combat = new_stats;

        let new_hp_max =
            new_stats.max_hp.round() as i32;

        let new_mp_max =
            new_stats.max_mp.round() as i32;

        health.max = new_hp_max;
        health.current =
            health.current.clamp(0, new_hp_max);

        mana.max = new_mp_max;
        mana.current =
            mana.current.clamp(0, new_mp_max);
    }
}