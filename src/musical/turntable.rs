use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::cel_shader::{ApplyToonMaterial, TextureOverride};
use crate::components::CurrentScene;
use super::MUSICAL_TEXTURE_PATH;

#[derive(Component)]
pub struct Turntable;

const TURNTABLE_GLB: &str = "musical/musical_turntable.glb";
const TURNTABLE_TABLE_GLB: &str = "musical/musical_turntableTable.glb";
const AMP_TURNTABLE_GLB: &str = "musical/ampTurntable.glb";

pub fn spawn_turntable_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    let turntable_scene = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset(TURNTABLE_GLB),
    );

    let table_scene = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset(TURNTABLE_TABLE_GLB),
    );

    let amp_scene = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset(AMP_TURNTABLE_GLB),
    );

    let musical_texture = asset_server.load(MUSICAL_TEXTURE_PATH);

    commands
        .spawn((
            Name::new("MusicalTurntable"),
            Turntable,
            CurrentScene,
            ApplyToonMaterial,
            TextureOverride(musical_texture.clone()),
            SceneRoot(turntable_scene),
            RigidBody::Static,
            Collider::cuboid(0.7, 1.0, 0.35),
            transform,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("MusicalTurntableTable"),
                ApplyToonMaterial,
                TextureOverride(musical_texture.clone()),
                SceneRoot(table_scene),
                Collider::cuboid(1.0, 1.6, 0.6),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            parent.spawn((
                Name::new("AmpTurntable"),
                ApplyToonMaterial,
                TextureOverride(musical_texture.clone()),
                SceneRoot(amp_scene),
                Collider::cuboid(0.35, 0.5, 0.25),
                Transform::from_xyz(0.7, 0.0, 0.0),
            ));
        });
}