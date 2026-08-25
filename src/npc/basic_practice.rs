use bevy::prelude::*;
use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use rand::Rng;
use crate::cel_shader::*;
use crate::components::*;
use crate::npc::practice_common::spawn_enemy_health_bar;
use crate::player::{
    play_player_hurt_animation,
    spawn_floating_damage_text,
};
use crate::camera::*;
use crate::pause_menu::GameMode;

pub struct BasicPracticePlugin;

impl Plugin for BasicPracticePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(
                BasicGunRespawnTimer(
                    Timer::from_seconds(
                        3.0,
                        TimerMode::Once,
                    ),
                ),
            )
            //.add_systems(Update,guardian_dialog_basic_input.run_if(in_state(GameScene::Hub)),)
            .add_systems(Update,(
                    rotate_basic_practice_gun_to_player,
                    basic_practice_gun_shoot_projectile,
                    move_basic_practice_projectiles,
                    basic_projectile_hit_player,
                    respawn_basic_gun_when_defeated,
                ).run_if(in_state(GameScene::Hub).and(in_state(GameMode::Playing))),
            );
    }
}

pub fn spawn_basic_practice_gun(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let mut rng = rand::rng();
    let x = rng.random_range(-4.0..=10.0);
    let y = 0.0;
    let z = rng.random_range(-4.0..=10.0);
    let base_stats = BaseStats::BASIC_PRACTICE_GUN;
    const BASIC_GUN_BODY_Y: f32 = 0.7;

let gun_entity = commands
        .spawn((
            HubOnly,
            PracticeEntity,
            BasicPracticeGun,
            CombatTarget,

            Health {
                current: base_stats.max_hp as i32,
                max: base_stats.max_hp as i32,
            },

            base_stats,
            CombatStats::from(base_stats),
            AtkAndDefElement(Element::Neutral),
            ElementExpReward::BASIC_PRACTICE_GUN,

            BasicGunShootTimer(
                Timer::from_seconds(
                    3.0,
                    TimerMode::Repeating,
                ),
            ),

            RigidBody::Kinematic,
            Collider::cuboid(0.5, 0.7, 0.5),

            Transform::from_xyz(
                x,
                y + BASIC_GUN_BODY_Y,
                z,
            ),

            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(0)
                            .from_asset(
                                "npc/BasicPracticeGun.glb",
                            ),
                    ),
                ),

                Transform::from_xyz(
                    0.0,
                    -BASIC_GUN_BODY_Y,
                    0.0,
                ),
                ApplyToonMaterial
                // WindWakerShaderBuilder::default()
                //     .time_of_day(TimeOfDay::Day)
                //     .weather(Weather::Sunny)
                //     .build(),
            ));
        })
        .id();

    spawn_enemy_health_bar(
        commands,
        gun_entity,
    );
}

// pub fn guardian_dialog_basic_input(
//     mut commands: Commands,
//     keyboard: Res<ButtonInput<KeyCode>>,
//     gamepads: Query<&Gamepad>,
//     asset_server: Res<AssetServer>,
//     mut basic_practice_active: ResMut<BasicPracticeActive>,
//     mut advanced_practice_active: ResMut<AdvancedPracticeActive>,
//     mut respawn_timer: ResMut<BasicGunRespawnTimer>,
//     dialog_query: Query<Entity, With<GuardianDialogUI>>,
//     practice_query: Query<Entity, With<PracticeEntity>>,
//     mut player_query: Query<&mut Transform, With<Player>>,
// ) {
//     if dialog_query.is_empty() {
//         return;
//     }

//     let gamepad_basic_pressed = gamepads.iter().any(|gamepad| {gamepad.just_pressed(GamepadButton::South,)});
//     let basic_pressed = keyboard.just_pressed(KeyCode::Digit1) || gamepad_basic_pressed;
//     if !basic_pressed {
//         return;
//     }

//     commands.spawn(AudioPlayer::new(asset_server.load("sounds/npc/basic_pt.ogg")));
//     basic_practice_active.0 = true;
//     advanced_practice_active.0 = false;
//     respawn_timer.0.reset();

//     for entity in &practice_query {
//         commands.entity(entity).despawn();
//     }

//     if let Ok(mut transform) = player_query.single_mut() {
//         transform.translation =
//             Vec3::new(0.0, 0.0, 0.0);
//     }

//     spawn_basic_practice_gun(
//         &mut commands,
//         &asset_server,
//     );
// }

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
    mut gun_query: Query<(&Transform, &mut BasicGunShootTimer), (With<BasicPracticeGun>, Without<Player>),>,
) {
    let Ok(player_tf) = player_query.single() else { return };

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
        let speed = 5.0;
        let mut spawn_pos = gun_tf.translation + direction * 0.3;
        spawn_pos.y = 0.5;
        let yaw = direction.x.atan2(direction.z);

        commands.spawn((
            PracticeEntity,
            BasicPracticeProjectile {velocity: direction * speed, hp_damage: 5},
            ProjectileLifetime(Timer::from_seconds(4.0, TimerMode::Once)),
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("npc/BasicPracticeProjectile.glb"))),
            Transform {translation: spawn_pos, rotation: Quat::from_rotation_y(yaw), scale: Vec3::splat(1.0), ..default()},
            AudioPlayer::new(asset_server.load("sounds/npc/basic_atk.ogg")),
            GlobalTransform::default(),
        ));
    }
}

pub fn move_basic_practice_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Transform, &BasicPracticeProjectile, &mut ProjectileLifetime)>,
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
    asset_server: Res<AssetServer>,
    projectile_query: Query<(Entity, &Transform, &BasicPracticeProjectile), (With<BasicPracticeProjectile>, Without<Player>)>,
    mut player_query: Query<(Entity, &Transform, &mut Health), (With<Player>, Without<BasicPracticeProjectile>)>,
    anim_graph: Res<PlayerAnimationGraph>,
    mut anim_query: Query<(&mut AnimationPlayer, &mut PlayerAnimState), With<PlayerAnimationTarget>>,
    camera_query: Query<Entity, With<MainCamera>>,
) {
    let Ok((player_entity, player_tf, mut health)) = player_query.single_mut()
    else { return };

    for (projectile_entity, projectile_tf, projectile) in &projectile_query {
        let distance =
            player_tf.translation.distance(projectile_tf.translation);

        if distance >= 0.8 { continue }

        health.current -= projectile.hp_damage;
        health.current = health.current.clamp(0, health.max);

        spawn_floating_damage_text(
            &mut commands,
            projectile.hp_damage,
            player_tf.translation + Vec3::new(0.0, 2.0, 0.0),
            FloatingDamageKind::PlayerHit,
        );

        play_player_hurt_animation(
            &mut commands,
            player_entity,
            &anim_graph,
            &mut anim_query,
            &camera_query,
            &asset_server
        );

        commands.entity(projectile_entity).despawn();
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
    if !basic_practice_active.0 {
        respawn_timer.0.reset();
        return;
    }
    if !basic_gun_query.is_empty() {
        respawn_timer.0.reset();
        return;
    }
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