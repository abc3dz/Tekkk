use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;
use bevy_wind_waker_shader::prelude::*;
use bevy::animation::graph::AnimationGraph;
use bevy::animation::AnimationPlayer;
use rand::Rng;
use crate::components::*;
use crate::player::{
    FloatingDamageKind,
    play_player_hurt_animation,
    spawn_floating_damage_text,
};
use crate::combat::*;

pub struct GuardianPlugin;

impl Plugin for GuardianPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<BasicPracticeActive>()
        .init_resource::<AdvancedPracticeActive>()
        .insert_resource(BasicGunRespawnTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .insert_resource(AdvancedMinionRespawnTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .add_systems(Startup, setup_guardian_animation_graph)
            .add_systems(OnEnter(GameScene::Hub), setup_guardian_npc)
            .add_systems(Update,setup_guardian_animation_player.run_if(in_state(GameScene::Hub))
            )
            .add_systems(
                Update,
                (
                    check_guardian_interaction_area,
                    check_guardian_interaction_area_exit,
                    show_guardian_dialog,
                    cleanup_guardian_ui_when_player_leave,
                )
                .run_if(in_state(GameScene::Hub))
            )

            .add_systems(Update, guardian_dialog_exit_input.run_if(in_state(GameScene::Hub)))
            .add_systems(Update, guardian_dialog_basic_input.run_if(in_state(GameScene::Hub)))
            .add_systems(Update, guardian_dialog_advanced_input.run_if(in_state(GameScene::Hub)))

            .add_systems(
                Update,
                (
                    rotate_basic_practice_gun_to_player,
                    basic_practice_gun_shoot_projectile,
                    move_basic_practice_projectiles,
                    basic_projectile_hit_player,
                    update_basic_gun_health_bar,
                )
                .run_if(in_state(GameScene::Hub))
            )
        .add_systems(
                Update,
                (
                    minion_chase_player,
                    minion_drain_player_life,
                    update_minion_health_bar,
                )
                .run_if(in_state(GameScene::Hub))
            )
        .add_systems(Update, (
            respawn_basic_gun_when_defeated,
            respawn_advanced_minion_when_defeated.run_if(in_state(GameScene::Hub)),
        ))
        .add_systems(OnExit(GameScene::Hub), despawn_hub_only_entities);
    }
}

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
            Transform::from_xyz(0.0, 0.0, 1.5),
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
                        "Guardian:\nWhat kind of practice do you want?\n\n1. Basic Practice\n2. Advanced Practice\n3. Full HP / Mana\nEsc. Stop Practice"
                    ),
                    TextFont {
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}
pub fn guardian_dialog_exit_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog_query: Query<Entity, With<GuardianDialogUI>>,
    practice_query: Query<Entity, With<PracticeEntity>>,
    mut player_query: Query<(&mut Health, &mut Mana, &mut Transform), With<Player>>,
    mut basic_practice_active: ResMut<BasicPracticeActive>,
    mut advanced_practice_active: ResMut<AdvancedPracticeActive>,
) {
    if dialog_query.is_empty() {
        return;
    }

    let Ok((mut health, mut mana, mut transform)) = player_query.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Digit3) {
        health.current = health.max;
        mana.current = mana.max;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        for entity in &practice_query {
            commands.entity(entity).despawn();
        }
        transform.translation.z += 3.5;
        basic_practice_active.0 = false;
        advanced_practice_active.0 = false;
    }
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

