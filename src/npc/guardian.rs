use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;
use bevy_wind_waker_shader::prelude::*;
use bevy::animation::graph::AnimationGraph;
use bevy::animation::AnimationPlayer;
use crate::components::*;
use crate::npc::{
    advanced_practice::AdvancedPracticePlugin,
    basic_practice::BasicPracticePlugin,
    practice_common::PracticeCommonPlugin,
};

pub struct GuardianPlugin;

impl Plugin for GuardianPlugin {
    fn build(&self, app: &mut App) {app
        .add_plugins((
            BasicPracticePlugin,
            AdvancedPracticePlugin,
            PracticeCommonPlugin,
        ))
        .init_resource::<BasicPracticeActive>()
        .init_resource::<AdvancedPracticeActive>()
        .insert_resource(BasicGunRespawnTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .insert_resource(AdvancedMinionRespawnTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .add_systems(Startup, setup_guardian_animation_graph)
        .add_systems(OnEnter(GameScene::Hub), spawn_guardian_npc)
        .add_systems(Update,setup_guardian_animation_player.run_if(in_state(GameScene::Hub)))
        .add_systems(Update,(
            check_guardian_interaction_area,
            check_guardian_interaction_area_exit,
            show_guardian_dialog,
            cleanup_guardian_ui_when_player_leave,
        ).run_if(in_state(GameScene::Hub)))
        .add_systems(Update, guardian_dialog_exit_input.run_if(in_state(GameScene::Hub)))
        .add_systems(OnExit(GameScene::Hub), despawn_hub_only_entities);
    }
}

pub fn spawn_guardian_npc(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
        parent.spawn((
            GuardianInteractArea,
            Sensor,
            CollisionEventsEnabled,
            Collider::cuboid(1.4, 2.0, 1.4),
            Transform::from_xyz(0.0, 0.0, 1.5),
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

    let graph_handle = graphs.add(graph);

    commands.insert_resource(GuardianAnimationGraph {
        graph: graph_handle,
        idle,
        welcome,
    });
}

pub fn setup_guardian_animation_player(
    mut commands: Commands,
    anim_graph: Res<GuardianAnimationGraph>,
    mut query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parent_query: Query<&ChildOf>,
    guardian_query: Query<(), With<GuardianNpc>>,
) {
    for (entity, mut player) in &mut query {
        if !is_child_of_guardian(entity, &parent_query, &guardian_query) {
            continue;
        }

        println!("Guardian AnimationPlayer found");

        commands.entity(entity).insert((
            AnimationGraphHandle(anim_graph.graph.clone()),
            GuardianAnimationTarget,
            GuardianAnimState::Idle,
        ));

        player.stop_all();
        player.play(anim_graph.idle).repeat();
    }
}

fn is_child_of_guardian(
    mut entity: Entity,
    parent_query: &Query<&ChildOf>,
    guardian_query: &Query<(), With<GuardianNpc>>,
) -> bool {
    loop {
        if guardian_query.get(entity).is_ok() {
            return true;
        }

        let Ok(parent) = parent_query.get(entity) else {
            return false;
        };

        entity = parent.0;
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
        guardian_area_query.get(collider1).is_ok() || guardian_area_query.get(collider2).is_ok();

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
                anim_player.stop_all();
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
            guardian_area_query.get(collider1).is_ok() || guardian_area_query.get(collider2).is_ok();

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
            commands.entity(player_entity).remove::<PlayerInGuardianArea>();

            for mut anim_player in &mut guardian_anim_query {
                anim_player.stop_all();
                anim_player.play(anim_graph.idle).repeat();
            }
        }
    }
}

pub fn show_guardian_dialog(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(), With<PlayerInGuardianArea>>,
    dialog_query: Query<Entity, With<GuardianDialogUI>>,
) {
    if player_query.is_empty() {
        return;
    }

    if !dialog_query.is_empty() {
        return;
    }

    commands
        .spawn((
            GuardianDialogUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
            // อันนี้คือ blur ปลอม / dim background
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.60)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Px(220.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(24.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    ImageNode::new(asset_server.load("npc/GuardianWelcome.png")),
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(150.0),
                        ..default()
                    },
                ));
                parent.spawn((
                    Text::new(
                        "Guardian: What kind of practice do you want?\n1 / A: Basic Practice\n2 / X: Advanced Practice\n3 / Y: Full HP / Mana\nEsc / B: Stop Practice"),
                    TextFont {font_size: 26.0, ..default()},
                    TextColor(Color::WHITE),
                ));
            });
        });
}

pub fn guardian_dialog_exit_input(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    dialog_query: Query<Entity, With<GuardianDialogUI>,>,
    practice_query: Query<Entity, With<PracticeEntity>,>,
    mut player_query: Query<(&mut Health, &mut Mana, &mut Transform,), With<Player>,>,
    mut basic_practice_active: ResMut<BasicPracticeActive>,
    mut advanced_practice_active: ResMut<AdvancedPracticeActive>,
) {
    if dialog_query.is_empty() {
        return;
    }

    let Ok((
        mut health,
        mut mana,
        mut transform,
    )) = player_query.single_mut()
    else {
        return;
    };

    let gamepad_full_pressed =
        gamepads.iter().any(|gamepad| {
            gamepad.just_pressed(
                GamepadButton::North,
            )
        });

    let full_pressed =
        keyboard.just_pressed(KeyCode::Digit3)
            || gamepad_full_pressed;

    if full_pressed {
        health.current = health.max;
        mana.current = mana.max;

        commands.spawn(AudioPlayer::new(asset_server.load("sounds/npc/fullhpmp.ogg")));
        transform.translation.z += 3.5;
    }

    let gamepad_stop_pressed =
        gamepads.iter().any(|gamepad| {
            gamepad.just_pressed(
                GamepadButton::East,
            )
        });

    let stop_pressed =keyboard.just_pressed(KeyCode::Escape)|| gamepad_stop_pressed;
    if !stop_pressed {
        return;
    }

    for entity in &practice_query {
        commands.entity(entity).despawn();
    }

    commands.spawn(AudioPlayer::new(asset_server.load("sounds/npc/exit_pt.ogg")));
    transform.translation.z += 3.5;

    basic_practice_active.0 = false;
    advanced_practice_active.0 = false;
}

pub fn cleanup_guardian_ui_when_player_leave(
    mut commands: Commands,
    player_query: Query<(), With<PlayerInGuardianArea>>,
    dialog_query: Query<Entity, With<GuardianDialogUI>>,
) {
    if !player_query.is_empty() {
        return;
    }

    for entity in &dialog_query {
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