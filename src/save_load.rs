use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::Local;

use crate::combat::ElementMastery;
use crate::components::{Health, Mana, Player, SaveScene, GameScene};

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveData {
    pub scene: SaveScene,
    pub player_position: [f32; 3],
    pub hp: i32,
    pub mp: i32,
    pub element_water_exp: i32,
    pub element_fire_exp: i32,
    pub element_wind_exp: i32,
    pub element_earth_exp: i32,
    pub element_inw_exp: i32,

    #[serde(default)]
    pub saved_at: String,
}

#[derive(Resource, Default)]
pub struct SaveRequest {
    pub slot: Option<usize>,
}

#[derive(Resource, Default)]
pub struct LoadRequest {
    pub slot: Option<usize>,
}

#[derive(Resource, Default)]
pub struct PendingLoad {
    pub data: Option<SaveData>,
}

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SaveRequest>()
            .init_resource::<LoadRequest>()
            .init_resource::<PendingLoad>()
            .add_systems(
                Update,
                (
                    process_save_request,
                    process_load_request,
                    apply_pending_load,
                ),
            );
    }
}

pub fn save_game(
    slot: usize,
    player_query: Query<
        (
            &Transform,
            &Health,
            &Mana,
            &ElementMastery,
        ),
        With<Player>,
    >,
    game_scene: Res<State<GameScene>>,
) {
    let Ok((
        transform,
        health,
        mana,
        mastery,
    )) = player_query.single()
    else {
        println!("Save failed: Player not found");
        return;
    };

    let scene = match game_scene.get() {
        GameScene::Hub => SaveScene::Hub,
        GameScene::Desert => SaveScene::Desert,
        GameScene::FloatingIsland => SaveScene::FloatingIsland,
        GameScene::Lagoon => SaveScene::Lagoon,
        GameScene::Volcano => SaveScene::Volcano,

        // Loading state ไม่อนุญาตให้ Save
        GameScene::LoadingHub
        | GameScene::LoadingDesert
        | GameScene::LoadingFloatingIsland
        | GameScene::LoadingLagoon
        | GameScene::LoadingVolcano => {
            println!("Cannot save while loading.");
            return;
        }

        // ถ้ามีฉากอื่นก็เพิ่มตรงนี้
    };

    let save_data = SaveData {
        scene,
        player_position: [
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        ],

        hp: health.current,
        mp: mana.current,

        element_water_exp: mastery.water.exp as i32,
        element_fire_exp: mastery.fire.exp as i32,
        element_wind_exp: mastery.wind.exp as i32,
        element_earth_exp: mastery.earth.exp as i32,
        element_inw_exp: mastery.inw.exp as i32,

        saved_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    let json = match serde_json::to_string_pretty(&save_data) {
        Ok(json) => json,

        Err(error) => {
            println!("Save failed: {error}");
            return;
        }
    };

    let save_dir = "save";

    if let Err(error) = std::fs::create_dir_all(save_dir) {
        println!("Could not create save directory: {error}");
        return;
    }
    let filename = format!("save/save{:02}.json", slot);
    match std::fs::write(
        filename,
        json,
    ) {
        Ok(_) => {
            println!("Game saved!");
        }

        Err(error) => {
            println!("Save failed: {error}");
        }
    }
}

pub fn save_directory() -> PathBuf {
    PathBuf::from("save")
}

pub fn save_path(slot: usize) -> PathBuf {
    save_directory()
        .join(format!("save{:02}.json", slot))
}

pub fn read_save_slot(slot: usize) -> Option<SaveData> {
    let path = save_path(slot);

    if !path.exists() {
        return None;
    }

    let json = match fs::read_to_string(&path) {
        Ok(json) => json,

        Err(error) => {
            println!(
                "Failed to read Slot {}: {}",
                slot,
                error
            );

            return None;
        }
    };

    match serde_json::from_str::<SaveData>(&json) {
        Ok(data) => Some(data),

        Err(error) => {
            println!(
                "Failed to parse Slot {}: {}",
                slot,
                error
            );

            None
        }
    }
}

fn process_save_request(
    mut request: ResMut<SaveRequest>,
    game_scene: Res<State<GameScene>>,
    player_query: Query<
        (
            &Transform,
            &Health,
            &Mana,
            &ElementMastery,
        ),
        With<Player>,
    >,
) {
    let Some(slot) = request.slot.take()
    else {
        return;
    };

    save_game(
        slot,
        player_query,
        game_scene,
    );
}

fn process_load_request(
    mut request: ResMut<LoadRequest>,

    mut next_scene: ResMut<NextState<GameScene>>,

    mut pending_load: ResMut<PendingLoad>,
) {
    let Some(slot) = request.slot.take()
    else {
        return;
    };
    let Some(save_data) = read_save_slot(slot) else {
        println!("Load failed: Slot {} is empty.", slot);
        return;
    };
    println!(
        "Loading Slot {} → {:?}",
        slot,
        save_data.scene
    );
    let next_game_scene = match save_data.scene {
        SaveScene::Hub => GameScene::LoadingHub,
        SaveScene::Desert => GameScene::LoadingDesert,
        SaveScene::FloatingIsland => GameScene::LoadingFloatingIsland,
        SaveScene::Lagoon => GameScene::LoadingLagoon,
        SaveScene::Volcano => GameScene::LoadingVolcano,
    };

    next_scene.set(next_game_scene);

    pending_load.data = Some(save_data);
}

fn apply_pending_load(
    mut pending_load: ResMut<PendingLoad>,

    mut player_query: Query<
        (
            &mut Transform,
            &mut Health,
            &mut Mana,
            &mut ElementMastery,
        ),
        With<Player>,
    >,
) {
    let Some(save_data) = pending_load.data.take() else {
        return;
    };

    let Ok((
        mut transform,
        mut health,
        mut mana,
        mut mastery,
    )) = player_query.single_mut()
    else {
        // Player ยังไม่ spawn
        // อย่าเอา pending_load ทิ้ง
        pending_load.data = Some(save_data);
        return;
    };

    transform.translation = Vec3::new(
        save_data.player_position[0],
        save_data.player_position[1],
        save_data.player_position[2],
    );

    health.current = save_data.hp;
    mana.current = save_data.mp;

    mastery.water.exp = save_data.element_water_exp as u32;
    mastery.fire.exp = save_data.element_fire_exp as u32;
    mastery.wind.exp = save_data.element_wind_exp as u32;
    mastery.earth.exp = save_data.element_earth_exp as u32;
    mastery.inw.exp = save_data.element_inw_exp as u32;

    println!("Pending save data applied.");
}