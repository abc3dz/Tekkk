use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct CurrentScene;

#[derive(Component)]
pub struct LoadingUI;

#[derive(Component)]
pub struct WarpToDesert;

#[derive(Component)]
pub struct WarpToHub;

#[derive(Component)]
pub struct HubOnly;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameScene {
    #[default]
    LoadingHub,
    Hub,
    LoadingDesert,
    Desert,
    LoadingFloatingIsland,
    FloatingIsland,
    LoadingLagoon,
    Lagoon,
    LoadingVolcano,
    Volcano,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SaveScene {
    Hub,
    Desert,
    FloatingIsland,
    Lagoon,
    Volcano,
}