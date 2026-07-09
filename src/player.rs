use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::animation::graph::AnimationGraph;
use bevy::animation::AnimationPlayer;
use avian3d::prelude::*;
use bevy_wind_waker_shader::prelude::*;
use crate::components::*;

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
                player_combo_input,
                player_combo_update,
                update_player_animation,
                update_player_status_ui,
            ).chain()
        );
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
    .spawn((
        Player,
        MoveSpeed(6.0),
        Health { current: 80, max: 100 },
        Mana { current: 70, max: 100 },
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
                    GltfAssetLabel::Scene(0).from_asset("models/PlayerMoya.glb")
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
        asset_server.load(GltfAssetLabel::Animation(1).from_asset("models/PlayerMoya.glb")),
        1.0,
        graph.root,
    );

    let walk = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(4).from_asset("models/PlayerMoya.glb")),
        1.0,
        graph.root,
    );
    let punch_r = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(3).from_asset("models/PlayerMoya.glb")),
        1.0,
        graph.root,
    );
    let punch_l = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(2).from_asset("models/PlayerMoya.glb")),
        1.0,
        graph.root,
    );

    let graph_handle = graphs.add(graph);

    commands.insert_resource(PlayerAnimationGraph {
        graph: graph_handle,
        idle,
        walk,
        punch_r,
        punch_l,
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
            PlayerAnimState::PunchR | PlayerAnimState::PunchL
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

const PLAYER_COMBO_COUNT: usize = 2;

fn combo_duration(index: usize) -> f32 {
    match index {
        0 => 0.45, // punch_r
        1 => 0.45, // punch_l
        _ => 0.45,
    }
}

fn combo_anim_node(
    anim_graph: &PlayerAnimationGraph,
    index: usize,
) -> AnimationNodeIndex {
    match index {
        0 => anim_graph.punch_r,
        1 => anim_graph.punch_l,
        _ => anim_graph.punch_r,
    }
}

fn combo_anim_state(index: usize) -> PlayerAnimState {
    match index {
        0 => PlayerAnimState::PunchR,
        1 => PlayerAnimState::PunchL,
        _ => PlayerAnimState::PunchR,
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

    println!("Combo attack {}", index);
}