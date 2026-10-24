use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::cel_shader::{ApplyToonMaterial, TextureOverride};
use crate::components::CurrentScene;
use super::MUSICAL_TEXTURE_PATH;

#[derive(Component)]
pub struct Bass;

const BASS_GLB: &str = "musical/musical_bass.glb";
const AMP_BASS_GLB: &str = "musical/ampBass.glb";

pub fn spawn_bass_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    let bass_scene = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset(BASS_GLB),
    );

    let amp_bass_scene = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset(AMP_BASS_GLB),
    );

    let musical_texture = asset_server.load(MUSICAL_TEXTURE_PATH);

    commands
        .spawn((
            Name::new("MusicalBass"),
            Bass,
            CurrentScene,
            ApplyToonMaterial,
            TextureOverride(musical_texture.clone()),
            SceneRoot(bass_scene),
            RigidBody::Static,
            Collider::cuboid(0.7, 1.6, 0.35),
            transform,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("AmpBass"),
                ApplyToonMaterial,
                TextureOverride(musical_texture.clone()),
                SceneRoot(amp_bass_scene),
                Collider::cuboid(0.35, 0.5, 0.25),
                Transform::from_xyz(0.8, 0.0, 0.0),
            ));
        });
}