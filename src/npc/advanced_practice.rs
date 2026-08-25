use bevy::prelude::*;
use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
//use bevy_wind_waker_shader::prelude::*;
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

pub struct AdvancedPracticePlugin;

impl Plugin for AdvancedPracticePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(
                AdvancedMinionRespawnTimer(
                    Timer::from_seconds(
                        2.0,
                        TimerMode::Once,
                    ),
                ),
            )
            //.add_systems(Update,guardian_dialog_advanced_input.run_if(in_state(GameScene::Hub)))
            .add_systems(Update,(
                    minion_chase_player,
                    minion_drain_player_life,
                    respawn_advanced_minion_when_defeated,
                ).run_if(in_state(GameScene::Hub).and(in_state(GameMode::Playing))),
                    
            );
    }
}

// pub fn guardian_dialog_advanced_input(
//     mut commands: Commands,
//     keyboard: Res<ButtonInput<KeyCode>>,
//     gamepads: Query<&Gamepad>,
//     asset_server: Res<AssetServer>,
//     mut basic_practice_active: ResMut<BasicPracticeActive>,
//     mut advanced_practice_active: ResMut<AdvancedPracticeActive>,
//     mut advanced_respawn_timer: ResMut<AdvancedMinionRespawnTimer>,
//     dialog_query: Query<Entity, With<GuardianDialogUI>>,
//     practice_query: Query<Entity, With<PracticeEntity>>,
//     mut player_query: Query<&mut Transform, With<Player>>,
// ) {
//     if dialog_query.is_empty() {
//         return;
//     }

//     let gamepad_advanced_pressed = gamepads.iter().any(|gamepad| {gamepad.just_pressed(GamepadButton::West)});
//     let advanced_pressed = keyboard.just_pressed(KeyCode::Digit2) || gamepad_advanced_pressed;

//     if !advanced_pressed {
//         return;
//     }

//     commands.spawn(AudioPlayer::new(asset_server.load("sounds/npc/advance_pt.ogg")));

//     basic_practice_active.0 = false;
//     advanced_practice_active.0 = true;
//     advanced_respawn_timer.0.reset();

//     for entity in &practice_query {
//         commands.entity(entity).despawn();
//     }

//     spawn_advanced_minion(
//         &mut commands,
//         &asset_server,
//     );

//     if let Ok(mut player_tf) = player_query.single_mut() {
//         player_tf.translation = Vec3::new(0.0, 0.0, 0.0);
//     }
// }

pub fn spawn_advanced_minion(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let mut rng = rand::rng();
    let x = rng.random_range(-4.0..=10.0);
    let y = 0.0;
    let z = rng.random_range(-4.0..=10.0);
    let base_stats = BaseStats::ADVANCED_PRACTICE_MINION;
    const ADVANCED_MINION_BODY_Y: f32 = 0.8;

let minion_entity = commands
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
                Timer::from_seconds(
                    1.0,
                    TimerMode::Repeating,
                ),
            ),

            RigidBody::Kinematic,
            Collider::capsule(0.35, 0.8),

            Transform::from_xyz(
                x,
                y + ADVANCED_MINION_BODY_Y,
                z,
            ),

            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            parent.spawn((SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("npc/MinionChar.glb"))),
                Transform::from_xyz(0.0,-ADVANCED_MINION_BODY_Y,0.0,),
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
        minion_entity,
    );
}

pub fn minion_chase_player(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<GuardianClone>)>,
    mut clone_query: Query<&mut Transform, (With<GuardianClone>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.single() else { return };

    for mut clone_tf in &mut clone_query {
        let mut direction = player_tf.translation - clone_tf.translation;
        direction.y = 0.0;
        let distance = direction.length();
        if distance < 1.0 { continue }
        let move_dir = direction.normalize();
        let speed = 2.5;
        clone_tf.translation += move_dir * speed * time.delta_secs();
        let yaw = move_dir.x.atan2(move_dir.z);
        clone_tf.rotation = Quat::from_rotation_y(yaw);
    }
}

pub fn minion_drain_player_life(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    anim_graph: Res<PlayerAnimationGraph>,
    mut anim_query: Query<(&mut AnimationPlayer, &mut PlayerAnimState), With<PlayerAnimationTarget>,>,
    mut player_query: Query<(Entity, &Transform, &mut Health),(With<Player>, Without<GuardianClone>),>,
    mut minion_query: Query<(&Transform, &mut Health, &mut MinionLifeDrainTimer),(With<GuardianClone>, Without<Player>)>,
    camera_query: Query<Entity, With<MainCamera>>,
) {
    let Ok((player_entity, player_tf, mut player_health)) = player_query.single_mut() else { return };

    for (minion_tf, mut minion_health, mut drain_timer) in &mut minion_query {
        drain_timer.0.tick(time.delta());
        let distance = player_tf.translation.distance(minion_tf.translation);
        if distance >= 1.4 || !drain_timer.0.just_finished() { continue}
        let drain_amount = 5;
        player_health.current -= drain_amount;
        player_health.current = player_health.current.clamp(0, player_health.max);

        spawn_floating_damage_text(
            &mut commands,
            drain_amount,
            player_tf.translation
                + Vec3::new(0.0, 2.0, 0.0),
            FloatingDamageKind::PlayerDrain,
        );
        commands.spawn(AudioPlayer::new(asset_server.load("sounds/npc/advance_atk.ogg")));
        minion_health.current += drain_amount;
        minion_health.current = minion_health.current.clamp(0, minion_health.max);

        play_player_hurt_animation(
            &mut commands,
            player_entity,
            &anim_graph,
            &mut anim_query,
            &camera_query,
            &asset_server
        );
        
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

    if !minion_query.is_empty() {
        respawn_timer.0.reset();
        return;
    }

    respawn_timer.0.tick(time.delta());

    if !respawn_timer.0.just_finished() { return }

    spawn_advanced_minion(
        &mut commands,
        &asset_server,
    );

    respawn_timer.0.reset();
}