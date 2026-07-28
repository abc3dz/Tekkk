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
    fn build(&self, app: &mut App) {app
        .add_systems(Startup, (
            setup_player_animation_graph, 
            spawn_player,
            setup_player_status_ui
        ))
        .add_systems(Update, (
            setup_player_animation_player,
            player_movement,
            player_footstep_sound,
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
            rebuild_player_combat_stats_from_exp,
            update_floating_damage_text,
            update_basic_gun_defeat_particles,
            player_return_after_hurt,
            respawn_player_when_defeated,
            ).chain()
        )
        .add_systems(Update,(
                tag_player_hand_bones,
                spawn_requested_player_punch_hitboxes,
                update_player_punch_hitbox_lifetime,
            ).chain()
        )
        .add_systems(
            Update,
            (
                spawn_player_slap_hitbox,
                player_slap_hit_enemy,
                despawn_player_slap_hitbox,
            )
                .chain(),
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
        Health {current: base_stats.max_hp as i32, max: base_stats.max_hp as i32},
        Mana {current: base_stats.max_mp as i32, max: base_stats.max_mp as i32},
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
    )).with_children(|parent| {
        parent.spawn((SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("player/PlayerMoya.glb"))),
            Transform::from_xyz(0.0, -0.83, 0.0),
            WindWakerShaderBuilder::default().time_of_day(TimeOfDay::Day).weather(Weather::Sunny).build(),
        ));
    });
}

const GAMEPAD_DEADZONE: f32 = 0.20;

fn apply_gamepad_deadzone(
    input: Vec2,
    deadzone: f32,
) -> Vec2 {
    let length = input.length();

    if length <= deadzone {
        return Vec2::ZERO;
    }

    let adjusted_length =((length - deadzone) / (1.0 - deadzone)).clamp(0.0, 1.0);

    input.normalize() * adjusted_length
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut player_query: Query<(&MoveSpeed, &mut LinearVelocity, &mut Transform), With<Player>>,
    dialog_query: Query<(), With<GuardianDialogUI>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {

    let Ok((speed, mut velocity, mut transform)) = player_query.single_mut()
    else { return };
    if !dialog_query.is_empty() {
        velocity.x = 0.0;
        velocity.z = 0.0;
        return;
    }

    let mut movement_input = Vec2::ZERO;

    // Keyboard
    if keyboard.pressed(KeyCode::KeyW) {
        movement_input.y += 1.0;
    }

    if keyboard.pressed(KeyCode::KeyS) {
        movement_input.y -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyA) {
        movement_input.x -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyD) {
        movement_input.x += 1.0;
    }

    if let Some(gamepad) = gamepads.iter().next() {
        let left_stick = apply_gamepad_deadzone(gamepad.left_stick(), GAMEPAD_DEADZONE,);
        let dpad = gamepad.dpad();
        let gamepad_input =
            if left_stick.length_squared() > 0.0 {
                left_stick
            } else {
                dpad
            };

        movement_input += gamepad_input;
    }

    if movement_input.length_squared() > 1.0 {
        movement_input = movement_input.normalize();
    }

    let direction = Vec3::new(movement_input.x, 0.0, -movement_input.y);

    if direction.length_squared() > 0.0001 {
        velocity.x = direction.x * speed.0;
        velocity.z = direction.z * speed.0;
        transform.rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
    } else {
        velocity.x = 0.0;
        velocity.z = 0.0;
    }

    if transform.translation.y < -5.0 {
        transform.translation = Vec3::new(0.0, 2.0, 0.0);
        commands.spawn(AudioPlayer::new(asset_server.load("sounds/sfx_fall.ogg")));
        velocity.x = 0.0;
        velocity.y = 0.0;
        velocity.z = 0.0;
    }
}

pub fn player_footstep_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&LinearVelocity, With<Player>>,
    playing_sound_query: Query<(), With<PlayerFootstepSound>>,
) {
    let Ok(velocity) = player_query.single()
    else { return };

    let is_moving = velocity.x.abs() > 0.05 || velocity.z.abs() > 0.05;
    if !is_moving {
        return;
    }

    if !playing_sound_query.is_empty() {
        return;
    }

    commands.spawn((
        PlayerFootstepSound,
        AudioPlayer::new(asset_server.load("sounds/sfx_walk.ogg")),
        PlaybackSettings::DESPAWN,
    ));
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
    let Ok(player_children) = player_root_query.single() 
    else { return };

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
    mut anim_query: Query<(Entity, &mut AnimationPlayer, &mut PlayerAnimState), With<PlayerAnimationTarget>>,
    gamepads: Query<&Gamepad>,
    dialog_query: Query<(), With<GuardianDialogUI>>,
) {
    if !dialog_query.is_empty() {
        return;
    }

    let dash_pressed =
        keyboard.just_pressed(KeyCode::KeyL)
            || gamepads.iter().any(|gamepad| {
                gamepad.just_pressed(
                    GamepadButton::East,
                )
            });

    if !dash_pressed {
        return;
    }

    let Ok((player_entity, player_tf, combo)) = player_query.single() 
    else { return };

    if combo.current_index.is_some() {
        return;
    }

    let Ok((entity, mut anim_player, mut anim_state)) = anim_query.single_mut() 
    else { return };

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
        PlayerDashTrailTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
    ));
}

