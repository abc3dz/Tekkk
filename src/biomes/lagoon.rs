use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::*;
use crate::cel_shader::*;
use std::f32::consts::TAU;

// ---------------------------------------------------------------------
// ค่าทั่วไป
// ---------------------------------------------------------------------
const WARP_DISTANCE: f32 = 2.5;

// เผื่อโค้ดอื่นเคยเรียก lagoon::STEP มาก่อน
#[allow(dead_code)]
pub const STEP: f32 = 11.2;

// ---------------------------------------------------------------------
// โครงสร้างฉาก
// ---------------------------------------------------------------------
const GROUND_SIZE: f32 = 120.0;
const GROUND_HEIGHT: f32 = 0.2;

const WATER_SIZE: f32 = 26.0;
const WATER_LEVEL: f32 = 0.07;

const WALKWAY_WIDTH: f32 = 3.0;
const WALKWAY_HEIGHT: f32 = 0.18;

// ระยะกึ่งกลางของทางเดินรอบสระน้ำตื้น
// น้ำกว้าง 26 -> ครึ่งหนึ่ง 13
// ทางเดินกว้าง 3 -> ครึ่งหนึ่ง 1.5
// ดังนั้นแนวกลางทางเดินอยู่ที่ 14.5 จากจุดศูนย์กลาง
const WALKWAY_HALF_SPAN: f32 = WATER_SIZE / 2.0 + WALKWAY_WIDTH / 2.0;

// ความยาวของทางเดินด้านทิศบน/ล่าง
const WALKWAY_LENGTH: f32 = WATER_SIZE + WALKWAY_WIDTH * 2.0;

// ภูเขาล้อมรอบ
const MOUNTAIN_COUNT: usize = 32;
const MOUNTAIN_RING_RADIUS: f32 = 44.0;

// ---------------------------------------------------------------------
// ตรวจว่าผู้เล่นเดินเข้าใกล้ประตูไป Lagoon หรือยัง
// ---------------------------------------------------------------------
pub fn check_warp_to_lagoon(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToLagoon>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);

        if distance < WARP_DISTANCE {
            next_state.set(GameScene::LoadingLagoon);
        }
    }
}

// ---------------------------------------------------------------------
// Loading -> Lagoon
// ---------------------------------------------------------------------
pub fn go_to_lagoon(
    mut next_state: ResMut<NextState<GameScene>>,
    mut scene_spawn: ResMut<SceneSpawnPoint>,
) {
    // ให้ผู้เล่นเกิดบนทางเดินไม้ด้านทิศ +z
    scene_spawn.position = Some(Vec3::new(0.0, 2.0, WALKWAY_HALF_SPAN));
    next_state.set(GameScene::Lagoon);

    println!("Entering Lagoon");
}

// ---------------------------------------------------------------------
// Spawn ฉาก Lagoon
// ---------------------------------------------------------------------
pub fn spawn_lagoon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // พื้นดิน/ทรายรองรับทั้งฉาก
    spawn_lagoon_ground(&mut commands, &mut meshes, &mut materials);

    // น้ำตื้นกลางฉาก
    spawn_shallow_water(&mut commands, &mut meshes, &mut materials);

    // ทางเดินไม้รอบน้ำตื้น
    spawn_walkways(&mut commands, &mut meshes, &mut materials);

    // บังกะโล 4 หลัง
    spawn_bungalows(&mut commands, &mut meshes, &mut materials);

    // ภูเขาล้อมรอบ
    spawn_surrounding_mountains(&mut commands, &mut meshes, &mut materials);

    // รายละเอียดตกแต่งแบบ procedural สำหรับ Prototype
    spawn_lagoon_decorations(&mut commands, &mut meshes, &mut materials);
}

// ---------------------------------------------------------------------
// พื้นดินขนาดใหญ่
// ---------------------------------------------------------------------
fn spawn_lagoon_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.62, 0.48),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });

    spawn_box(
        commands,
        meshes,
        material,
        Vec3::new(GROUND_SIZE, GROUND_HEIGHT, GROUND_SIZE),
        Vec3::new(0.0, -GROUND_HEIGHT * 0.5, 0.0),
        true,
    );
}

