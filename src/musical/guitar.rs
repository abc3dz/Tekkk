use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::cel_shader::{ApplyToonMaterial, TextureOverride};
use crate::components::CurrentScene;
use super::MUSICAL_TEXTURE_PATH;

#[derive(Component)]
pub struct Guitar;

const GUITAR_GLB: &str = "musical/musical_guitar.glb";
const AMP_GUITAR_GLB: &str = "musical/ampGuitar.glb";

pub fn spawn_guitar_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    let guitar_scene = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset(GUITAR_GLB),
    );

    let amp_guitar_scene = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset(AMP_GUITAR_GLB),
    );

    let musical_texture = asset_server.load(MUSICAL_TEXTURE_PATH);

    commands
        .spawn((
            Name::new("MusicalGuitar"),
            Guitar,
            CurrentScene,
            ApplyToonMaterial,
            TextureOverride(musical_texture.clone()),
            SceneRoot(guitar_scene),
            RigidBody::Static,
            Collider::cuboid(0.8, 1.6, 0.35),
            transform,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("AmpGuitar"),
                ApplyToonMaterial,
                TextureOverride(musical_texture.clone()),
                SceneRoot(amp_guitar_scene),
                Collider::cuboid(0.35, 0.5, 0.25),
                Transform::from_xyz(0.8, 0.0, 0.0),
            ));
        });
}