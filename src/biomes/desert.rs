use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;

pub struct DesertPlugin;

impl Plugin for DesertPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_desert);
    }
}

fn spawn_desert(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        WorldAssetRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("maps/EvrmDesert.glb")
            )
        ),
    //Transform::default(),
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(100.0, 0.1, 100.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}