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
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("musical/ampTurntable.glb"),),),
        RigidBody::Dynamic,        
        transform,
    ))
    .with_children(|parent| {
        parent.spawn((
            Name::new("MusicalBassCollider"),
            ColliderDensity(30.0),
            Collider::cuboid(0.35, 0.3, 0.25),
            Transform::from_translation(Vec3::new(0.0, 0.2, 0.0)),
        ));
    });
}