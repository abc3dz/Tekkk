use avian3d::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::components::*;
use crate::cel_shader::*;

use super::{MusicalTexture, MUSICAL_TEXTURE_PATH};

#[derive(Component)]
pub struct Guitar;

/// spawn guitar แบบมีตำแหน่งเริ่มต้น
pub fn spawn_guitar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    spawn_guitar_at(
        &mut commands,
        &asset_server,
        Transform::from_xyz(-9.4, 0.0, -9.5),
    );
}

/// spawn guitar ตามตำแหน่งที่กำหนดเอง
pub fn spawn_guitar_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    transform: Transform,
) {
    commands.spawn((
        Name::new("MusicalGuitar"),
        Guitar,
        CurrentScene,
        DespawnOnExit(GameScene::Hub),
        ApplyToonMaterial,
        MusicalTexture(asset_server.load(MUSICAL_TEXTURE_PATH)),
        SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("musical/musical_guitar.glb"),
            ),
        ),
        RigidBody::Static,
        // ขนาด collider เป็นค่าประมาณ ควรปรับให้พอดีกับโมเดลจริง
        Collider::cuboid(0.8, 1.6, 0.35),
        transform,
    ));
}