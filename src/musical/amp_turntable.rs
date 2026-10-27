use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::cel_shader::{ApplyToonMaterial, TextureOverride};
use crate::components::CurrentScene;

use super::MUSICAL_TEXTURE_PATH;

#[derive(Component)]
pub struct AmpTurntable;

pub fn spawn_amp_turntable_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    commands.spawn((
        Name::new("AmpTurntable"),
        AmpTurntable,
        CurrentScene,
        ApplyToonMaterial,
        TextureOverride(asset_server.load(MUSICAL_TEXTURE_PATH)),
        SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("musical/ampTurntable.glb"),
            ),
        ),
        RigidBody::Static,
        Collider::cuboid(0.35, 0.5, 0.25),
        transform,
    ));
}