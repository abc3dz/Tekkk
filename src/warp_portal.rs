use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

const WARP_PORTAL_SHADER: &str = "shaders/warp_portal.wgsl";

pub struct WarpPortalPlugin;

impl Plugin for WarpPortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WarpPortalMaterial>::default());
    }
}

#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)]
pub struct WarpPortalMaterial {}

impl Material for WarpPortalMaterial {
    fn fragment_shader() -> ShaderRef {
        WARP_PORTAL_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Component)]
pub struct WarpPortal;

pub fn spawn_warp_portal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WarpPortalMaterial>>,
) {
    let portal_mesh = meshes.add(Circle::new(2.0));
    let portal_material = materials.add(WarpPortalMaterial {});
    commands.spawn((
        Name::new("Warp Portal"),
        WarpPortal,
        Mesh3d(portal_mesh),
        MeshMaterial3d(portal_material),
        Transform::from_xyz(0.0,2.0,-14.0,),
    ));
}