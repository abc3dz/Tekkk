use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;
use bevy_wind_waker_shader::prelude::*;
use bevy::animation::graph::AnimationGraph;
use bevy::animation::AnimationPlayer;

use crate::components::*;

pub fn spawn_guardian_npc(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    commands
    .spawn((
        Npc,
        GuardianNpc,
        Transform {
            translation: Vec3::new(-8.0, 1.25, -6.0),
            //rotation: Quat::from_rotation_y(std::f32::consts::PI_2),
            ..default()
        },
        RigidBody::Static,
        Collider::capsule(0.45, 1.6),
    ))
    .with_children(|parent| {
        parent.spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("npc/Guardian.glb")
                )
            ),
            Transform::from_xyz(0.0, -1.25, 0.0),
            WindWakerShaderBuilder::default()
            .time_of_day(TimeOfDay::Day)
            .weather(Weather::Sunny)
            .build(),
        ));
    });
}
pub fn setup_guardian_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(0).from_asset("npc/Guardian.glb")
        ),
        1.0,
        graph.root,
    );

    let graph_handle = graphs.add(graph);

    commands.insert_resource(GuardianAnimationGraph {
        graph: graph_handle,
        idle,
    });
}
pub fn setup_guardian_animation_player(
    mut commands: Commands,
    anim_graph: Res<GuardianAnimationGraph>,
    mut query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut query {
        commands.entity(entity).insert((
            AnimationGraphHandle(anim_graph.graph.clone()),
            GuardianAnimationTarget,
        ));

        player.play(anim_graph.idle).repeat();
    }
}