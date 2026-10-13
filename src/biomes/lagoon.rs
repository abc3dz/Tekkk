use bevy::prelude::*;
use avian3d::prelude::*; // ใช้ Avian3D สำหรับฟิสิกส์
use crate::components::*;

const TILE_SIZE: f32 = 10.0;
const GAP: f32 = 1.2;
const STEP: f32 = TILE_SIZE + GAP;
const TILES_PER_DIRECTION: i32 = 12;
const WARP_DISTANCE: f32 = 2.5;

pub fn check_warp_to_lagoon(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToLagoon>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() else { return; };
    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);
        if distance < WARP_DISTANCE {
            next_state.set(GameScene::LoadingLagoon);
        }
    }
}

// Loading -> Lagoon
pub fn go_to_lagoon(
    mut next_state: ResMut<NextState<GameScene>>,
    mut scene_spawn: ResMut<SceneSpawnPoint>,
) {
    scene_spawn.position = Some(Vec3::new(0.0, 2.0, 0.0));
    next_state.set(GameScene::Lagoon);
    println!("Entering Lagoon");
}

// ---------------------------------------------------------------
// Spawn ฉาก Lagoon
// ---------------------------------------------------------------
pub fn spawn_lagoon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tile_mesh = meshes.add(Plane3d::new(
        Vec3::Y,
        Vec2::splat(TILE_SIZE * 0.5),
    ));

    let tile_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.45, 0.55),
        perceptual_roughness: 0.35,
        metallic: 0.05,
        ..default()
    });

    let center_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.60, 0.70),
        perceptual_roughness: 0.30,
        metallic: 0.05,
        ..default()
    });

    // แผ่นกลางฉาก
    spawn_tile(
        &mut commands,
        tile_mesh.clone(),
        center_material.clone(),
        Vec3::ZERO,
    );

    // 4 ทิศทาง: +x, -x, +z, -z
    let directions = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, -1.0),
    ];

    for dir in directions {
        for i in 1..=TILES_PER_DIRECTION {
            let position = dir * STEP * i as f32;
            spawn_tile(
                &mut commands,
                tile_mesh.clone(),
                tile_material.clone(),
                position,
            );
        }
    }
}

// ---------------------------------------------------------------
// ฟังก์ชันสร้างแผ่นพื้น (แก้ไขตรงนี้สำคัญสุด)
// ---------------------------------------------------------------
fn spawn_tile(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    translation: Vec3,
) {
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(translation),
        
        // ✨ บังคับให้เป็นพื้นแข็งที่เหยียบได้
        RigidBody::Static,
        // ขนาด Collider กว้าง 10, สูง 0.1 (บางๆ), ลึก 10 
        // ตรงกับ Plane3d ขนาด 10x10 พอดี
        Collider::cuboid(TILE_SIZE, 0.1, TILE_SIZE),
        
        CurrentScene,
    ));
}

// ---------------------------------------------------------------
// แสงในฉาก Lagoon
// ---------------------------------------------------------------
pub fn setup_lagoon_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 35_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.8,
            -0.3,
            0.0,
        )),
        CurrentScene,
    ));
}

pub fn spawn_back_to_floating_island_portal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let portal_position = Vec3::new(-STEP * 3.0, 1.5, 0.0);
    let mesh = meshes.add(Cuboid::new(2.0, 3.0, 0.4));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.75, 0.25, 0.75),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.2,
        metallic: 0.1,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(portal_position),
        RigidBody::Static,
        Collider::cuboid(2.0, 3.0, 0.4),
        WarpToFloatingIsland,
        CurrentScene,
    ));
}

pub fn check_warp_back_to_floating_island(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToFloatingIsland>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() else { return; };
    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);
        if distance < WARP_DISTANCE {
            next_state.set(GameScene::LoadingFloatingIsland);
        }
    }
}