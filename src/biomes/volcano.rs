use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::*;

// ---------------------------------------------------------------------
// ค่าคงที่ของ Volcano
// ---------------------------------------------------------------------
const FLOOR_SIZE: f32 = 50.0;
const FLOOR_HEIGHT: f32 = 0.2;

const WARP_DISTANCE: f32 = 2.5;

// ค่าเดียวกับ lagoon.rs เพื่อให้ประตูอยู่ในแผ่นที่ 3 ทาง -z
const LAGOON_TILE_SIZE: f32 = 10.0;
const LAGOON_GAP: f32 = 1.2;
const LAGOON_STEP: f32 = LAGOON_TILE_SIZE + LAGOON_GAP;
const VOLCANO_PORTAL_TILE_INDEX: f32 = 3.0;

// จำนวนภูเขาสุ่ม
const MIN_MOUNTAINS: usize = 10;
const MAX_MOUNTAINS: usize = 18;
const MAX_SPAWN_ATTEMPTS: usize = 1200;

// ---------------------------------------------------------------------
// ตัวสุ่มแบบง่าย ไม่พึ่งพา crate เพิ่ม
// ---------------------------------------------------------------------
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let state = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x1234_5678_9abc_def0);

        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);

        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn gen_f32(&mut self, min: f32, max: f32) -> f32 {
        let value = (self.next_u64() >> 40) as f32 / ((1u64 << 24) - 1) as f32;
        min + value * (max - min)
    }

    fn gen_usize(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }

        let range = (max - min + 1) as u64;
        min + (self.next_u64() % range) as usize
    }
}

// ---------------------------------------------------------------------
// ประตูจาก Lagoon -> Volcano
// ---------------------------------------------------------------------
pub fn spawn_portal_to_volcano_in_lagoon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Lagoon แผ่นที่ 3 ทาง -z
    // แผ่นที่ 1: -11.2
    // แผ่นที่ 2: -22.4
    // แผ่นที่ 3: -33.6
    let portal_position = Vec3::new(
        0.0,
        1.5,
        -LAGOON_STEP * VOLCANO_PORTAL_TILE_INDEX,
    );

    let mesh = meshes.add(Cuboid::new(2.0, 3.0, 0.4));

    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.35, 0.10, 0.80),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.2,
        metallic: 0.1,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(portal_position),
        WarpToVolcano,
        CurrentScene,
    ));
}

pub fn check_warp_to_volcano(
    player_query: Query<&Transform, With<Player>>,
    warp_query: Query<&Transform, With<WarpToVolcano>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };

    for warp_tf in &warp_query {
        let distance = player_tf.translation.distance(warp_tf.translation);

        if distance < WARP_DISTANCE {
            next_state.set(GameScene::LoadingVolcano);
        }
    }
}

pub fn go_to_volcano(
    mut next_state: ResMut<NextState<GameScene>>,
    mut scene_spawn: ResMut<SceneSpawnPoint>,
) {
    scene_spawn.position = Some(Vec3::new(0.0, 2.5, 0.0));
    next_state.set(GameScene::Volcano);

    println!("Entering Volcano");
}

pub fn spawn_volcano(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // พื้น 50 x 50
    let floor_mesh = meshes.add(Plane3d::new(
        Vec3::Y,
        Vec2::splat(FLOOR_SIZE * 0.5),
    ));

    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.10, 0.08),
        perceptual_roughness: 0.7,
        metallic: 0.05,
        ..default()
    });

    spawn_floor(
        &mut commands,
        floor_mesh,
        floor_material,
    );

    // วัสดุภูเขา
    let mountain_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.15, 0.12),
        perceptual_roughness: 0.8,
        metallic: 0.05,
        ..default()
    });

    let mut rng = SimpleRng::new();

    let target_count = rng.gen_usize(MIN_MOUNTAINS, MAX_MOUNTAINS);
    let mut placed: Vec<(Vec2, f32)> = Vec::new();

    let half_floor = FLOOR_SIZE * 0.5;

    // จุดเกิดผู้เล่น
    let spawn_point = Vec2::ZERO;

    // ประตูกลับ Lagoon จะวางไว้ที่ z = 10
    let back_portal_point = Vec2::new(0.0, 10.0);

    let mut attempts = 0;

    while placed.len() < target_count && attempts < MAX_SPAWN_ATTEMPTS {
        attempts += 1;

        let radius = rng.gen_f32(2.0, 6.5);
        let x = rng.gen_f32(-half_floor, half_floor);
        let z = rng.gen_f32(-half_floor, half_floor);

        let position = Vec2::new(x, z);

        // อย่าให้อยู่ชิดขอบพื้นเกินไป
        if position.x.abs() + radius > half_floor - 1.0 {
            continue;
        }

        if position.y.abs() + radius > half_floor - 1.0 {
            continue;
        }

        // อย่าให้บังจุดเกิด
        if position.distance(spawn_point) < radius + 6.0 {
            continue;
        }

        // อย่าให้บังประตูกลับ Lagoon
        if position.distance(back_portal_point) < radius + 4.0 {
            continue;
        }

        // อย่าให้ภูเขาทับกันเกินไป
        let overlaps = placed.iter().any(|(p, r): &(Vec2, f32)| {
            p.distance(position) < (radius + *r) * 0.85
        });

        if overlaps {
            continue;
        }

        placed.push((position, radius));

        let height = rng.gen_f32(radius * 1.7, radius * 3.0);

        spawn_mountain(
            &mut commands,
            &mut meshes,
            mountain_material.clone(),
            position.x,
            position.y,
            radius,
            height,
        );
    }
}

fn spawn_floor(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) {
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::ZERO),

        // ทำให้พื้นแข็งและเหยียบได้
        RigidBody::Static,
        Collider::cuboid(FLOOR_SIZE, FLOOR_HEIGHT, FLOOR_SIZE),

        CurrentScene,
    ));
}

fn spawn_mountain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    x: f32,
    z: f32,
    radius: f32,
    height: f32,
) {
    let mesh = meshes.add(Cone {
        radius,
        height,
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),

        // Cone โดยปกติจะอยู่กึ่งกลางตามแกน Y
        // จึงต้องยกขึ้นครึ่งหนึ่งของความสูงเพื่อให้ฐานแตะพื้น
        Transform::from_xyz(x, height * 0.5, z),

        // ทำให้ภูเขาแข็ง
        // RigidBody::Static,
        // Collider::cone(height, radius),

        CurrentScene,
    ));
}

pub fn setup_volcano_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
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

pub fn spawn_back_to_lagoon_portal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let portal_position = Vec3::new(0.0, 1.5, 10.0);

    let mesh = meshes.add(Cuboid::new(2.0, 3.0, 0.4));

    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.15, 0.85, 1.0, 0.80),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.2,
        metallic: 0.1,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(portal_position),
        WarpToLagoon,
        CurrentScene,
    ));
}

pub fn check_warp_back_to_lagoon(
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