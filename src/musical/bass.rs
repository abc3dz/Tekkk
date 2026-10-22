use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::components::*;
use crate::cel_shader::*;

use super::{MusicalTexture, MUSICAL_TEXTURE_PATH};

#[derive(Component)]
pub struct Bass;

/// spawn bass แบบมีตำแหน่งเริ่มต้น
pub fn spawn_bass(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    spawn_bass_at(
        &mut commands,
        &asset_server,
        Transform::from_xyz(-5.2, 0.0, -9.5),
    );
}

/// spawn bass ตามตำแหน่งที่กำหนดเอง
pub fn spawn_bass_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    commands.spawn((
        Name::new("MusicalBass"),
        Bass,
        CurrentScene,
        DespawnOnExit(GameScene::Hub),
        ApplyToonMaterial,
        MusicalTexture(asset_server.load(MUSICAL_TEXTURE_PATH)),
        SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("musical/musical_bass.glb"),
            ),
        ),
        RigidBody::Static,
        // ขนาด collider เป็นค่าประมาณ ควรปรับให้พอดีกับโมเดลจริง
        Collider::cuboid(0.7, 1.6, 0.35),
        transform,
    ));
}