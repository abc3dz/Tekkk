use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::cel_shader::{ApplyToonMaterial, TextureOverride};
use crate::components::CurrentScene;

use super::MUSICAL_TEXTURE_PATH;

#[derive(Component)]
pub struct Guitar;

pub fn spawn_guitar_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    commands
        .spawn((
            Name::new("MusicalGuitar"),
            Guitar,
            CurrentScene,
            ApplyToonMaterial,
            TextureOverride(asset_server.load(MUSICAL_TEXTURE_PATH)),
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("musical/musical_guitar.glb")),),
            RigidBody::Dynamic,
            ColliderDensity(70.0),
            transform,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("MusicalGuitarCollider"),
                Collider::cuboid(0.45, 1.4, 0.1),
                Transform::from_translation(Vec3::new(0.0, 0.7, 0.0)),
            ));
        });
}