// ---------------------------------------------------------------------
// น้ำตื้น
// ---------------------------------------------------------------------
fn spawn_shallow_water(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.10, 0.55, 0.75, 0.55),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.15,
        metallic: 0.0,
        ..default()
    });

    // ไม่ใส่ Collider เพื่อให้ผู้เล่นเดินบนพื้นดินใต้น้ำได้
    spawn_box(
        commands,
        meshes,
        material,
        Vec3::new(WATER_SIZE, 0.02, WATER_SIZE),
        Vec3::new(0.0, WATER_LEVEL - 0.01, 0.0),
        false,
    );
}

// ---------------------------------------------------------------------
// ทางเดินไม้รอบน้ำตื้น
// ---------------------------------------------------------------------
fn spawn_walkways(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.30, 0.18),
        perceptual_roughness: 0.7,
        metallic: 0.0,
        ..default()
    });

    let y = WALKWAY_HEIGHT * 0.5;
    let offset = WALKWAY_HALF_SPAN;

    // ทางเดินบน/ล่าง
    let horizontal_size = Vec3::new(WALKWAY_LENGTH, WALKWAY_HEIGHT, WALKWAY_WIDTH);

    // ทางเดินซ้าย/ขวา
    // ใช้ความยาวแค่เท่ากับน้ำ เพื่อไม่ให้ทับมุมซ้ำกับบน/ล่างมากเกินไป
    let vertical_size = Vec3::new(WALKWAY_WIDTH, WALKWAY_HEIGHT, WATER_SIZE);

    // ด้าน -z
    spawn_box(
        commands,
        meshes,
        wood.clone(),
        horizontal_size,
        Vec3::new(0.0, y, -offset),
        true,
    );

    // ด้าน +z
    spawn_box(
        commands,
        meshes,
        wood.clone(),
        horizontal_size,
        Vec3::new(0.0, y, offset),
        true,
    );

    // ด้าน -x
    spawn_box(
        commands,
        meshes,
        wood.clone(),
        vertical_size,
        Vec3::new(-offset, y, 0.0),
        true,
    );

    // ด้าน +x
    spawn_box(
        commands,
        meshes,
        wood.clone(),
        vertical_size,
        Vec3::new(offset, y, 0.0),
        true,
    );
}

// ---------------------------------------------------------------------
// บังกะโล 4 หลัง
// ---------------------------------------------------------------------
fn spawn_bungalows(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let offset = WALKWAY_HALF_SPAN + 6.0;

    let positions = [
        Vec3::new(offset, 0.0, offset),
        Vec3::new(-offset, 0.0, offset),
        Vec3::new(offset, 0.0, -offset),
        Vec3::new(-offset, 0.0, -offset),
    ];

    let platform_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.24, 0.16),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });

    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.40, 0.26),
        perceptual_roughness: 0.75,
        metallic: 0.0,
        ..default()
    });

    let roof_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.18, 0.12),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });

    for position in positions {
        // หันหน้าบังกะโลเข้าหาจุดกลางฉาก
        let rotation = Quat::from_rotation_y((-position.x).atan2(-position.z));

        spawn_bungalow(
            commands,
            meshes,
            platform_material.clone(),
            wall_material.clone(),
            roof_material.clone(),
            position,
            rotation,
        );
    }
}

fn spawn_bungalow(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    platform_material: Handle<StandardMaterial>,
    wall_material: Handle<StandardMaterial>,
    roof_material: Handle<StandardMaterial>,
    position: Vec3,
    rotation: Quat,
) {
    // แพลตฟอร์มพื้น
    let platform_size = Vec3::new(5.6, 0.35, 5.6);
    let platform_translation = position + Vec3::new(0.0, platform_size.y * 0.5, 0.0);

    spawn_box(
        commands,
        meshes,
        platform_material,
        platform_size,
        platform_translation,
        true,
    );

    // ตัวบ้าน
    let wall_size = Vec3::new(4.2, 2.4, 4.2);
    let wall_translation =
        position + Vec3::new(0.0, platform_size.y + wall_size.y * 0.5, 0.0);

    spawn_box(
        commands,
        meshes,
        wall_material,
        wall_size,
        wall_translation,
        true,
    );

    // หลังคา
    let roof_radius = 3.8;
    let roof_height = 2.4;

    let roof_translation = position
        + Vec3::new(
            0.0,
            platform_size.y + wall_size.y + roof_height * 0.5,
            0.0,
        );

    spawn_cone(
        commands,
        meshes,
        roof_material,
        roof_radius,
        roof_height,
        roof_translation,
        rotation,
        false,
    );
}