pub fn player_dash_update(
    mut commands: Commands,
    time: Res<Time>,
    anim_graph: Res<PlayerAnimationGraph>,
    mut anim_query: Query<(Entity, &mut AnimationPlayer, &mut PlayerAnimState, &mut PlayerDashTimer), With<PlayerAnimationTarget>>,
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
    mut anim_query: Query<(Entity, &mut AnimationPlayer, &mut PlayerAnimState), With<PlayerAnimationTarget>,>,
    gamepads: Query<&Gamepad>,
    dialog_query: Query<(), With<GuardianDialogUI>,>,
    asset_server: Res<AssetServer>
) {
    if !dialog_query.is_empty() {
        return;
    }

    let jump_pressed =
        keyboard.just_pressed(KeyCode::KeyK)
            || gamepads.iter().any(|gamepad| {
                gamepad.just_pressed(
                    GamepadButton::South,
                )
            });

    if !jump_pressed {
        return;
    }

    let Ok(combo) = combo_query.single() 
    else { return };

    if combo.current_index.is_some() {
        return;
    }

    let Ok(mut velocity) = player_query.single_mut() 
    else { return };

    let Ok((anim_entity, mut anim_player, mut anim_state)) = anim_query.single_mut() 
    else { return };

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
    commands.spawn(AudioPlayer::new(asset_server.load("sounds/sfx_jump.ogg")));
}

pub fn player_jump_update(
    mut commands: Commands,
    time: Res<Time>,
    anim_graph: Res<PlayerAnimationGraph>,
    player_query: Query<&LinearVelocity, With<Player>>,
    mut anim_query: Query<(Entity, &mut AnimationPlayer, &mut PlayerAnimState, &mut PlayerJumpTimer), With<PlayerAnimationTarget>>,
) {
    let Ok(velocity) = player_query.single() 
    else { return };
    let is_moving = velocity.x.abs() > 0.01 || velocity.z.abs() > 0.01;

    for (entity, mut anim_player, mut anim_state, mut jump_timer) in &mut anim_query {
        jump_timer.0.tick(time.delta());

        if !jump_timer.0.is_finished() { continue }

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
    mut player_query: Query<(&Transform, &mut PlayerDashTrailTimer), With<PlayerDashMove>>,
) {
    for (player_tf, mut trail_timer) in &mut player_query {
        trail_timer.0.tick(time.delta());

        if !trail_timer.0.just_finished() {
            continue;
        }

        commands.spawn((
            PlayerDashEffect {timer: Timer::from_seconds(0.35, TimerMode::Once),},
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("player/PlayerMoyaDash.glb"))),
            Transform {
                translation: player_tf.translation + Vec3::Y * -1.0,
                rotation: player_tf.rotation,
                scale: Vec3::splat(1.0),
            },
            GlobalTransform::default(),
        ));
        commands.spawn(AudioPlayer::new(asset_server.load("sounds/742717__artix0__dash-sound-effect.ogg")));
    }
}

