use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
use bevy::scene::SceneInstanceReady;
use crate::components::*;

const TOON_SHADER_PATH: &str = "shaders/toon.wgsl";

type ToonMaterial = ExtendedMaterial<StandardMaterial, ToonExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct ToonExtension {
    // ต่ำกว่าค่านี้เป็นเงามืด
    #[uniform(100)]
    pub shadow_cutoff: f32,

    // ต่ำกว่าค่านี้เป็นแสงระดับกลาง
    #[uniform(100)]
    pub mid_cutoff: f32,

    // ความสว่างของเงามืด
    #[uniform(100)]
    pub shadow_brightness: f32,

    // ความสว่างของแสงระดับกลาง
    #[uniform(100)]
    pub mid_brightness: f32,
}

impl Default for ToonExtension {
    fn default() -> Self {
        Self {
            shadow_cutoff: 0.45,
            mid_cutoff: 0.85,
            shadow_brightness: 0.25,
            mid_brightness: 0.65,
        }
    }
}

#[derive(Component)]
pub struct ApplyToonMaterial;

impl MaterialExtension for ToonExtension {
    fn fragment_shader() -> ShaderRef {
        TOON_SHADER_PATH.into()
    }
}
pub struct CelShaderPlugin;
impl Plugin for CelShaderPlugin {
    fn build(&self, app: &mut App) {app
        .add_plugins(MaterialPlugin::<ToonMaterial>::default())
        //.add_systems(Startup, test_spawn)
        .add_observer(apply_toon_material);
    }
}

pub fn test_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            HubOnly,
            Npc,
            GuardianNpc,
            Transform {
                translation: Vec3::new(-5.0, 1.25, -6.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("npc/Guardian.glb"))),
                ApplyToonMaterial,
                Transform::from_xyz(0.0, -1.25, 0.0),
            ));
        });
}

fn apply_toon_material(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    toon_scenes: Query<(), With<ApplyToonMaterial>>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    let scene_root = scene_ready.entity;

    // ทำเฉพาะ SceneRoot ที่มี marker
    if toon_scenes.get(scene_root).is_err() {
        return;
    }

    for descendant in children.iter_descendants(scene_root) {
        let Ok(old_handle) = mesh_materials.get(descendant)
        else {
            continue;
        };

        let Some(old_material) = standard_materials.get(old_handle.id())
        else {
            continue;
        };

        // Clone เพื่อรักษา texture เดิมจาก GLB
        let mut base_material = old_material.clone();

        base_material.perceptual_roughness = 1.0;
        base_material.metallic = 0.0;
        base_material.reflectance = 0.0;

        let toon_handle =
            toon_materials.add(ExtendedMaterial {
                base: base_material,
                extension: ToonExtension::default(),
            });

        commands
            .entity(descendant)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(toon_handle));
    }
}