// ---------------------------------------------------------------------
// ภูเขาล้อมรอบ
// ---------------------------------------------------------------------
fn spawn_surrounding_mountains(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mountain_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.40, 0.26),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });

    for i in 0..MOUNTAIN_COUNT {
        let t = i as f32;
        let angle = t / MOUNTAIN_COUNT as f32 * TAU;

        let r1 = pseudo_random(i, 17.31);
        let r2 = pseudo_random(i, 91.17);
        let r3 = pseudo_random(i, 43.77);

        let radius = 5.0 + r1 * 4.5;
        let height = 10.0 + r2 * 14.0;

        let ring_radius = MOUNTAIN_RING_RADIUS + (r3 - 0.5) * 5.0;

        let x = angle.cos() * ring_radius;
        let z = angle.sin() * ring_radius;

        // กดลงดินเล็กน้อยเพื่อให้ฐานดูไม่ลอย
        let translation = Vec3::new(x, height * 0.5 - 0.3, z);

        let rotation = Quat::from_rotation_y(angle);

        spawn_cone(
            commands,
            meshes,
            mountain_material.clone(),
            radius,
            height,
            translation,
            rotation,
            true,
        );
    }
}


// ---------------------------------------------------------------------
// รายละเอียดตกแต่ง Lagoon: ต้นปาล์ม หิน และใบบัว
// สร้างจาก primitive meshes ทั้งหมด ไม่ต้องใช้ external assets
// ---------------------------------------------------------------------
fn spawn_lagoon_decorations(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.38, 0.24, 0.13),
        perceptual_roughness: 0.9,
        ..default()
    });
    let leaf_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.42, 0.22),
        perceptual_roughness: 0.85,
        ..default()
    });
    let leaf_light_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.57, 0.27),
        perceptual_roughness: 0.85,
        ..default()
    });
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.38, 0.43, 0.40),
        perceptual_roughness: 1.0,
        ..default()
    });
    let lily_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.48, 0.28),
        perceptual_roughness: 0.9,
        ..default()
    });

    // วางต้นปาล์มตามขอบฉาก โดยไม่กีดขวางทางเดินกลาง
    let palm_positions = [
        Vec3::new(-23.0, 0.0, -18.0),
        Vec3::new(23.0, 0.0, -18.0),
        Vec3::new(-24.0, 0.0, 18.0),
        Vec3::new(24.0, 0.0, 18.0),
        Vec3::new(-34.0, 0.0, 3.0),
        Vec3::new(34.0, 0.0, -3.0),
    ];

    for (i, p) in palm_positions.iter().enumerate() {
        let height = if i % 2 == 0 { 4.2 } else { 3.6 };
        let trunk_mesh = meshes.add(Cylinder::new(0.24, height));
        commands.spawn((
            Mesh3d(trunk_mesh),
            MeshMaterial3d(trunk_mat.clone()),
            Transform::from_translation(*p + Vec3::Y * (height * 0.5)),
            CurrentScene,
            ApplyToonMaterial,
        ));

        let crown = *p + Vec3::Y * (height + 0.1);
        // ใบปาล์ม 5 ใบเป็นกรวยเรียวยื่นออกจากยอด
        for leaf in 0..5 {
            let angle = leaf as f32 / 5.0 * TAU + i as f32 * 0.37;
            let direction = Vec3::new(angle.cos(), 0.25, angle.sin());
            let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
            let leaf_pos = crown + direction * 0.85;
            spawn_cone(
                commands,
                meshes,
                if leaf % 2 == 0 { leaf_mat.clone() } else { leaf_light_mat.clone() },
                0.42,
                2.0,
                leaf_pos,
                rotation,
                false,
            );
        }
    }

    // กลุ่มหินตกแต่งริมสระและขอบพื้นที่
    let rock_positions = [
        Vec3::new(-18.0, 0.0, -3.0),
        Vec3::new(18.0, 0.0, 4.0),
        Vec3::new(-31.0, 0.0, -12.0),
        Vec3::new(30.0, 0.0, 14.0),
        Vec3::new(0.0, 0.0, -31.0),
        Vec3::new(-7.0, 0.0, 31.0),
    ];
    for (i, p) in rock_positions.iter().enumerate() {
        let scale = 0.8 + (i % 3) as f32 * 0.35;
        let size = Vec3::new(1.3 * scale, 0.8 * scale, 1.0 * scale);
        let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(rock_mat.clone()),
            Transform::from_translation(*p + Vec3::Y * (size.y * 0.5))
                .with_rotation(Quat::from_rotation_y(i as f32 * 0.71)),
            CurrentScene,
            ApplyToonMaterial,
        ));
    }

    // ใบบัวแบน ๆ บนผิวน้ำ เป็น landmark เล็ก ๆ ไม่ใช่ collider
    for (x, z, radius) in [
        (-5.0, -4.0, 0.8),
        (4.5, 3.5, 1.0),
        (7.0, -6.0, 0.65),
        (-7.0, 6.0, 0.7),
    ] {
        let mesh = meshes.add(Cylinder::new(radius, 0.08));
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(lily_mat.clone()),
            Transform::from_xyz(x, WATER_LEVEL + 0.04, z),
            CurrentScene,
            ApplyToonMaterial,
        ));
    }
}

