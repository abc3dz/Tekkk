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
        HubOnly,
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
        // Trigger area in front of Guardian
        parent.spawn((
            GuardianInteractArea,
            Sensor,
            CollisionEventsEnabled,
            Collider::cuboid(2.0, 2.0, 2.0),
            Transform::from_xyz(0.0, 0.0, 2.0),
        ));
    });
}
pub fn setup_guardian_npc(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    spawn_guardian_npc(&mut commands, &asset_server);
}
pub fn setup_guardian_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(2).from_asset("npc/Guardian.glb")
        ),
        1.0,
        graph.root,
    );
    let welcome = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(3).from_asset("npc/Guardian.glb")
        ),
        1.0,
        graph.root,
    );
    let basic_practice = graph.add_clip(
        asset_server.load(
            GltfAssetLabel::Animation(1).from_asset("npc/Guardian.glb")
        ),
        1.0,
        graph.root,
    );
    let advanced_practice = graph.add_clip(
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
        welcome,
        basic_practice,
        advanced_practice,
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

pub fn check_guardian_interaction_area(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    guardian_area_query: Query<Entity, With<GuardianInteractArea>>,
    player_query: Query<Entity, With<Player>>,
    anim_graph: Res<GuardianAnimationGraph>,
    mut guardian_anim_query: Query<&mut AnimationPlayer, With<GuardianAnimationTarget>>,
) {
    for event in collision_events.read() {
        let collider1 = event.collider1;
        let collider2 = event.collider2;

        // ถ้า collider นี้ผูกกับ RigidBody parent ให้ใช้ body แทน
        let body1 = event.body1.unwrap_or(collider1);
        let body2 = event.body2.unwrap_or(collider2);

        let hit_guardian_area =
            guardian_area_query.get(collider1).is_ok()
            || guardian_area_query.get(collider2).is_ok();

        if !hit_guardian_area {
            continue;
        }

        let player_entity =
            if player_query.get(body1).is_ok() {
                Some(body1)
            } else if player_query.get(body2).is_ok() {
                Some(body2)
            } else if player_query.get(collider1).is_ok() {
                Some(collider1)
            } else if player_query.get(collider2).is_ok() {
                Some(collider2)
            } else {
                None
            };

        if let Some(player_entity) = player_entity {
            println!("Player entered Guardian area");
            commands.entity(player_entity).insert(PlayerInGuardianArea);

            for mut anim_player in &mut guardian_anim_query {
                anim_player.play(anim_graph.welcome);
            }
        }
    }
}

pub fn check_guardian_interaction_area_exit(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEnd>,
    guardian_area_query: Query<Entity, With<GuardianInteractArea>>,
    player_query: Query<Entity, With<Player>>,
    anim_graph: Res<GuardianAnimationGraph>,
    mut guardian_anim_query: Query<&mut AnimationPlayer, With<GuardianAnimationTarget>>,
) {
    for event in collision_events.read() {
        let collider1 = event.collider1;
        let collider2 = event.collider2;

        let body1 = event.body1.unwrap_or(collider1);
        let body2 = event.body2.unwrap_or(collider2);

        let hit_guardian_area =
            guardian_area_query.get(collider1).is_ok()
            || guardian_area_query.get(collider2).is_ok();

        if !hit_guardian_area {
            continue;
        }

        let player_entity =
            if player_query.get(body1).is_ok() {
                Some(body1)
            } else if player_query.get(body2).is_ok() {
                Some(body2)
            } else if player_query.get(collider1).is_ok() {
                Some(collider1)
            } else if player_query.get(collider2).is_ok() {
                Some(collider2)
            } else {
                None
            };

        if let Some(player_entity) = player_entity {
            println!("Player left Guardian area");
            commands.entity(player_entity).remove::<PlayerInGuardianArea>();

            for mut anim_player in &mut guardian_anim_query {
                anim_player.play(anim_graph.idle).repeat();
            }

        }
    }
}

pub fn guardian_interact_input(
    mut commands: Commands,
    player_query: Query<(), With<PlayerInGuardianArea>>,
    menu_query: Query<Entity, With<GuardianMenuUI>>,
) {

    if player_query.is_empty() {
        return;
    }

    if !menu_query.is_empty() {
        return;
    }

    println!("Interact with Guardian");

    commands
        .spawn((
            GuardianMenuUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                left: Val::Percent(35.0),
                width: Val::Percent(30.0),
                height: Val::Percent(35.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Guardian Practice"),
                TextFont {
                    font_size: 34.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("1. Basic Practice"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("2. Advance Practice"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("3. Exit"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
pub fn guardian_menu_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    menu_query: Query<Entity, With<GuardianMenuUI>>,
    anim_graph: Res<GuardianAnimationGraph>,
    mut guardian_anim_query: Query<&mut AnimationPlayer, With<GuardianAnimationTarget>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if menu_query.is_empty() {
        return;
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        println!("Basic Practice selected");

        // ตรงนี้ค่อยเปลี่ยน State หรือเปิดระบบฝึก basic
        // commands.insert_resource(...);
    }

    if keyboard.just_pressed(KeyCode::Digit2) {
        println!("Advance Practice selected");

        // ตรงนี้ค่อยเปลี่ยน State หรือเปิดระบบฝึก advance
    }

    if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Escape) {
        println!("Exit Guardian menu");

        for mut anim_player in &mut guardian_anim_query {
                anim_player.play(anim_graph.idle).repeat();

            for entity in &menu_query {
                commands.entity(entity).despawn();
            }

            for mut transform in &mut player_query {
                transform.translation.z += 2.0;
            }
        }
    }
}
pub fn cleanup_guardian_ui_when_player_leave(
    mut commands: Commands,
    player_query: Query<(), With<PlayerInGuardianArea>>,
    menu_query: Query<Entity, With<GuardianMenuUI>>,
) {
    if !player_query.is_empty() {
        return;
    }

    for entity in &menu_query {
        commands.entity(entity).despawn();
    }
}
pub fn despawn_hub_only_entities(
    mut commands: Commands,
    hub_query: Query<Entity, With<HubOnly>>,
) {
    for entity in &hub_query {
        commands.entity(entity).despawn();
    }
}