pub fn play_player_hurt_animation(
    commands: &mut Commands,
    player_entity: Entity,
    anim_graph: &PlayerAnimationGraph,
    anim_query: &mut Query<(&mut AnimationPlayer, &mut PlayerAnimState), With<PlayerAnimationTarget>>,
) {
    let Ok((mut anim_player, mut anim_state)) = anim_query.single_mut() 
    else { return };
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
    player_query: Query<Entity, (With<Player>, With<PlayerHurtTimer>)>,
    mut anim_query: Query<(&mut AnimationPlayer, &mut PlayerAnimState), With<PlayerAnimationTarget>>,
) {
    let Ok(player_entity) = player_query.single() 
    else { return };

    let Ok((mut anim_player, mut anim_state)) =anim_query.single_mut()
    else { return };

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

    commands.entity(player_entity).remove::<PlayerHurtTimer>();
}

pub fn player_combo_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    anim_graph: Res<PlayerAnimationGraph>,
    mut combo_query: Query<(Entity, &mut PlayerCombo), With<Player>>,
    mut anim_query: Query<(&mut AnimationPlayer, &mut PlayerAnimState), With<PlayerAnimationTarget>>,
    gamepads: Query<&Gamepad>,
    dialog_query: Query<(), With<GuardianDialogUI>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if !dialog_query.is_empty() {
        return;
    }
    let attack_pressed =
        keyboard.just_pressed(KeyCode::KeyJ)
            || gamepads.iter().any(|gamepad| {
                gamepad.just_pressed(
                    GamepadButton::West,
                )
            });
    if !attack_pressed {
        return;
    }

    let Ok((player_entity, mut combo)) =
        combo_query.single_mut()
    else {
        return;
    };
    let Ok((mut anim_player, mut anim_state)) = anim_query.single_mut() else { return };

    if *anim_state == PlayerAnimState::Hurt {
        return;
    }

    if combo.current_index.is_none() {
        start_player_combo_attack(
            player_entity,
            0,
            &anim_graph,
            &mut anim_player,
            &mut anim_state,
            &mut combo,
            &mut commands,
            &asset_server,
        );
    } else {
        combo.queued_next = true;
    }
}

