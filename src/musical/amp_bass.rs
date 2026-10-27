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
        RigidBody::Static,
        Collider::cuboid(0.35, 0.5, 0.25),
        transform,
    ));
}