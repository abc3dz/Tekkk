use bevy::prelude::*;
use avian3d::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world);
        app.add_plugins(crate::biomes::desert::DesertPlugin);
    }
}

fn setup_world(
    mut commands: Commands,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // commands.spawn((
    //     Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color: Color::srgb(0.76, 0.62, 0.38), // sand color
    //         perceptual_roughness: 1.0,
    //         ..default()
    //     })),
    //     RigidBody::Static,
    //     Collider::cuboid(100.0, 0.1, 100.0),
    // ));

    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}