pub fn player_combo_update(
    time: Res<Time>,
    anim_graph: Res<PlayerAnimationGraph>,
    mut combo_query: Query<(Entity, &mut PlayerCombo), With<Player>>,
    mut anim_query: Query<(&mut AnimationPlayer, &mut PlayerAnimState), With<PlayerAnimationTarget>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok((player_entity, mut combo)) =
        combo_query.single_mut()
    else {
        return;
    };
    let Some(current_index) = combo.current_index else { return };
    combo.timer.tick(time.delta());
    if !combo.timer.is_finished() {
        return;
    }

    let Ok((mut anim_player, mut anim_state)) = anim_query.single_mut() else { return };
    let next_index = current_index + 1;
    if combo.queued_next && next_index < PLAYER_COMBO_COUNT {
        start_player_combo_attack(
            player_entity,
            next_index,
            &anim_graph,
            &mut anim_player,
            &mut anim_state,
            &mut combo,
            &mut commands,
            &asset_server,
        );
    } else {
        anim_player.stop_all();
        anim_player.play(anim_graph.idle).repeat();
        *anim_state = PlayerAnimState::Idle;
        combo.current_index = None;
        combo.queued_next = false;
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
    mut anim_query: Query<(&mut AnimationPlayer, &mut PlayerAnimState), With<PlayerAnimationTarget>>,
) {
    let Ok(velocity) = player_query.single() else {return};
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
    commands.spawn((
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
            )).with_children(|parent| {
                    parent.spawn((
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
                    parent.spawn((
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
    let Ok((health, mana)) = player_query.single() else {return;};
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
fn combo_hit_sound(index: usize) -> &'static str {
    match index {
        0 => "sounds/hit1.ogg",
        1 => "sounds/hit2.ogg",
        2 => "sounds/hit3.ogg",
        _ => "sounds/hit1.ogg",
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
    player_entity: Entity,
    index: usize,
    anim_graph: &PlayerAnimationGraph,
    anim_player: &mut AnimationPlayer,
    anim_state: &mut PlayerAnimState,
    combo: &mut PlayerCombo,
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    anim_player.stop_all();

    anim_player.play(
        combo_anim_node(
            anim_graph,
            index,
        ),
    );

    *anim_state = combo_anim_state(index);
    combo.current_index = Some(index);
    combo.queued_next = false;
    combo.timer = Timer::from_seconds(
        combo_duration(index),
        TimerMode::Once,
    );

    commands.spawn((
        AudioPlayer::new(
            asset_server.load(
                combo_hit_sound(index),
            ),
        ),
        PlaybackSettings::DESPAWN,
    ));

    match index {
        // SlapR
        0 => {
            queue_player_punch_hitbox(
                commands,
                player_entity,
                PlayerHand::Right,
                0.18,
                0.12,
            );
        }

        // SlapL
        1 => {
            queue_player_punch_hitbox(
                commands,
                player_entity,
                PlayerHand::Left,
                0.18,
                0.12,
            );
        }

        // SlapLR
        2 => {
            queue_player_punch_hitbox(
                commands,
                player_entity,
                PlayerHand::Right,
                0.14,
                0.12,
            );

            queue_player_punch_hitbox(
                commands,
                player_entity,
                PlayerHand::Left,
                0.36,
                0.12,
            );
        }

        _ => {}
    }
}

fn queue_player_punch_hitbox(
    commands: &mut Commands,
    owner: Entity,
    hand: PlayerHand,
    delay: f32,
    lifetime: f32,
) {
    commands.spawn(PunchHitboxRequest {
        owner,
        hand,
        delay: Timer::from_seconds(
            delay,
            TimerMode::Once,
        ),
        lifetime,
    });
}

fn spawn_requested_player_punch_hitboxes(
    mut commands: Commands,
    time: Res<Time>,

    mut request_query: Query<
        (Entity, &mut PunchHitboxRequest),
    >,

    hand_query: Query<
        (Entity, &PlayerHandBone),
    >,
) {
    for (request_entity, mut request) in &mut request_query {
        request.delay.tick(time.delta());

        if !request.delay.is_finished() {
            continue;
        }

        let Some((hand_entity, _)) = hand_query
            .iter()
            .find(|(_, hand)| hand.0 == request.hand)
        else {
            warn!("Player hand bone not found");

            commands
                .entity(request_entity)
                .despawn();

            continue;
        };

        commands
            .entity(hand_entity)
            .with_children(|parent| {
                parent.spawn((
                    Name::new("Player Punch Hitbox"),

                    PlayerPunchHitbox {
                        owner: request.owner,
                        already_hit: Vec::new(),
                    },

                    PlayerPunchHitboxLifetime(
                        Timer::from_seconds(
                            request.lifetime,
                            TimerMode::Once,
                        ),
                    ),

                    // ขนาดหมัด
                    Collider::sphere(0.22),

                    // ตรวจชนอย่างเดียว ไม่ผลักศัตรู
                    Sensor,

                    // อ่านรายชื่อ Entity ที่กำลังชน
                    CollidingEntities::default(),

                    // Offset จากจุด origin ของ Bone
                    Transform::from_xyz(
                        0.0,
                        0.08,
                        0.0,
                    ),
                ));
            });

        commands
            .entity(request_entity)
            .despawn();
    }
}

fn update_player_punch_hitbox_lifetime(
    mut commands: Commands,
    time: Res<Time>,

    mut hitbox_query: Query<
        (
            Entity,
            &mut PlayerPunchHitboxLifetime,
        ),
    >,
) {
    for (entity, mut lifetime) in &mut hitbox_query {
        lifetime.0.tick(time.delta());

        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_floating_damage_text(
    commands: &mut Commands,
    damage: i32,
    position: Vec3,
    kind: FloatingDamageKind,
) {
    let (text, font_size, color, velocity, lifetime) =
        match kind {
            FloatingDamageKind::EnemyNormal => (
                format!("-{}", damage),
                32.0,
                Color::srgb(1.0, 0.35, 0.1),
                Vec3::new(0.0, 1.5, 0.0),
                0.8,
            ),

            FloatingDamageKind::EnemyCritical => (
                format!("CRIT -{}", damage),
                52.0,
                Color::srgb(1.0, 0.85, 0.1),
                Vec3::new(0.0, 2.0, 0.0),
                1.1,
            ),

            FloatingDamageKind::PlayerHit => (
                format!("HP -{}", damage),
                40.0,
                Color::srgb(1.0, 0.05, 0.05),
                Vec3::new(-0.5, 1.7, 0.0),
                1.0,
            ),

            FloatingDamageKind::PlayerDrain => (
                format!("DRAIN -{}", damage),
                40.0,
                Color::srgb(0.85, 0.2, 1.0),
                Vec3::new(0.5, 1.7, 0.0),
                1.0,
            ),
        };

    commands.spawn((
        FloatingDamageText {
            timer: Timer::from_seconds(
                lifetime,
                TimerMode::Once,
            ),
            world_position: position,
            velocity,
        },
        Text::new(text),
        TextFont {font_size, ..default()},
        TextColor(color),
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
    let Ok((camera, camera_transform)) = camera_query.single() else {return};

    for (entity, mut node, mut floating_text) in &mut text_query {
        floating_text.timer.tick(time.delta());
        let velocity = floating_text.velocity;
        let delta_seconds = time.delta_secs();
        floating_text.world_position += velocity * delta_seconds;
        if floating_text.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let Ok(screen_pos) = camera.world_to_viewport(camera_transform,floating_text.world_position,
        ) else {continue};
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

pub fn rebuild_player_combat_stats_from_exp(
    mut player_query: Query<(&BaseStats,&ElementMastery,&mut CombatStats,&mut Health,&mut Mana),(With<Player>,Changed<ElementMastery>,)>,
) {
    for (base,mastery,mut combat,mut health,mut mana) in &mut player_query{
        let new_stats = combat_stats_from_element_exp(base, mastery);
        *combat = new_stats;
        let new_hp_max = new_stats.max_hp.round() as i32;
        let new_mp_max = new_stats.max_mp.round() as i32;
        health.max = new_hp_max;
        health.current = health.current.clamp(0, new_hp_max);
        mana.max = new_mp_max;
        mana.current = mana.current.clamp(0, new_mp_max);
    }
}

pub fn respawn_player_when_defeated(
    mut player_query: Query<(&mut Health,&mut Transform,&mut LinearVelocity),With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok((mut health,mut transform,mut velocity)) = player_query.single_mut() else { return };

    if health.current > 0 {
        return;
    }

    transform.translation = Vec3::new(0.0, 2.0, 0.0);
    velocity.x = 0.0;
    velocity.y = 0.0;
    velocity.z = 0.0;
    health.current = health.max;
    commands.spawn(AudioPlayer::new(asset_server.load("sounds/sfx_game_over.ogg")));
}

const PLAYER_LEFT_HAND_BONE: &str = "mixamorig_LeftHand";
const PLAYER_RIGHT_HAND_BONE: &str = "mixamorig_RightHand";
fn tag_player_hand_bones(
    mut commands: Commands,

    bone_query: Query<
        (Entity, &Name),
        Added<Name>,
    >,

    child_of_query: Query<&ChildOf>,
    player_query: Query<(), With<Player>>,
) {
    for (entity, name) in &bone_query {
        if !belongs_to_player(
            entity,
            &child_of_query,
            &player_query,
        ) {
            continue;
        }

        let hand = match name.as_str() {
            PLAYER_LEFT_HAND_BONE => PlayerHand::Left,
            PLAYER_RIGHT_HAND_BONE => PlayerHand::Right,
            _ => continue,
        };

        commands
            .entity(entity)
            .insert(PlayerHandBone(hand));

        info!("Found player hand bone: {}", name.as_str());
    }
}
fn belongs_to_player(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    player_query: &Query<(), With<Player>>,
) -> bool {
    let mut current = entity;

    loop {
        if player_query.get(current).is_ok() {
            return true;
        }

        let Ok(child_of) = child_of_query.get(current) else {
            return false;
        };

        current = child_of.parent();
    }
}
fn spawn_player_slap_hitbox(
    mut commands: Commands,

    anim_query: Query<
        &PlayerAnimState,
        (
            With<PlayerAnimationTarget>,
            Changed<PlayerAnimState>,
        ),
    >,

    bone_query: Query<(Entity, &Name)>,

    child_of_query: Query<&ChildOf>,
    player_query: Query<(), With<Player>>,
) {
    let Ok(anim_state) = anim_query.single()
    else {
        return;
    };

    let is_slap = matches!(
        *anim_state,
        PlayerAnimState::SlapR
            | PlayerAnimState::SlapL
            | PlayerAnimState::SlapLR
    );

    if !is_slap {
        return;
    }

    let mut spawned_count = 0;

    for (bone_entity, bone_name) in &bone_query {
        if !belongs_to_player(
            bone_entity,
            &child_of_query,
            &player_query,
        ) {
            continue;
        }

        let correct_hand = match *anim_state {
            PlayerAnimState::SlapR => {
                bone_name.as_str()
                    == PLAYER_RIGHT_HAND_BONE
            }

            PlayerAnimState::SlapL => {
                bone_name.as_str()
                    == PLAYER_LEFT_HAND_BONE
            }

            // SlapLR ให้ Collider ออกทั้งสองมือ
            PlayerAnimState::SlapLR => {
                bone_name.as_str()
                    == PLAYER_RIGHT_HAND_BONE
                    || bone_name.as_str()
                        == PLAYER_LEFT_HAND_BONE
            }

            _ => false,
        };

        if !correct_hand {
            continue;
        }

        commands
            .entity(bone_entity)
            .with_children(|parent| {
                parent.spawn((
                    Name::new("Player Slap Hitbox"),

                    PlayerSlapHitbox {
                        lifetime: Timer::from_seconds(
                            0.30,
                            TimerMode::Once,
                        ),
                        has_hit: false,
                    },

                    // ขนาด Collider ที่มือ
                    Collider::sphere(0.22),

                    // ตรวจชนแต่ไม่ผลัก Enemy
                    Sensor,

                    // จำเป็นสำหรับ CollisionStart
                    CollisionEventsEnabled,

                    // Collider อยู่ตรง origin ของกระดูกมือ
                    Transform::IDENTITY,
                ));
            });

        spawned_count += 1;
    }

    if spawned_count == 0 {
        warn!(
            "Slap hitbox not spawned: hand bone not found"
        );
    }
}

fn player_slap_hit_enemy(
    mut commands: Commands,
    time: Res<Time>,

    mut meshes:
        ResMut<Assets<Mesh>>,

    mut materials:
        ResMut<Assets<StandardMaterial>>,

    mut collision_reader:
        MessageReader<CollisionStart>,

    mut hitbox_query:
        Query<&mut PlayerSlapHitbox>,

    mut player_query: Query<
        (
            &CombatStats,
            &AtkAndDefElement,
            &mut ElementMastery,
        ),
        (
            With<Player>,
            Without<CombatTarget>,
        ),
    >,

    mut target_query: Query<
        (
            &mut Health,
            &CombatStats,
            &AtkAndDefElement,
            &GlobalTransform,
            Option<&ElementExpReward>,

            // Practice ไม่มี EnemyState
            // Muamua มี EnemyState
            Option<&mut EnemyState>,
        ),
        (
            With<CombatTarget>,
            Without<Player>,
        ),
    >,
) {
    let Ok((
        player_stats,
        player_element,
        mut element_mastery,
    )) = player_query.single_mut()
    else {
        return;
    };

    let mut rng = rand::rng();

    for event in collision_reader.read() {
        // เช็กว่าฝั่งไหนคือ Hitbox ของ Player
        let (
            hitbox_entity,
            target_entity,
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

        // Hitbox นี้เคยทำดาเมจแล้ว
        if hitbox.has_hit {
            continue;
        }

        let Ok((
            mut target_health,
            target_stats,
            target_element,
            target_transform,
            reward,
            mut enemy_state,
        )) = target_query.get_mut(
            target_entity,
        )
        else {
            continue;
        };

        // เป้าหมายตายไปแล้ว
        // ห้ามแจก EXP หรือทำดาเมจซ้ำ
        if target_health.current <= 0 {
            commands
                .entity(hitbox_entity)
                .despawn();

            continue;
        }

        hitbox.has_hit = true;

        let (
            base_damage,
            is_critical,
        ) = calculate_combat_damage(
            player_stats,
            target_stats,
            &mut rng,
        );

        let element_multiplier =
            elemental_multiplier(
                player_element.0,
                target_element.0,
            );

        let damage = (
            base_damage as f32
                * element_multiplier
        )
            .round()
            .max(1.0) as i32;

        target_health.current -= damage;

        target_health.current =
            target_health.current.clamp(
                0,
                target_health.max,
            );

        let damage_kind =
            if is_critical {
                FloatingDamageKind::EnemyCritical
            } else {
                FloatingDamageKind::EnemyNormal
            };

        let target_position =
            target_transform.translation();

        spawn_floating_damage_text(
            &mut commands,
            damage,
            target_position
                + Vec3::new(0.0, 2.0, 0.0),
            damage_kind,
        );

        info!(
            "Slap hit: damage={}, HP={}/{}",
            damage,
            target_health.current,
            target_health.max,
        );

        // ==============================
        // ยังไม่ตาย → เล่น Hurt
        // ==============================
        if target_health.current > 0 {
            if let Some(state) =
                enemy_state.as_mut()
            {
                **state =
                    EnemyState::Hurt;

                commands
                    .entity(target_entity)
                    .insert(
                        MuamuaStateTimer(
                            Timer::from_seconds(
                                0.45,
                                TimerMode::Once,
                            ),
                        ),
                    );
            }

            commands
                .entity(hitbox_entity)
                .despawn();

            continue;
        }

        // ==============================
        // HP เหลือ 0 → แจก EXP
        // ==============================
        if let Some(reward) = reward {
            let gain = reward.grant_all(
                &mut *element_mastery,
                &mut rng,
            );

            info!(
                "EXP gain: Water +{}, Fire +{}, Wind +{}, Earth +{}, Inw +{}",
                gain.water,
                gain.fire,
                gain.wind,
                gain.earth,
                gain.inw,
            );
        }

        // ==============================
        // มี EnemyState = Muamua
        // เล่น Dead ก่อน ยังไม่ despawn
        // ==============================
        if let Some(state) =
            enemy_state.as_mut()
        {
            **state =
                EnemyState::Dead;

            commands
                .entity(target_entity)
                .insert(
                    MuamuaStateTimer(
                        Timer::from_seconds(
                            1.20,
                            TimerMode::Once,
                        ),
                    ),
                );

            // ป้องกัน Player ตบซ้ำระหว่างเล่น Dead
            commands
                .entity(target_entity)
                .remove::<CombatTarget>();

            spawn_defeat_particles(
                &mut commands,
                &mut meshes,
                &mut materials,
                target_position,
                time.elapsed_secs(),
            );

            commands
                .entity(hitbox_entity)
                .despawn();

            continue;
        }

        // ==============================
        // ไม่มี EnemyState
        // เช่น Basic / Advanced Practice
        // ตายแล้วหายทันทีเหมือนเดิม
        // ==============================
        spawn_defeat_particles(
            &mut commands,
            &mut meshes,
            &mut materials,
            target_position,
            time.elapsed_secs(),
        );

        commands
            .entity(target_entity)
            .despawn();

        commands
            .entity(hitbox_entity)
            .despawn();
    }
}

fn despawn_player_slap_hitbox(
    mut commands: Commands,
    time: Res<Time>,

    mut hitbox_query: Query<
        (
            Entity,
            &mut PlayerSlapHitbox,
        ),
    >,
) {
    for (entity, mut hitbox) in &mut hitbox_query {
        hitbox.lifetime.tick(time.delta());

        if hitbox.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