// Basic practice
fn spawn_basic_practice_gun(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let mut rng = rand::rng();

    let x = rng.random_range(-4.0..=10.0);
    let y = 0.0;
    let z = rng.random_range(-4.0..=10.0);

    let base_stats = BaseStats::BASIC_PRACTICE_GUN;

    commands
        .spawn((
            HubOnly,
            PracticeEntity,
            BasicPracticeGun,
            CombatTarget,

            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },
             // ระบบสถานะใหม่
            base_stats,
            CombatStats::from(base_stats),

            // หุ่นฝึกเป็น Neutral
            AtkAndDefElement(Element::Neutral),

            // EXP ที่ผู้เล่นได้รับเมื่อกำจัด
            ElementExpReward::BASIC_PRACTICE_GUN,

            BasicGunShootTimer(
                Timer::from_seconds(1.0, TimerMode::Repeating)
            ),

            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0)
                        .from_asset("npc/BasicPracticeGun.glb")
                )
            ),

            Transform::from_xyz(x, y, z),
            GlobalTransform::default(),

            WindWakerShaderBuilder::default()
                .time_of_day(TimeOfDay::Day)
                .weather(Weather::Sunny)
                .build(),
        ));
}
pub fn guardian_dialog_basic_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,

    mut basic_practice_active: ResMut<BasicPracticeActive>,
    mut advanced_practice_active: ResMut<AdvancedPracticeActive>,
    mut respawn_timer: ResMut<BasicGunRespawnTimer>,

    dialog_query: Query<Entity, With<GuardianDialogUI>>,
    practice_query: Query<Entity, With<PracticeEntity>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if dialog_query.is_empty() {
        return;
    }

    if !keyboard.just_pressed(KeyCode::Digit1) {
        return;
    }

    println!("Basic Practice selected");

    basic_practice_active.0 = true;
    advanced_practice_active.0 = false;
    respawn_timer.0.reset();

    for entity in &practice_query {
        commands.entity(entity).despawn();
    }

    if let Ok(mut transform) = player_query.single_mut() {
        transform.translation = Vec3::new(0.0, 0.0, 0.0);
    }

    spawn_basic_practice_gun(
        &mut commands,
        &asset_server,
    );
}
pub fn rotate_basic_practice_gun_to_player(
    player_query: Query<&Transform, (With<Player>, Without<BasicPracticeGun>)>,
    mut gun_query: Query<&mut Transform, (With<BasicPracticeGun>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    for mut gun_tf in &mut gun_query {
        let mut direction = player_tf.translation - gun_tf.translation;
        direction.y = 0.0;

        if direction.length_squared() < 0.0001 {
            continue;
        }

        let yaw = direction.x.atan2(direction.z);

        gun_tf.rotation = Quat::from_rotation_y(yaw);
    }
}
pub fn basic_practice_gun_shoot_projectile(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, (With<Player>, Without<BasicPracticeGun>)>,
    mut gun_query: Query<
        (&Transform, &mut BasicGunShootTimer),
        (With<BasicPracticeGun>, Without<Player>),
    >,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    for (gun_tf, mut shoot_timer) in &mut gun_query {
        shoot_timer.0.tick(time.delta());

        if !shoot_timer.0.just_finished() {
            continue;
        }

        let mut direction = player_tf.translation - gun_tf.translation;
        direction.y = 0.0;

        if direction.length_squared() < 0.0001 {
            continue;
        }

        let direction = direction.normalize();
        let speed = 7.0;

        let spawn_pos = gun_tf.translation + direction * 0.8 + Vec3::Y * 0.3;

        let yaw = direction.x.atan2(direction.z);

        commands.spawn((
            PracticeEntity,
            BasicPracticeProjectile {
                velocity: direction * speed,
                hp_damage: 5
            },
            ProjectileLifetime(Timer::from_seconds(4.0, TimerMode::Once)),

            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("npc/BasicPracticeProjectile.glb")
                )
            ),

            Transform {
                translation: spawn_pos,
                rotation: Quat::from_rotation_y(yaw),
                scale: Vec3::splat(1.0),
                ..default()
            },
            GlobalTransform::default(),
        ));
    }
}
pub fn move_basic_practice_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectile_query: Query<(
        Entity,
        &mut Transform,
        &BasicPracticeProjectile,
        &mut ProjectileLifetime,
    )>,
) {
    for (entity, mut transform, projectile, mut lifetime) in &mut projectile_query {
        transform.translation += projectile.velocity * time.delta_secs();

        lifetime.0.tick(time.delta());

        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
pub fn basic_projectile_hit_player(
    mut commands: Commands,

    projectile_query: Query<
        (Entity, &Transform, &BasicPracticeProjectile),
        (With<BasicPracticeProjectile>, Without<Player>),
    >,

    mut player_query: Query<
        (Entity, &Transform, &mut Health),
        (With<Player>, Without<BasicPracticeProjectile>),
    >,

    anim_graph: Res<PlayerAnimationGraph>,

    mut anim_query: Query<
        (&mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,
) {
    let Ok((player_entity, player_tf, mut health)) =
        player_query.single_mut()
    else {
        return;
    };

    for (projectile_entity, projectile_tf, projectile) in &projectile_query {
        let distance =
            player_tf.translation.distance(projectile_tf.translation);

        if distance >= 0.8 {
            continue;
        }

        health.current -= projectile.hp_damage;
        health.current = health.current.clamp(0, health.max);

        spawn_floating_damage_text(
            &mut commands,
            projectile.hp_damage,
            player_tf.translation
                + Vec3::new(0.0, 2.0, 0.0),
            FloatingDamageKind::PlayerHit,
        );

        play_player_hurt_animation(
            &mut commands,
            player_entity,
            &anim_graph,
            &mut anim_query,
        );

        commands.entity(projectile_entity).despawn();
    }
}
pub fn update_basic_gun_health_bar(
    gun_query: Query<(&Health, &Children), With<BasicPracticeGun>>,
    mut fill_query: Query<&mut Transform, With<BasicGunHealthBarFill>>,
) {
    let full_width = 1.3;

    for (health, children) in &gun_query {
        let ratio = health.current as f32 / health.max as f32;
        let ratio = ratio.clamp(0.0, 1.0);

        for child in children.iter() {
            if let Ok(mut fill_tf) = fill_query.get_mut(child) {
                fill_tf.scale.x = ratio;

                // Keep the left side fixed while shrinking
                fill_tf.translation.x = -full_width * (1.0 - ratio) * 0.5;
            }
        }
    }
}
pub fn respawn_basic_gun_when_defeated(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,

    basic_practice_active: Res<BasicPracticeActive>,
    mut respawn_timer: ResMut<BasicGunRespawnTimer>,

    basic_gun_query: Query<(), With<BasicPracticeGun>>,
) {
    // ยังไม่ได้เลือก Basic Practice
    if !basic_practice_active.0 {
        respawn_timer.0.reset();
        return;
    }

    // ปืนยังอยู่ ไม่ต้องสปอน
    if !basic_gun_query.is_empty() {
        respawn_timer.0.reset();
        return;
    }

    // ปืนถูก defeated แล้ว เริ่มนับเวลาก่อนเกิดใหม่
    respawn_timer.0.tick(time.delta());

    if !respawn_timer.0.just_finished() {
        return;
    }

    spawn_basic_practice_gun(
        &mut commands,
        &asset_server,
    );

    respawn_timer.0.reset();
}

//Advanced practice
fn spawn_advanced_minion(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let mut rng = rand::rng();

    let x = rng.random_range(-4.0..=10.0);
    let y = 0.0;
    let z = rng.random_range(-4.0..=10.0);

    let base_stats = BaseStats::ADVANCED_PRACTICE_MINION;

    commands
        .spawn((
            HubOnly,
            PracticeEntity,
            GuardianClone,
            CombatTarget,

            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },

            base_stats,
            CombatStats::from(base_stats),

            AtkAndDefElement(Element::Neutral),
            ElementExpReward::ADVANCED_PRACTICE_MINION,

            MinionLifeDrainTimer(
                Timer::from_seconds(0.5, TimerMode::Repeating)
            ),

            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0)
                        .from_asset("npc/MinionChar.glb")
                )
            ),

            Transform {
                translation: Vec3::new(x, y, z),
                scale: Vec3::splat(1.0),
                ..default()
            },

            GlobalTransform::default(),

            WindWakerShaderBuilder::default()
                .time_of_day(TimeOfDay::Day)
                .weather(Weather::Sunny)
                .build(),
        ));
}
pub fn guardian_dialog_advanced_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,

    mut basic_practice_active: ResMut<BasicPracticeActive>,
    mut advanced_practice_active: ResMut<AdvancedPracticeActive>,
    mut advanced_respawn_timer: ResMut<AdvancedMinionRespawnTimer>,

    dialog_query: Query<Entity, With<GuardianDialogUI>>,
    practice_query: Query<Entity, With<PracticeEntity>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if dialog_query.is_empty() {
        return;
    }

    if !keyboard.just_pressed(KeyCode::Digit2) {
        return;
    }

    println!("Advanced Practice selected");

    basic_practice_active.0 = false;
    advanced_practice_active.0 = true;
    advanced_respawn_timer.0.reset();

    for entity in &practice_query {
        commands.entity(entity).despawn();
    }

    spawn_advanced_minion(
        &mut commands,
        &asset_server,
    );

    if let Ok(mut player_tf) = player_query.single_mut() {
        player_tf.translation = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    }
}
pub fn minion_chase_player(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<GuardianClone>)>,
    mut clone_query: Query<&mut Transform, (With<GuardianClone>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    for mut clone_tf in &mut clone_query {
        let mut direction = player_tf.translation - clone_tf.translation;

        // Do not move up/down
        direction.y = 0.0;

        let distance = direction.length();

        // Stop near player
        if distance < 1.0 {
            continue;
        }

        let move_dir = direction.normalize();
        let speed = 2.5;

        clone_tf.translation += move_dir * speed * time.delta_secs();

        // Rotate capsule to face player
        let yaw = move_dir.x.atan2(move_dir.z);
        clone_tf.rotation = Quat::from_rotation_y(yaw);
    }
}
pub fn minion_drain_player_life(
    mut commands: Commands,
    time: Res<Time>,

    anim_graph: Res<PlayerAnimationGraph>,

    mut anim_query: Query<
        (&mut AnimationPlayer, &mut PlayerAnimState),
        With<PlayerAnimationTarget>,
    >,

    mut player_query: Query<
        (Entity, &Transform, &mut Health),
        (With<Player>, Without<GuardianClone>),
    >,

    mut minion_query: Query<
        (&Transform, &mut Health, &mut MinionLifeDrainTimer),
        (With<GuardianClone>, Without<Player>),
    >,
) {
    let Ok((player_entity, player_tf, mut player_health)) =
        player_query.single_mut()
    else {
        return;
    };

    for (minion_tf, mut minion_health, mut drain_timer) in &mut minion_query {
        drain_timer.0.tick(time.delta());

        let distance =
            player_tf.translation.distance(minion_tf.translation);

        if distance >= 1.4 || !drain_timer.0.just_finished() {
            continue;
        }

        let drain_amount = 2;

        player_health.current -= drain_amount;
        player_health.current =
            player_health.current.clamp(0, player_health.max);

        spawn_floating_damage_text(
            &mut commands,
            drain_amount,
            player_tf.translation
                + Vec3::new(0.0, 2.0, 0.0),
            FloatingDamageKind::PlayerDrain,
        );

        minion_health.current += drain_amount;
        minion_health.current =
            minion_health.current.clamp(0, minion_health.max);

        play_player_hurt_animation(
            &mut commands,
            player_entity,
            &anim_graph,
            &mut anim_query,
        );
    }
}
pub fn update_minion_health_bar(
    minion_query: Query<(&Health, &Children), With<GuardianClone>>,
    mut fill_query: Query<&mut Transform, With<MinionHealthBarFill>>,
) {
    let full_width = 1.2;

    for (health, children) in &minion_query {
        let ratio = health.current as f32 / health.max as f32;
        let ratio = ratio.clamp(0.0, 1.0);

        for child in children.iter() {
            if let Ok(mut fill_tf) = fill_query.get_mut(child) {
                fill_tf.scale.x = ratio;

                // ให้แถบเลือดหดจากขวาไปซ้าย
                fill_tf.translation.x = -full_width * (1.0 - ratio) * 0.5;
            }
        }
    }
}
pub fn respawn_advanced_minion_when_defeated(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,

    advanced_practice_active: Res<AdvancedPracticeActive>,
    mut respawn_timer: ResMut<AdvancedMinionRespawnTimer>,

    minion_query: Query<(), With<GuardianClone>>,
) {
    if !advanced_practice_active.0 {
        respawn_timer.0.reset();
        return;
    }

    // Minion ยังอยู่
    if !minion_query.is_empty() {
        respawn_timer.0.reset();
        return;
    }

    // Minion ถูก defeated แล้ว
    respawn_timer.0.tick(time.delta());

    if !respawn_timer.0.just_finished() {
        return;
    }

    spawn_advanced_minion(
        &mut commands,
        &asset_server,
    );

    respawn_timer.0.reset();
}