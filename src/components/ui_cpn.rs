use bevy::prelude::*;

#[derive(Resource)]
pub struct GameFonts {
    pub abc3dz: Handle<Font>,
}

pub fn load_fonts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(GameFonts {
        abc3dz: asset_server.load("fonts/abc3dz.ttf"),
    });
}