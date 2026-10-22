use bevy::prelude::*;

pub mod bass;
pub mod drum;
pub mod guitar;

pub use bass::*;
pub use drum::*;
pub use guitar::*;

/// path ของ texture ที่ใช้กับเครื่องดนตรีทุกชิ้น
/// ถ้าไฟล์อยู่ใน assets/textures/baba_tt.png ให้แก้เป็น "textures/baba_tt.png"
pub const MUSICAL_TEXTURE_PATH: &str = "musical/baba_tt.png";

/// component สำหรับบอกว่า entity นี้ต้องการใช้ texture ของเครื่องดนตรี
#[derive(Component)]
pub struct MusicalTexture(pub Handle<Image>);

/// component สำหรับกันระบบ apply texture ทำซ้ำกับ entity เดิม
#[derive(Component)]
pub struct MusicalTextureApplied;

pub struct MusicalPlugin;

impl Plugin for MusicalPlugin {
    fn build(&self, app: &mut App) {
        // ระบบนี้จะช่วยเปลี่ยน StandardMaterial ของ mesh ลูก ๆ ให้ใช้ baba_tt.png
        // หมายเหตุ: ถ้า cel_shader ของคุณจัดการ texture เองแล้ว สามารถลบระบบนี้ออกได้
        app.add_systems(Update, apply_baba_texture_to_musical_meshes);

        // ถ้าต้องการให้ spawn อัตโนมัติเมื่อเข้า Hub สามารถเปิดใช้บรรทัดด้านล่างได้
        // แต่ต้องแน่ใจว่าไม่ได้ spawn ซ้ำกับใน hub.rs
        //
        // app.add_systems(
        //     OnEnter(crate::components::GameScene::Hub),
        //     (
        //         drum::spawn_drum,
        //         bass::spawn_bass,
        //         guitar::spawn_guitar,
        //     ),
        // );
    }
}

/// ระบบนี้เกิดมาเพื่อ apply texture `baba_tt.png` ให้กับ mesh ที่เป็น `StandardMaterial`
/// ที่อยู่ภายใต้ entity ที่มี `MusicalTexture`
fn apply_baba_texture_to_musical_meshes(
    mut commands: Commands,
    musical_roots: Query<&Children, With<MusicalTexture>>,
    children_query: Query<&Children>,
    mut mesh_material_query: Query<
        (Entity, &mut MeshMaterial3d<StandardMaterial>),
        Without<MusicalTextureApplied>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut baba_material: Local<Option<Handle<StandardMaterial>>>,
) {
    // สร้าง material ที่มี texture baba_tt.png เพียงครั้งเดียว
    let material_handle = baba_material
        .get_or_insert_with(|| {
            let texture: Handle<Image> = asset_server.load(MUSICAL_TEXTURE_PATH);
            materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                ..default()
            })
        })
        .clone();

    for root_children in &musical_roots {
        let mut stack: Vec<Entity> = root_children.iter().collect();

        while let Some(entity) = stack.pop() {
            // ถ้า entity ไหนมี StandardMaterial ให้แทนที่ด้วย material ที่มี baba_tt.png
            if let Ok((entity, mut material)) = mesh_material_query.get_mut(entity) {
                material.0 = material_handle.clone();
                commands.entity(entity).insert(MusicalTextureApplied);
            }

            // เดินต่อไปยัง child ถัดไป
            if let Ok(children) = children_query.get(entity) {
                stack.extend(children.iter());
            }
        }
    }
}