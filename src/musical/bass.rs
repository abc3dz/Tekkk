use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::cel_shader::{ApplyToonMaterial, TextureOverride};
use crate::components::CurrentScene;

use super::MUSICAL_TEXTURE_PATH;

#[derive(Component)]
pub struct Bass;

pub fn spawn_bass_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    commands
        .spawn((
            Name::new("MusicalBass"),
            Bass,
            CurrentScene,
            ApplyToonMaterial,
            TextureOverride(asset_server.load(MUSICAL_TEXTURE_PATH)),
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("musical/musical_bass.glb")),),
            RigidBody::Dynamic,            
            transform,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("MusicalBassCollider"),
                Collider::cuboid(0.45, 1.4, 0.1),
                ColliderDensity(100.0),
                Transform::from_translation(Vec3::new(0.0, 0.7, 0.0)),
            ));
        });
}