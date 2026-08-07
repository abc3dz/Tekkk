use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::components::*;

const QUICKSAND_SHADER_PATH: &str = "shaders/quicksand.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct QuicksandMaterial {
    // สีทรายสว่าง
    #[uniform(0)]
    pub color_light: LinearRgba,

    // สีทรายมืด
    #[uniform(0)]
    pub color_dark: LinearRgba,

    // x = time
    // y = speed
    // z = scale
    // w = edge darkness
    #[uniform(0)]
    pub parameters: Vec4,
}

impl Material for QuicksandMaterial {
    fn fragment_shader() -> ShaderRef {
        QUICKSAND_SHADER_PATH.into()
    }
}

#[derive(Component)]
pub struct Quicksand;

pub fn spawn_quicksand(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<QuicksandMaterial>>,
) {
    let quicksand_material = materials.add(QuicksandMaterial {
        color_light: LinearRgba::new(0.82,0.49,0.20,1.0,),
        color_dark: LinearRgba::new(0.34,0.13,0.055,1.0,),
        parameters: Vec4::new(
            0.0,  // เวลา
            0.30, // ความเร็ว
            3.0,  // จำนวนลาย
            0.25, // ความเข้มบริเวณขอบ
        ),
    });

    commands.spawn((
        Quicksand,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 12.0))),
        MeshMaterial3d(quicksand_material),
        Transform::from_xyz(0.0,0.03,-28.0),
        CurrentScene,
    ));
}

pub fn update_quicksand_shader(
    time: Res<Time>,
    query: Query<&MeshMaterial3d<QuicksandMaterial>,With<Quicksand>>,
    mut materials: ResMut<Assets<QuicksandMaterial>>,
) {
    for material_handle in &query {
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        material.parameters.x = time.elapsed_secs();
    }
}