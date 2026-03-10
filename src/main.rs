mod arena;
mod constants;
mod stats;
mod systems;

use bevy::prelude::*;
use std::fs;

use crate::arena::ArenaGrid;
use crate::stats::{GameStats, GlobalStats};
use crate::systems::{draw_debug_grid, mouse_interaction, setup_camera, window_controls};

fn main() {
    let stats_file = fs::read_to_string("assets/stats.json")
        .expect("Failed to find assets/stats.json! Make sure the folder exists.");

    // Parse the JSON text into our Rust structs
    let parsed_stats: GameStats =
        serde_json::from_str(&stats_file).expect("Failed to parse JSON! Check for typos.");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Royale".into(),
                mode: bevy::window::WindowMode::Windowed,
                ..default()
            }),
            ..default()
        })) // Loads the window, renderer, and core engine
        .add_systems(Startup, setup_camera) // Runs exactly once when the app starts
        .add_systems(Update, draw_debug_grid) // Runs every single frame (60+ FPS)
        .insert_resource(ArenaGrid::new())
        .insert_resource(GlobalStats(parsed_stats))
        .add_systems(Update, mouse_interaction)
        .add_systems(Update, window_controls)
        .run();
}
