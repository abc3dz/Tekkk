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
    commands.spawn((
        Name::new("MusicalBass"),
        Bass,
        CurrentScene,
        ApplyToonMaterial,
        TextureOverride(asset_server.load(MUSICAL_TEXTURE_PATH)),
        SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("musical/musical_bass.glb"),
            ),
        ),
        RigidBody::Static,
        // ขนาด collider ประมาณ ควรปรับตามโมเดลจริง
        Collider::cuboid(0.7, 1.6, 0.35),
        transform,
    ));
}