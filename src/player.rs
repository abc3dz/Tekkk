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
        PlayerActionState::Idle,
        MoveSpeed(6.0),
        Health { current: 80, max: 100 },
        Mana { current: 70, max: 100 },
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
        &mut PlayerActionState,
    ), With<Player>>,
) {
    let Ok((speed, mut velocity, mut transform, mut action_state)) = query.single_mut() else {
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
        *action_state = PlayerActionState::Walk;
    } else {
        velocity.x = 0.0;
        velocity.z = 0.0;
        *action_state = PlayerActionState::Idle;
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
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/PlayerMoya.glb")),
        1.0,
        graph.root,
    );

    let walk = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(1).from_asset("models/PlayerMoya.glb")),
        1.0,
        graph.root,
    );

    let graph_handle = graphs.add(graph);

    commands.insert_resource(PlayerAnimationGraph {
        graph: graph_handle,
        idle,
        walk,
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
    player_query: Query<&PlayerActionState, With<Player>>,
    mut anim_query: Query<
        (&mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
) {
    let Ok(action_state) = player_query.single() else {
        return;
    };

    for (mut player, mut anim_state) in &mut anim_query {
        match *action_state {
            PlayerActionState::Idle => {
                if *anim_state != PlayerAnimState::Idle {
                    player.stop(anim_graph.walk);
                    player.play(anim_graph.idle).repeat();
                    *anim_state = PlayerAnimState::Idle;
                }
            }

            PlayerActionState::Walk => {
                if *anim_state != PlayerAnimState::Walk {
                    player.stop(anim_graph.idle);
                    player.play(anim_graph.walk).repeat();
                    *anim_state = PlayerAnimState::Walk;
                }
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