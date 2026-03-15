use bevy::prelude::*;
use rust_royale_core::arena::{ArenaGrid, TileType};
use rust_royale_core::components::{
    ElixirUIText, Health, HealthValueText, MatchState, PlayerDeck, Position, Team, TowerFootprint,
    TowerType,
};
use rust_royale_core::constants::{ARENA_HEIGHT, ARENA_WIDTH, TILE_SIZE};

/// Uses Bevy's Gizmos to draw the 18x32 wireframe matrix
pub fn draw_debug_grid(
    mut gizmos: Gizmos,
    grid: Res<ArenaGrid>,
    towers: Query<(&Team, &TowerType, &TowerFootprint)>,
) {
    let total_width = ARENA_WIDTH as f32 * TILE_SIZE;
    let total_height = ARENA_HEIGHT as f32 * TILE_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;

    // --- 1. SCAN THE ALIVE TOWERS TO FIND POCKETS ---
    let mut red_left_alive = false;
    let mut red_right_alive = false;

    // Half of ARENA_WIDTH is the lane divider
    let divider = ARENA_WIDTH / 2;

    for (t_team, t_type, footprint) in towers.iter() {
        if matches!(t_type, TowerType::Princess) && *t_team == Team::Red {
            if footprint.start_x < divider {
                red_left_alive = true;
            } else {
                red_right_alive = true;
            }
        }
    }

    // Determine the max Y coordinate Blue is allowed to deploy in
    // Standard limit is 14 (base of river). If tower is gone, limit is 20 (4 tiles deep).
    let blue_max_y_left = if red_left_alive { 14 } else { 20 };
    let blue_max_y_right = if red_right_alive { 14 } else { 20 };

    // --- 2. DRAW THE TINTED GRID ---
    for y in 0..ARENA_HEIGHT {
        for x in 0..ARENA_WIDTH {
            let tile = &grid.tiles[y * ARENA_WIDTH + x];

            let pos = Vec2::new(
                start_x + (x as f32 * TILE_SIZE) + (TILE_SIZE / 2.0),
                start_y + (y as f32 * TILE_SIZE) + (TILE_SIZE / 2.0),
            );

            // Is this tile in a valid Blue deployment zone?
            let is_left_lane = x < divider;
            let is_valid_depth = if is_left_lane {
                y as i32 <= blue_max_y_left
            } else {
                y as i32 <= blue_max_y_right
            };

            let color = match tile {
                TileType::River => Color::rgb(0.0, 0.4, 0.8),
                TileType::Bridge => Color::rgb(0.5, 0.3, 0.1),
                TileType::Grass => {
                    if is_valid_depth {
                        Color::rgb(0.2, 0.7, 0.2) // Brighter Green (Valid)
                    } else {
                        Color::rgb(0.1, 0.3, 0.1) // Dark Green (Invalid)
                    }
                }
                TileType::Tower => Color::rgb(0.6, 0.6, 0.2),
                TileType::Wall => Color::rgb(0.3, 0.3, 0.3),
            };

            // Draw a slightly smaller rect to see the grid lines
            gizmos.rect_2d(pos, 0.0, Vec2::splat(TILE_SIZE * 0.95), color);
        }
    }
}

pub fn sync_visuals_system(
    mut query: Query<(&Position, &mut Transform), (With<Sprite>, Without<HealthValueText>)>,
) {
    let total_width = ARENA_WIDTH as f32 * TILE_SIZE;
    let total_height = ARENA_HEIGHT as f32 * TILE_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;

    for (pos, mut transform) in query.iter_mut() {
        let float_x = pos.x as f32 / 1000.0;
        let float_y = pos.y as f32 / 1000.0;
        transform.translation.x = start_x + (float_x * TILE_SIZE);
        transform.translation.y = start_y + (float_y * TILE_SIZE);
    }
}

pub fn update_health_text_system(
    parent_query: Query<&Health, Changed<Health>>,
    mut text_query: Query<(&Parent, &mut Text), With<HealthValueText>>,
) {
    for (parent, mut text) in text_query.iter_mut() {
        let parent_entity = parent.get();
        if let Ok(health) = parent_query.get(parent_entity) {
            text.sections[0].value = health.0.to_string();
        }
    }
}

pub fn setup_ui(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Loading HUD...\n",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "Blue Hand\n",
                TextStyle {
                    font_size: 24.0,
                    color: Color::CYAN,
                    ..default()
                },
            ),
            TextSection::new(
                "Red Hand",
                TextStyle {
                    font_size: 24.0,
                    color: Color::TOMATO,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        ElixirUIText,
    ));
}

pub fn update_elixir_ui(
    match_state: Res<MatchState>,
    deck: Res<PlayerDeck>,
    mut query: Query<&mut Text, With<ElixirUIText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        let minutes = (match_state.clock_seconds / 60.0) as u32;
        let seconds = (match_state.clock_seconds % 60.0) as u32;

        let selected_idx = deck.selected_index;
        let selected_text = selected_idx
            .map(|i| format!("{}", i + 1))
            .unwrap_or_else(|| "None".to_string());

        // --- SECTION 0: STATUS HUD (White) ---
        text.sections[0].value = format!(
            "⏱ {}:{:02} | 👑 {}-{} | Selected Slot: {}\n",
            minutes, seconds, match_state.blue_crowns, match_state.red_crowns, selected_text
        );

        // --- SECTION 1: BLUE TEAM (Cyan) ---
        let mut blue_str = format!("💧 Blue {:.1}: ", match_state.blue_elixir);
        for i in 0..4 {
            let card = deck.blue.hand[i].as_deref().unwrap_or("---");
            if selected_idx == Some(i) {
                blue_str += &format!("[{}]{}* ", i + 1, card.to_uppercase());
            } else {
                blue_str += &format!("[{}]{} ", i + 1, card);
            }
        }
        text.sections[1].value = blue_str + "\n";

        // --- SECTION 2: RED TEAM (Tomato) ---
        let mut red_str = format!("🔴 Red  {:.1}: ", match_state.red_elixir);
        for i in 0..4 {
            let card = deck.red.hand[i].as_deref().unwrap_or("---");
            if selected_idx == Some(i) {
                red_str += &format!("[{}]{}* ", i + 1, card.to_uppercase());
            } else {
                red_str += &format!("[{}]{} ", i + 1, card);
            }
        }
        text.sections[2].value = red_str;
    }
}
