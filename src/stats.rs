#![allow(dead_code)]
use bevy::prelude::Resource;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct BuildingStats {
    pub id: u32,
    pub name: String,
    pub health: i32,
    pub damage: i32,
    pub hit_speed_ms: u32,
    pub range: f32,
    pub footprint_x: usize,
    pub footprint_y: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TroopStats {
    pub id: u32,
    pub name: String,
    pub elixir_cost: u32,
    pub health: i32,
    pub damage: i32,
    pub hit_speed_ms: u32,
    pub speed: String,
    pub range: f32,
    pub footprint_x: usize,
    pub footprint_y: usize,
    // Splash Mechanics (Optional)
    pub splash_radius: Option<f32>,  // e.g., 1.5 tiles
    pub splash_type: Option<String>, // "target_centered", "self_centered", or "linear"
    pub pierce_length: Option<f32>,  // Only used if splash_type is "linear" (Bowler)
}

#[derive(Deserialize, Debug, Clone)]
pub struct SpellStats {
    pub id: u32,
    pub name: String,
    pub elixir_cost: u32,
    pub spell_type: String, // e.g., "damage" or "spawner"
    pub radius: f32,        // The continuous float for Area of Effect

    pub damage: Option<i32>,
    pub crown_tower_damage: Option<i32>,
    pub knockback_force: Option<i32>,
    pub spawns_troop_id: Option<u32>,
    pub spawn_count: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameStats {
    pub buildings: HashMap<String, BuildingStats>,
    pub troops: HashMap<String, TroopStats>,
    pub spells: HashMap<String, SpellStats>,
}

// 3. Make it a Bevy Resource so any System can read it
#[derive(Resource)]
pub struct GlobalStats(pub GameStats);
