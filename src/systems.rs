use crate::arena::{ArenaGrid, TileType};
use crate::constants::{ARENA_HEIGHT, ARENA_WIDTH, TILE_SIZE};
use bevy::{app::AppExit, prelude::*};

/// Spawns the 2D camera so we can actually see the world
pub fn setup_camera(mut commands: Commands, mut window_query: Query<&mut Window>) {
    let mut camera = Camera2dBundle::default();

    // Automatically scale the camera so the entire grid (plus a small margin) is ALWAYS visible.
    // This fixes clipping issues for users on smaller laptop screens like MacBooks.
    let min_width = (ARENA_WIDTH as f32 * TILE_SIZE) + 100.0;
    let min_height = (ARENA_HEIGHT as f32 * TILE_SIZE) + 100.0;

    camera.projection.scaling_mode = bevy::render::camera::ScalingMode::AutoMin {
        min_width,
        min_height,
    };

    commands.spawn(camera);

    // Maximize the window on startup
    if let Ok(mut window) = window_query.get_single_mut() {
        window.set_maximized(true);
    }
}

/// Uses Bevy's Gizmos to draw the 18x32 wireframe matrix
pub fn draw_debug_grid(mut gizmos: Gizmos, grid: Res<ArenaGrid>) {
    let total_width = ARENA_WIDTH as f32 * TILE_SIZE;
    let total_height = ARENA_HEIGHT as f32 * TILE_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;

    // Draw the Background Tiles
    for y in 0..ARENA_HEIGHT {
        for x in 0..ARENA_WIDTH {
            let color = match grid.tiles[y * ARENA_WIDTH + x] {
                TileType::Grass => Color::DARK_GREEN,
                TileType::River => Color::BLUE,
                TileType::Bridge => Color::GRAY,
                TileType::Tower => Color::GOLD,
            };

            let pos = Vec2::new(
                start_x + (x as f32 * TILE_SIZE) + (TILE_SIZE / 2.0),
                start_y + (y as f32 * TILE_SIZE) + (TILE_SIZE / 2.0),
            );

            // Draw a slightly smaller rect to see the grid lines
            gizmos.rect_2d(pos, 0.0, Vec2::splat(TILE_SIZE * 0.9), color);
        }
    }
}

pub fn mouse_interaction(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    // 1. Get mouse position in world coordinates
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        // 2. Map continuous world position to grid indices
        // We add the offset to align with the grid drawing math
        let total_width = ARENA_WIDTH as f32 * TILE_SIZE;
        let total_height = ARENA_HEIGHT as f32 * TILE_SIZE;

        let grid_x = ((world_position.x + total_width / 2.0) / TILE_SIZE) as i32;
        let grid_y = ((world_position.y + total_height / 2.0) / TILE_SIZE) as i32;

        // 3. Highlight the tile if inside the 18x32 bounds
        if grid_x >= 0 && grid_x < ARENA_WIDTH as i32 && grid_y >= 0 && grid_y < ARENA_HEIGHT as i32
        {
            let pos = Vec2::new(
                (-total_width / 2.0) + (grid_x as f32 * TILE_SIZE) + (TILE_SIZE / 2.0),
                (-total_height / 2.0) + (grid_y as f32 * TILE_SIZE) + (TILE_SIZE / 2.0),
            );
            gizmos.rect_2d(pos, 0.0, Vec2::splat(TILE_SIZE * 0.9), Color::YELLOW);
        }
    }
}

pub fn window_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut window_query: Query<&mut Window>,
) {
    // Esc to Close
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }

    // Tab to Toggle Fullscreen (so you can minimize manually)
    if let Ok(mut window) = window_query.get_single_mut() {
        if keyboard_input.just_pressed(KeyCode::Tab) {
            window.mode = match window.mode {
                bevy::window::WindowMode::Windowed => bevy::window::WindowMode::Fullscreen,
                _ => bevy::window::WindowMode::Windowed,
            };
        }
    }
}
