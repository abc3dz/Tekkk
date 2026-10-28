use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::cel_shader::{ApplyToonMaterial, TextureOverride};
use crate::components::CurrentScene;

use super::MUSICAL_TEXTURE_PATH;

#[derive(Component)]
pub struct AmpBass;

pub fn spawn_amp_bass_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    commands.spawn((
        Name::new("AmpBass"),
        AmpBass,
        CurrentScene,
        ApplyToonMaterial,
        TextureOverride(asset_server.load(MUSICAL_TEXTURE_PATH)),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("musical/ampBass.glb"),),),
        RigidBody::Dynamic,
        ColliderDensity(30.0),
        transform,
    ))
    .with_children(|parent| {
        parent.spawn((
            Name::new("MusicalBassCollider"),
            Collider::cuboid(0.35, 0.3, 0.25),
            Transform::from_translation(Vec3::new(0.0, 0.07, 0.0)),
        ));
    });
}