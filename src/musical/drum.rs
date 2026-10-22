use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::components::*;
use crate::cel_shader::*;

use super::{MusicalTexture, MUSICAL_TEXTURE_PATH};

#[derive(Component)]
pub struct Drum;

/// spawn drum แบบมีตำแหน่งเริ่มต้น
pub fn spawn_drum(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    spawn_drum_at(
        &mut commands,
        &asset_server,
        Transform::from_xyz(-7.3, 0.0, -9.5),
    );
}

/// spawn drum ตามตำแหน่งที่กำหนดเอง
pub fn spawn_drum_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    commands.spawn((
        Name::new("MusicalDrum"),
        Drum,
        CurrentScene,
        DespawnOnExit(GameScene::Hub),
        ApplyToonMaterial,
        MusicalTexture(asset_server.load(MUSICAL_TEXTURE_PATH)),
        SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("musical/musical_drum.glb"),
            ),
        ),
        RigidBody::Static,
        Collider::cuboid(2.09, 1.73, 1.51),
        transform,
    ));
}