use bevy::prelude::*;
use std::fs;

use rust_royale_core::arena::ArenaGrid;
use rust_royale_core::components::{AppState, MatchState, PauseState, PlayerDeck};
use rust_royale_core::stats::{GameStats, GlobalStats};
use rust_royale_engine::systems::combat::{
    combat_damage_system, projectile_flight_system, spell_impact_system, targeting_system,
};
use rust_royale_engine::systems::input::{
    deck_builder_interaction_system, game_not_paused, handle_drag_and_drop, mouse_interaction,
    pause_toggle_system, select_card_system, setup_camera, state_transition_system, window_controls,
};
use rust_royale_engine::systems::match_manager::match_manager_system;
use rust_royale_engine::systems::movement::{physics_movement_system, troop_collision_system};
use rust_royale_engine::systems::spawning::{
    deployment_system, handle_death_spawns_system, spawn_entity_system, spawn_towers_system,
};
use rust_royale_engine::systems::ui::{
    announcement_cleanup_system, announcement_system, cleanup_all_game_entities,
    cleanup_deck_builder, cleanup_game_over_overlay, cleanup_main_menu, draw_debug_grid,
    reset_game_state, setup_deck_builder, setup_game_over_overlay, setup_main_menu, setup_ui,
    sync_deck_builder_visuals, sync_visuals_system, update_card_bar_system, update_elixir_ui,
    update_health_text_system,
};

fn main() {
    let stats_file = fs::read_to_string("assets/stats.json")
        .expect("Failed to find assets/stats.json! Make sure the folder exists.");
    let parsed_stats: GameStats = serde_json::from_str(&stats_file).expect("Failed to parse JSON!");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Royale - Official Match".into(),
                mode: bevy::window::WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .insert_resource(ArenaGrid::new())
        .insert_resource(GlobalStats(parsed_stats))
        .insert_resource(MatchState::default())
        .insert_resource(PlayerDeck::default())
        .insert_resource(rust_royale_core::components::DragState::default())
        .init_resource::<rust_royale_core::components::PathCache>()
        .init_resource::<PauseState>()
        .init_resource::<rust_royale_core::components::DeckBuilderState>()
        // Fixed timestep: 60 ticks per second for deterministic simulation
        .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0))
        .add_event::<rust_royale_core::components::SpawnRequest>()
        .add_event::<rust_royale_core::components::DeathSpawnEvent>()
        .add_event::<rust_royale_core::components::TowerDeathEvent>()
        // ---------------------------------------------------------------
        // Startup: camera only
        // ---------------------------------------------------------------
        .add_systems(Startup, setup_camera)
        // ---------------------------------------------------------------
        // State enter / exit hooks
        // ---------------------------------------------------------------
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
        .add_systems(OnEnter(AppState::DeckBuilder), setup_deck_builder)
        .add_systems(OnExit(AppState::DeckBuilder), cleanup_deck_builder)
        .add_systems(
            OnEnter(AppState::Playing),
            (cleanup_all_game_entities, reset_game_state, spawn_towers_system, setup_ui).chain(),
        )
        .add_systems(OnEnter(AppState::GameOver), setup_game_over_overlay)
        .add_systems(OnExit(AppState::GameOver), cleanup_game_over_overlay)
        // ---------------------------------------------------------------
        // Input: always-on (state transitions, window controls, pause)
        // ---------------------------------------------------------------
        .add_systems(
            Update,
            (window_controls, state_transition_system, pause_toggle_system),
        )
        // ---------------------------------------------------------------
        // Deck builder: interaction + visual sync
        // ---------------------------------------------------------------
        .add_systems(
            Update,
            (deck_builder_interaction_system, sync_deck_builder_visuals)
                .run_if(in_state(AppState::DeckBuilder)),
        )
        // ---------------------------------------------------------------
        // Input: gameplay-only
        // ---------------------------------------------------------------
        .add_systems(
            Update,
            (mouse_interaction, select_card_system, handle_drag_and_drop)
                .run_if(in_state(AppState::Playing)),
        )
        // ---------------------------------------------------------------
        // Game logic: FixedUpdate at 60Hz, only while Playing + unpaused
        // ---------------------------------------------------------------
        .add_systems(
            FixedUpdate,
            (
                spawn_entity_system,
                deployment_system,
                match_manager_system,
                targeting_system,
                combat_damage_system,
                projectile_flight_system,
                spell_impact_system,
                physics_movement_system,
                troop_collision_system,
                handle_death_spawns_system,
            )
                .chain()
                .run_if(in_state(AppState::Playing))
                .run_if(game_not_paused),
        )
        // ---------------------------------------------------------------
        // Visual sync: runs during Playing and GameOver
        // ---------------------------------------------------------------
        .add_systems(
            Update,
            (
                draw_debug_grid,
                sync_visuals_system,
                update_card_bar_system,
                update_elixir_ui,
                update_health_text_system,
                announcement_system,
                announcement_cleanup_system,
            )
                .run_if(in_state(AppState::Playing).or_else(in_state(AppState::GameOver))),
        )
        .run();
}
