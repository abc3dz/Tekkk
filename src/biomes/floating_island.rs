use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;

use crate::components::*;
use crate::cel_shader::*;

#[derive(Component, PartialEq, Clone, Copy)]
pub enum AutoPlatformPhase {
    Rising,
    Waiting,
    Hidden,
}

#[derive(Component)]
pub struct AutoPlatform {
    pub min_y: f32,
    pub max_y: f32,
    pub speed: f32,
    pub phase: AutoPlatformPhase,
    pub wait_timer: Timer,
    pub hidden_timer: Timer,
}

pub fn spawn_floating_island(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("maps/EvrmFloatingIsland.glb"))),
        ApplyToonMaterial,
        Transform::from_xyz(0.0, -0.1, 0.0),
        CurrentScene,
    ));
    //ground
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(9.8, 0.1),
        Transform::from_xyz(0.0, 0.0, 1.25),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //ground2
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(25.6, 0.1, 28.5),
        Transform::from_xyz(8.7, 20.85, -13.1),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //ground3
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(12.5, 0.1),
        Transform::from_xyz(41.0, 34.4, -23.12),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    
    // Mesh ของแพลตฟอร์ม ใช้ร่วมกันสองอัน
    let platform_mesh = meshes.add(Cuboid::new(4.0, 0.5, 4.0));

    // สีแพลตฟอร์ม
    let platform_material_1 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.65, 1.0),
        ..default()
    });

    let platform_material_2 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.65, 0.2),
        ..default()
    });

    // Platform อันที่ 1
    commands.spawn((
        RigidBody::Kinematic,
        Collider::cuboid(4.0, 0.5, 4.0),
        Mesh3d(platform_mesh.clone()),
        MeshMaterial3d(platform_material_1),
        Visibility::Visible,
        Transform::from_xyz(3.7, 0.83, 2.98),
        AutoPlatform {
            min_y: 0.0,
            max_y: 22.0,
            speed: 2.0,
            phase: AutoPlatformPhase::Rising,
            wait_timer: Timer::from_seconds(2.0, TimerMode::Once),
            hidden_timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
        LinearVelocity(Vec3::new(0.0, 2.0, 0.0)),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    // Platform อันที่ 2
    commands.spawn((
        RigidBody::Kinematic,
        Collider::cuboid(4.0, 0.5, 4.0),
        Mesh3d(platform_mesh.clone()),
        MeshMaterial3d(platform_material_2),
        Visibility::Visible,
        Transform::from_xyz(18.58, 21.68, -21.48),
        AutoPlatform {
            min_y: 20.0,
            max_y: 34.0,
            speed: 2.0,
            phase: AutoPlatformPhase::Rising,
            wait_timer: Timer::from_seconds(2.0, TimerMode::Once),
            hidden_timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
        LinearVelocity(Vec3::new(0.0, 2.0, 0.0)),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    
    //warp to desert
    commands.spawn((
        WarpToDesert,
        CurrentScene,
        Sensor,
        Collider::cuboid(2.0, 2.0, 2.0),
        Transform::from_xyz(0.0, 1.0, -5.0),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //warp to lagoon
    commands.spawn((
        WarpToLagoon,
        CurrentScene,
        Sensor,
        Collider::cuboid(2.0, 2.0, 2.0),
        //Transform::from_xyz(5.0, 1.0, 0.6),
        Transform::from_xyz(38.58, 35.23, -20.62),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
}

pub fn go_to_floating_island(
    mut next_state: ResMut<NextState<GameScene>>,
    mut scene_spawn: ResMut<SceneSpawnPoint>,
) {
    scene_spawn.position = Some(Vec3::new(0.0, 2.0, 0.0));
    next_state.set(GameScene::FloatingIsland);
    println!("Entering FloatingIsland");
}

pub fn check_warp_to_floating_island(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToFloatingIsland>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);
        if distance < 2.0 {
            next_state.set(GameScene::LoadingFloatingIsland);
        }
    }
}

pub fn check_warp_to_desert_from_floating(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToDesert>>, // ใช้ Component เดิมเพื่อเดินทางกลับ
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);
        if distance < 2.0 {
            next_state.set(GameScene::LoadingDesert);
        }
    }
}

pub fn setup_floating_island_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 40_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ,-0.8,-0.3,0.0)),
        CurrentScene,
    ));
}

pub fn update_auto_platforms(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut LinearVelocity,
        &mut Visibility,
        &mut AutoPlatform,
    )>,
) {
    for (entity, mut tf, mut vel, mut visibility, mut platform) in &mut query {
        match platform.phase {
            AutoPlatformPhase::Rising => {
                vel.0.x = 0.0;
                vel.0.z = 0.0;
                vel.0.y = platform.speed;

                // ถ้าถึงจุดสูงสุด
                if tf.translation.y >= platform.max_y {
                    tf.translation.y = platform.max_y;
                    vel.0 = Vec3::ZERO;

                    platform.phase = AutoPlatformPhase::Waiting;
                    platform.wait_timer.reset();
                }
            }

            AutoPlatformPhase::Waiting => {
                vel.0 = Vec3::ZERO;

                platform.wait_timer.tick(time.delta());

                // ค้างครบ 2 วินาที แล้วหายไป
                if platform.wait_timer.is_finished() {
                    platform.phase = AutoPlatformPhase::Hidden;

                    *visibility = Visibility::Hidden;

                    // ปิดการชน
                    commands.entity(entity).insert(CollisionLayers::NONE);

                    platform.hidden_timer.reset();
                }
            }

            AutoPlatformPhase::Hidden => {
                vel.0 = Vec3::ZERO;

                platform.hidden_timer.tick(time.delta());

                // หายไปครบเวลาแล้ว กลับมาที่จุดเริ่มต้นและเลื่อนขึ้นใหม่
                if platform.hidden_timer.is_finished() {
                    tf.translation.y = platform.min_y;

                    *visibility = Visibility::Visible;

                    // เปิดการชนอีกครั้ง
                    commands.entity(entity).insert(CollisionLayers::ALL);

                    platform.phase = AutoPlatformPhase::Rising;
                    vel.0.x = 0.0;
                    vel.0.z = 0.0;
                    vel.0.y = platform.speed;
                }
            }
        }
    }
}