// ---------------------------------------------------------------------
// แสงในฉาก Lagoon
// ---------------------------------------------------------------------
pub fn setup_lagoon_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 30_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.9,
            -0.25,
            0.0,
        )),
        CurrentScene,
    ));
}

// ---------------------------------------------------------------------
// Spawn portal สำหรับกลับ Floating Island 
// ---------------------------------------------------------------------
pub fn spawn_back_to_floating_island_portal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // วางไว้เลยทางเดินไม้ด้าน +z ออกไปเล็กน้อย
    let portal_position = Vec3::new(0.0, 1.5, WALKWAY_HALF_SPAN + 5.0);

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
        WarpToDesert,
        CurrentScene,
    ));
}

// ---------------------------------------------------------------------
// ตรวจว่าผู้เล่นเดินเข้าใกล้ประตูกลับ Floating Island หรือยัง
// ---------------------------------------------------------------------
pub fn check_warp_back_to_floating_island(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToFloatingIsland>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);

        if distance < WARP_DISTANCE {
            next_state.set(GameScene::LoadingFloatingIsland);
        }
    }
}

// ---------------------------------------------------------------------
// Helper: spawn กล่อง
// ---------------------------------------------------------------------
fn spawn_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    size: Vec3,
    translation: Vec3,
    solid: bool,
) {
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
    let transform = Transform::from_translation(translation);

    if solid {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
            CurrentScene,
            ApplyToonMaterial,
        ));
    } else {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            CurrentScene,
            ApplyToonMaterial,
        ));
    }
}

// ---------------------------------------------------------------------
// Helper: spawn กรวย
// ---------------------------------------------------------------------
fn spawn_cone(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    radius: f32,
    height: f32,
    translation: Vec3,
    rotation: Quat,
    solid: bool,
) {
    let mesh = meshes.add(Cone {
        radius,
        height,
    });

    let transform = Transform::from_translation(translation).with_rotation(rotation);

    if solid {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            RigidBody::Static,

            // ใช้ cylinder เป็น collider กันเดินทะลุภูเขา
            // ถ้าต้องการให้ตรงกับรูปกรวยมากกว่านี้ สามารถเปลี่ยนเป็น
            // Collider::cone(height, radius) ได้ หาก Avian เวอร์ชันของคุณรองรับ
            Collider::cylinder(height, radius),
            CurrentScene,
        ));
    } else {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            CurrentScene,
        ));
    }
}

// ---------------------------------------------------------------------
// ตัวสุ่มแบบง่ายสำหรับภูเขา
// ---------------------------------------------------------------------
fn pseudo_random(seed: usize, salt: f32) -> f32 {
    let value =
        ((seed as f32 + 1.0) * 12.9898 + salt * 78.233).sin() * 43758.5453;

    value.fract().abs()
}