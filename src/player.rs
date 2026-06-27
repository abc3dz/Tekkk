use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::animation::graph::AnimationGraph;
use bevy::animation::AnimationPlayer;
use avian3d::prelude::*;
use crate::components::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_player_animation_graph, spawn_player))
            .add_systems(Update, (
                setup_player_animation_player,
                player_movement,
                update_player_animation,
            ).chain());
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
        Health { current: 100, max: 100 },

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
            Transform::from_xyz(0.0, -0.8, 0.0),
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

    // ✅ insert_resource ต้องมี Resource derive ใน components.rs
    commands.insert_resource(PlayerAnimationGraph {
        graph: graph_handle,
        idle,
        walk,
    });
}

fn setup_player_animation_player(
    mut commands: Commands,
    anim_graph: Res<PlayerAnimationGraph>,
    mut query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut query {
        commands.entity(entity).insert((
            AnimationGraphHandle(anim_graph.graph.clone()),
            // ✅ ใช้ PlayerAnimState (ไม่ใช่ PlayerAnimationState)
            PlayerAnimState::Idle,
        ));

        player.play(anim_graph.idle).repeat();
    }
}

fn update_player_animation(
    anim_graph: Res<PlayerAnimationGraph>,
    player_query: Query<&PlayerActionState, With<Player>>,
    mut anim_query: Query<(&mut AnimationPlayer, &mut PlayerAnimState)>,
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