use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;

use crate::components::*;
use crate::cel_shader::*;

#[derive(Component)]
pub struct LocalWarpGate {
    pub destination: Vec3,
    pub radius: f32,
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
        Collider::cylinder(10.0, 0.1),
        Transform::from_xyz(0.0, 0.0, 1.25),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //ground2
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(14.0, 0.1),
        Transform::from_xyz(8.7, 20.95, -13.1),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //ground3
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(14.0, 0.1),
        Transform::from_xyz(41.0, 34.4, -23.12),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //three 1/1
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 2.62, 0.3),
        Transform::from_xyz(-3.06, 0.0,-1.38),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //three 1/2
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 2.1, 0.3),
        Transform::from_xyz(-1.4, 0.0,0.38),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //three 1/3
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 2.1, 0.3),
        Transform::from_xyz(0.35, 0.0,5.2),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //three 1/4
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 2.1, 0.3),
        Transform::from_xyz(2.7, 0.0,6.9),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //three 2/1
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 2.1, 0.3),
        Transform::from_xyz(10.91, 20.95,-7.02),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //rock 2/1
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.2, 1.85, 1.87),
        Transform::from_xyz(3.9, 20.95,-5.2),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //three 2/2
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 5.1, 0.3),
        Transform::from_xyz(14.0, 20.95,-16.74),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //rock 3/1
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.4, 2.9, 1.87),
        Transform::from_xyz(33.2, 34.4,-22.59),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //pillar1
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.8, 8.0, 0.8),
        Transform::from_xyz(35.9, 34.4,-23.96),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //pillar2
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.8, 8.0, 0.8),
        Transform::from_xyz(35.9, 34.4,-17.12),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //pillar3
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.8, 8.0, 0.8),
        Transform::from_xyz(41.0, 34.4,-23.96),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //pillar4
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.8, 8.0, 0.8),
        Transform::from_xyz(41.0, 34.4,-17.12),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //three 2/2
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(0.3, 5.1, 0.3),
        Transform::from_xyz(43.21, 34.4,-28.75),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //Local Warp Gates
    let warp_gate_mesh = meshes.add(Cuboid::new(2.0, 3.0, 0.35));

    let warp_gate_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),     // สีประตู = แดง
        normal_map_texture: Some(asset_server.load("textures/tt2_nm.png")), // แปะ normal map
        // base_color_texture: Some(asset_server.load("textures/tt2.png")), // ถ้าอยากได้ลายสีพื้นผิวด้วยค่อยเปิดบรรทัดนี้
        perceptual_roughness: 0.03,  // ผิวกึ่งมัน = ไฮไลต์วิ่งตามลายนูนชัดขึ้นมาก
        metallic: 0.03,
        unlit: false,        // ⚠️ สำคัญมาก: ต้องเป็น false ไม่งั้น normal map จะไม่มีผลเลย
        double_sided: true,
        cull_mode: None,
        flip_normal_map_y: true,
        
        ..default()
    });

    // ประตูชั้นที่ 1 -> ไปออกประตูชั้นที่ 2/1
    commands.spawn((
        LocalWarpGate {
            destination: Vec3::new(0.464, 21.680, -3.793),
            radius: 1.5,
        },
        Sensor,
        Collider::cuboid(2.0, 3.0, 1.0),
        Mesh3d(warp_gate_mesh.clone()),
        MeshMaterial3d(warp_gate_material.clone()),
        Transform::from_xyz(0.829, 0.830, 7.027),
        CurrentScene,
        DespawnOnExit(GameScene::FloatingIsland),
    ));

    // จุดออกประตูชั้นที่ 2/1
    // อันนี้เป็นแค่ภาพประตูปลายทาง ไม่มีระบบวาร์ปกลับ
    // commands.spawn((
    //     Mesh3d(warp_gate_mesh.clone()),
    //     MeshMaterial3d(warp_gate_material.clone()),
    //     Transform::from_xyz(0.464, 21.680, -3.793),
    //     DespawnOnExit(GameScene::FloatingIsland),
    // ));

    // ประตูชั้นที่ 2/2 -> ไปออกประตูชั้นที่ 3
    commands.spawn((
        LocalWarpGate {
            destination: Vec3::new(29.989, 35.230, -22.517),
            radius: 1.5,
        },
        Sensor,
        Collider::cuboid(2.0, 3.0, 1.0),
        Mesh3d(warp_gate_mesh.clone()),
        MeshMaterial3d(warp_gate_material.clone()),
        Transform::from_xyz(15.858, 21.680, -22.981),
        CurrentScene,
        DespawnOnExit(GameScene::FloatingIsland),
    ));

    // จุดออกประตูชั้นที่ 3
    // commands.spawn((
    //     Mesh3d(warp_gate_mesh.clone()),
    //     MeshMaterial3d(warp_gate_material.clone()),
    //     Transform::from_xyz(29.989, 35.230, -22.517),
    //     DespawnOnExit(GameScene::FloatingIsland),
    // ));
    
    //warp to desert
    commands.spawn((
        WarpToDesert,
        CurrentScene,
        Sensor,
        Collider::cuboid(2.0, 2.0, 1.0),
        Transform::from_xyz(5.0, 1.0, -5.0),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //warp to lagoon
    commands.spawn((
        WarpToLagoon,
        CurrentScene,
        Sensor,
        Collider::cuboid(2.0, 2.0, 1.0),
        //Transform::from_xyz(5.0, 1.0, 0.6),
        Transform::from_xyz(38.58, 35.23, -20.62),
        DespawnOnExit(GameScene::FloatingIsland),
    ));
    //sound
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/BGM_Dimension.ogg")),
        PlaybackSettings{mode: bevy::audio::PlaybackMode::Loop, volume:  bevy::audio::Volume::Linear(0.1), ..default()},
        MusicAudio,
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

pub fn update_local_warp_gates(
    time: Res<Time>,
    mut player_query: Query<
        (&mut Transform, Option<&mut LinearVelocity>),
        (With<Player>, Without<LocalWarpGate>),
    >,
    gate_query: Query<(&Transform, &LocalWarpGate), Without<Player>>,
    mut cooldown: Local<f32>,
) {
    const WARP_COOLDOWN: f32 = 1.0;

    if *cooldown > 0.0 {
        *cooldown -= time.delta().as_secs_f32();

        if *cooldown < 0.0 {
            *cooldown = 0.0;
        }

        return;
    }

    let Ok((mut player_tf, player_vel)) = player_query.single_mut() else {
        return;
    };

    let mut target_destination: Option<Vec3> = None;

    for (gate_tf, gate) in &gate_query {
        let distance = player_tf.translation.distance(gate_tf.translation);

        if distance < gate.radius {
            target_destination = Some(gate.destination);
            break;
        }
    }

    if let Some(destination) = target_destination {
        player_tf.translation = destination;

        if let Some(mut vel) = player_vel {
            vel.0 = Vec3::ZERO;
        }

        *cooldown = WARP_COOLDOWN;
    }
}