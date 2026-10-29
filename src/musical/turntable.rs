use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::cel_shader::{ApplyToonMaterial, TextureOverride};
use crate::components::CurrentScene;
use super::MUSICAL_TEXTURE_PATH;

#[derive(Component)]
pub struct Turntable;

pub fn spawn_turntable_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    let turntable_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("musical/musical_turntable.glb"),);
    let table_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("musical/musical_turntableTable.glb"),);
    let musical_texture = asset_server.load(MUSICAL_TEXTURE_PATH);
    let turntable_transform = transform.with_translation(transform.translation + Vec3::new(0.0, 0.7, 0.0),);
    // turntable
    commands.spawn((
        Name::new("MusicalTurntable"),
        Turntable,
        CurrentScene,
        ApplyToonMaterial,
        TextureOverride(musical_texture.clone()),
        SceneRoot(turntable_scene),
        RigidBody::Dynamic,
        turntable_transform,
    ))
    .with_children(|parent| {
        parent.spawn((
            Name::new("MusicalTurntableCollider"),
            ColliderDensity(50.0),
            Collider::cuboid(0.7, 0.1, 0.35),
            Transform::from_translation(Vec3::new(0.0, 0.7, 0.0)),
        ));
    });

    // table
    commands.spawn((
        Name::new("MusicalTurntableTable"),
        CurrentScene,
        ApplyToonMaterial,
        TextureOverride(musical_texture),
        SceneRoot(table_scene),
        RigidBody::Dynamic,
        transform,
    ))
    .with_children(|parent| {
        parent.spawn((
            Name::new("MusicalTurntableTableCollider"),
            Collider::cuboid(1.0, 0.7, 0.6),
            ColliderDensity(80.0),
            Transform::from_translation(Vec3::new(0.0, 0.3, 0.0)),
        ));
    });
    
}