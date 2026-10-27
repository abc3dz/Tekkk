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
    commands.spawn((
        Name::new("MusicalGuitar"),
        Guitar,
        CurrentScene,
        ApplyToonMaterial,
        TextureOverride(asset_server.load(MUSICAL_TEXTURE_PATH)),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("musical/musical_guitar.glb"),),),
        RigidBody::Static,
        Collider::cuboid(2.09, 1.73, 1.51),
        transform,
    ));
}