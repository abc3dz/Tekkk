use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::Local;

use crate::combat::ElementMastery;
use crate::components::{Health, Mana, Player};

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveData {
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

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SaveRequest>()
            .init_resource::<LoadRequest>()

            .add_systems(
                Update,
                (
                    process_save_request,
                    process_load_request,
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

    let save_data = SaveData {
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

pub fn load_game(
    slot: usize,
    player_query: &mut Query<
        (
            &mut Transform,
            &mut Health,
            &mut Mana,
            &mut ElementMastery,
        ),
        With<Player>,
    >,
) {
    let Some(save_data) = read_save_slot(slot) else {
        println!("Load failed: Slot {} is empty.", slot);
        return;
    };

    let Ok((
        mut transform,
        mut health,
        mut mana,
        mut mastery,
    )) = player_query.single_mut()
    else {
        println!("Load failed: Player not found.");
        return;
    };

    // =========================
    // Player Position
    // =========================

    transform.translation = Vec3::new(
        save_data.player_position[0],
        save_data.player_position[1],
        save_data.player_position[2],
    );

    // =========================
    // HP / MP
    // =========================

    health.current = save_data.hp;
    mana.current = save_data.mp;

    // =========================
    // Element EXP
    // =========================

    mastery.water.exp = save_data.element_water_exp as u32;
    mastery.fire.exp = save_data.element_fire_exp as u32;
    mastery.wind.exp = save_data.element_wind_exp as u32;
    mastery.earth.exp = save_data.element_earth_exp as u32;
    mastery.inw.exp = save_data.element_inw_exp as u32;

    println!(
        "Game loaded from Slot {}",
        slot
    );
}

fn process_save_request(
    mut request: ResMut<SaveRequest>,

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
    );
}

fn process_load_request(
    mut request: ResMut<LoadRequest>,

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
    let Some(slot) = request.slot.take()
    else {
        return;
    };

    load_game(
        slot,
        &mut player_query,
    );
}