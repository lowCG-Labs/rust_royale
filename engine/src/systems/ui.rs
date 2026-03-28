use bevy::prelude::*;
use rust_royale_core::arena::{ArenaGrid, TileType};
use rust_royale_core::components::{
    AnnouncementBanner, CardUI, DeckBuilderCardSlot, DeckBuilderState, DeckBuilderUIRoot,
    ElixirUIText, GameOverUIRoot, Health, HealthValueText, MatchPhase, MatchState, MenuUIRoot,
    PathCache, PauseState, PlayerDeck, Position, Team, TowerFootprint, TowerType,
};
use rust_royale_core::constants::{ARENA_HEIGHT, ARENA_WIDTH, TILE_SIZE};
use rust_royale_core::stats::GlobalStats;

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

    let mut red_left_alive = false;
    let mut red_right_alive = false;

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

    let blue_max_y_left  = if red_left_alive  { 14 } else { 20 };
    let blue_max_y_right = if red_right_alive { 14 } else { 20 };

    for y in 0..ARENA_HEIGHT {
        for x in 0..ARENA_WIDTH {
            let tile = &grid.tiles[y * ARENA_WIDTH + x];

            let pos = Vec2::new(
                start_x + (x as f32 * TILE_SIZE) + (TILE_SIZE / 2.0),
                start_y + (y as f32 * TILE_SIZE) + (TILE_SIZE / 2.0),
            );

            let is_left_lane  = x < divider;
            let is_valid_depth = if is_left_lane {
                y as i32 <= blue_max_y_left
            } else {
                y as i32 <= blue_max_y_right
            };

            let color = match tile {
                TileType::River  => Color::rgb(0.0, 0.4, 0.8),
                TileType::Bridge => Color::rgb(0.5, 0.3, 0.1),
                TileType::Grass  => {
                    if is_valid_depth {
                        Color::rgb(0.2, 0.7, 0.2)
                    } else {
                        Color::rgb(0.1, 0.3, 0.1)
                    }
                }
                TileType::Tower => Color::rgb(0.6, 0.6, 0.2),
                TileType::Wall  => Color::rgb(0.3, 0.3, 0.3),
            };

            gizmos.rect_2d(pos, 0.0, Vec2::splat(TILE_SIZE * 0.95), color);
        }
    }
}

pub fn sync_visuals_system(
    // Separate queries for towers and troops so we can set different z values.
    // Towers at z=0, troops at z=1 — ensures troops always render above tower
    // sprites when they're adjacent or overlapping (e.g. knight attacking a tower).
    mut troop_query: Query<
        (&Position, &mut Transform),
        (With<Sprite>, Without<HealthValueText>, Without<TowerType>),
    >,
    mut tower_query: Query<
        (&Position, &mut Transform),
        (With<Sprite>, Without<HealthValueText>, With<TowerType>),
    >,
) {
    let total_width  = ARENA_WIDTH  as f32 * TILE_SIZE;
    let total_height = ARENA_HEIGHT as f32 * TILE_SIZE;
    let start_x = -total_width  / 2.0;
    let start_y = -total_height / 2.0;

    // Towers at z=0
    for (pos, mut transform) in tower_query.iter_mut() {
        let float_x = pos.x as f32 / 1000.0;
        let float_y = pos.y as f32 / 1000.0;
        transform.translation.x = start_x + (float_x * TILE_SIZE);
        transform.translation.y = start_y + (float_y * TILE_SIZE);
        transform.translation.z = 0.0;
    }

    // Troops at z=1 — always on top of towers
    for (pos, mut transform) in troop_query.iter_mut() {
        let float_x = pos.x as f32 / 1000.0;
        let float_y = pos.y as f32 / 1000.0;
        transform.translation.x = start_x + (float_x * TILE_SIZE);
        transform.translation.y = start_y + (float_y * TILE_SIZE);
        transform.translation.z = 1.0;
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

    // Spawn the Card Bar Container for Blue (bottom of screen)
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(120.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(15.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for i in 0..4 {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(80.0),
                                height: Val::Px(100.0),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            border_color: Default::default(),
                            background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                            ..default()
                        },
                        CardUI {
                            slot_index: i,
                            team: Team::Blue,
                        },
                    ))
                    .with_children(|card| {
                        card.spawn(TextBundle::from_section(
                            format!("Card {}", i + 1),
                            TextStyle {
                                font_size: 16.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
            }
        });
}

pub fn update_elixir_ui(
    match_state: Res<MatchState>,
    deck: Res<PlayerDeck>,
    mut query: Query<&mut Text, With<ElixirUIText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        let minutes = (match_state.clock_seconds / 60.0) as u32;
        let seconds  = (match_state.clock_seconds % 60.0) as u32;

        let blue_sel = deck.blue_selected;
        let red_sel = deck.red_selected;
        let selected_text = blue_sel
            .map(|i| format!("{}", i + 1))
            .unwrap_or_else(|| "None".to_string());

        text.sections[0].value = format!(
            "⏱ {}:{:02} | 👑 {}-{} | Selected Slot: {}\n",
            minutes, seconds, match_state.blue_crowns, match_state.red_crowns, selected_text
        );

        let mut blue_str = format!("💧 Blue {:.1}: ", match_state.blue_elixir);
        for i in 0..4 {
            let card = deck.blue.hand[i].as_deref().unwrap_or("---");
            if blue_sel == Some(i) {
                blue_str += &format!("[{}]{}* ", i + 1, card.to_uppercase());
            } else {
                blue_str += &format!("[{}]{} ", i + 1, card);
            }
        }
        text.sections[1].value = blue_str + "\n";

        let mut red_str = format!("🔴 Red  {:.1}: ", match_state.red_elixir);
        for i in 0..4 {
            let card = deck.red.hand[i].as_deref().unwrap_or("---");
            if red_sel == Some(i) {
                red_str += &format!("[{}]{}* ", i + 1, card.to_uppercase());
            } else {
                red_str += &format!("[{}]{} ", i + 1, card);
            }
        }
        text.sections[2].value = red_str;
    }
}

pub fn update_card_bar_system(
    deck: Res<PlayerDeck>,
    mut card_query: Query<(&CardUI, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for (card_ui, mut bg_color, children) in card_query.iter_mut() {
        if card_ui.team == Team::Blue {
            let card_name = deck.blue.hand[card_ui.slot_index]
                .as_deref()
                .unwrap_or("Empty");
            
            // Highlight selected card
            if deck.blue_selected == Some(card_ui.slot_index) {
                *bg_color = Color::rgb(0.5, 0.5, 0.2).into();
            } else {
                *bg_color = Color::rgb(0.2, 0.2, 0.2).into();
            }

            // Update text
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.sections[0].value = card_name.to_string();
                }
            }
        }
    }
}

/// Watches for MatchPhase transitions and spawns dramatic center-screen banners.
pub fn announcement_system(
    mut commands: Commands,
    match_state: Res<MatchState>,
    mut last_phase: Local<Option<MatchPhase>>,
    existing: Query<Entity, With<AnnouncementBanner>>,
) {
    let current = match_state.phase.clone();

    // Skip if phase hasn't changed
    if last_phase.as_ref() == Some(&current) {
        return;
    }
    *last_phase = Some(current.clone());

    // Determine announcement text, color, and duration
    let (message, text_color, duration) = match current {
        MatchPhase::Regular => (
            "⚔️  FIGHT!  ⚔️",
            Color::WHITE,
            2.0,
        ),
        MatchPhase::DoubleElixir => (
            "⚡  DOUBLE ELIXIR  ⚡",
            Color::rgb(1.0, 0.85, 0.0),
            3.0,
        ),
        MatchPhase::Overtime => (
            "🔥  OVERTIME  🔥",
            Color::rgb(1.0, 0.3, 0.3),
            3.5,
        ),
        MatchPhase::GameOver => {
            let (msg, color) = if match_state.blue_crowns > match_state.red_crowns {
                (
                    "🏆  BLUE WINS!  🏆",
                    Color::rgb(0.3, 0.6, 1.0),
                )
            } else if match_state.red_crowns > match_state.blue_crowns {
                (
                    "🏆  RED WINS!  🏆",
                    Color::rgb(1.0, 0.3, 0.3),
                )
            } else {
                (
                    "⚖️  DRAW!  ⚖️",
                    Color::rgb(1.0, 0.84, 0.0),
                )
            };
            (msg, color, 10.0)
        }
    };

    // Remove any existing banners first
    for ent in existing.iter() {
        commands.entity(ent).despawn_recursive();
    }

    // Spawn the announcement banner — full-width dark strip centered at 30% from top
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    top: Val::Percent(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::vertical(Val::Px(18.0)),
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
                z_index: ZIndex::Global(100),
                ..default()
            },
            AnnouncementBanner {
                timer: Timer::from_seconds(duration, TimerMode::Once),
                total_duration: duration,
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                message,
                TextStyle {
                    font_size: 52.0,
                    color: Color::rgba(text_color.r(), text_color.g(), text_color.b(), 0.0),
                    ..default()
                },
            ));
        });
}

/// Fades in / fades out and removes expired announcement banners.
pub fn announcement_cleanup_system(
    mut commands: Commands,
    time: Res<Time>,
    mut banners: Query<(Entity, &mut AnnouncementBanner, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for (entity, mut banner, mut bg_color, children) in banners.iter_mut() {
        banner.timer.tick(time.delta());

        let remaining = banner.timer.remaining_secs();
        let elapsed = banner.timer.elapsed_secs();

        // Fade in during first 0.4s, full opacity in the middle, fade out during last 1.0s
        let alpha = if elapsed < 0.4 {
            elapsed / 0.4
        } else if remaining < 1.0 {
            remaining.max(0.0)
        } else {
            1.0
        };

        // Update background alpha (dark strip)
        *bg_color = Color::rgba(0.05, 0.02, 0.1, 0.85 * alpha).into();

        // Update text alpha
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                for section in text.sections.iter_mut() {
                    let c = section.style.color;
                    section.style.color = Color::rgba(c.r(), c.g(), c.b(), alpha);
                }
            }
        }

        if banner.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// ===========================================================================
// Game-state UI systems
// ===========================================================================

/// Spawns the main menu screen.
pub fn setup_main_menu(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(24.0),
                    ..default()
                },
                background_color: Color::rgb(0.04, 0.02, 0.12).into(),
                z_index: ZIndex::Global(80),
                ..default()
            },
            MenuUIRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "RUST ROYALE",
                TextStyle {
                    font_size: 80.0,
                    color: Color::rgb(1.0, 0.84, 0.0),
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                "\u{2694}\u{fe0f}  A Clash Royale Clone in Rust  \u{2694}\u{fe0f}",
                TextStyle {
                    font_size: 22.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                    ..default()
                },
            ));
            // Spacer
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(40.0),
                    ..default()
                },
                ..default()
            });
            parent.spawn(TextBundle::from_section(
                "Press  SPACE  to Play",
                TextStyle {
                    font_size: 32.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                "Press  Q  to Quit",
                TextStyle {
                    font_size: 18.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 0.35),
                    ..default()
                },
            ));
        });
}

/// Despawns the main menu.
pub fn cleanup_main_menu(
    mut commands: Commands,
    q: Query<Entity, With<MenuUIRoot>>,
) {
    for ent in q.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

/// Spawns the game-over results overlay.
pub fn setup_game_over_overlay(
    mut commands: Commands,
    match_state: Res<MatchState>,
) {
    let (winner_text, winner_color) = if match_state.blue_crowns > match_state.red_crowns {
        ("Blue Team Wins!", Color::rgb(0.3, 0.6, 1.0))
    } else if match_state.red_crowns > match_state.blue_crowns {
        ("Red Team Wins!", Color::rgb(1.0, 0.3, 0.3))
    } else {
        ("It's a Draw!", Color::rgb(1.0, 0.84, 0.0))
    };

    let score = format!(
        "\u{1f451} Blue {}  \u{2014}  {} Red \u{1f451}",
        match_state.blue_crowns, match_state.red_crowns
    );

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.65).into(),
                z_index: ZIndex::Global(50),
                ..default()
            },
            GameOverUIRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                winner_text,
                TextStyle {
                    font_size: 56.0,
                    color: winner_color,
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                score,
                TextStyle {
                    font_size: 32.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            // Spacer
            parent.spawn(NodeBundle {
                style: Style { height: Val::Px(30.0), ..default() },
                ..default()
            });
            parent.spawn(TextBundle::from_section(
                "Press  SPACE  to Play Again",
                TextStyle {
                    font_size: 26.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 0.85),
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                "Press  R  for Main Menu",
                TextStyle {
                    font_size: 20.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 0.45),
                    ..default()
                },
            ));
        });
}

/// Despawns the game-over overlay.
pub fn cleanup_game_over_overlay(
    mut commands: Commands,
    q: Query<Entity, With<GameOverUIRoot>>,
) {
    for ent in q.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

/// Despawns ALL entities except cameras and windows (for fresh restarts).
pub fn cleanup_all_game_entities(
    mut commands: Commands,
    entities: Query<Entity, (Without<Camera>, Without<Window>, Without<Parent>)>,
) {
    for ent in entities.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

/// Resets all game-level resources to their defaults for a fresh match.
pub fn reset_game_state(
    mut match_state: ResMut<MatchState>,
    mut deck: ResMut<PlayerDeck>,
    mut cache: ResMut<PathCache>,
    mut grid: ResMut<ArenaGrid>,
    mut pause_state: ResMut<PauseState>,
    builder: Res<DeckBuilderState>,
) {
    *match_state = MatchState::default();

    // Use the player's chosen deck if available, otherwise fall back to defaults
    if builder.selected.len() == 8 {
        use rust_royale_core::components::Deck;
        deck.blue = Deck::new_from_cards(&builder.selected, 0);
        deck.red = Deck::new_from_cards(&builder.selected, 99999);
        deck.blue_selected = None;
        deck.red_selected = None;
    } else {
        *deck = PlayerDeck::default();
    }

    cache.map.clear();
    *grid = ArenaGrid::new();
    pause_state.0 = false;
}

// ===========================================================================
// Deck Builder
// ===========================================================================

/// Spawns the full deck-builder screen with a clickable grid of all cards.
pub fn setup_deck_builder(
    mut commands: Commands,
    global_stats: Res<GlobalStats>,
    mut builder: ResMut<DeckBuilderState>,
) {
    builder.selected.clear();

    // Collect playable cards (elixir_cost > 0 excludes tokens like golemite/goblin)
    let mut cards: Vec<(String, String, u32, &str)> = Vec::new(); // (key, name, cost, type_tag)

    for (key, troop) in &global_stats.0.troops {
        if troop.elixir_cost > 0 {
            cards.push((key.clone(), troop.name.clone(), troop.elixir_cost, "Troop"));
        }
    }
    for (key, spell) in &global_stats.0.spells {
        cards.push((key.clone(), spell.name.clone(), spell.elixir_cost, "Spell"));
    }
    cards.sort_by_key(|(_, _, cost, _)| *cost);

    // Root container
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(30.0)),
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                background_color: Color::rgb(0.04, 0.02, 0.12).into(),
                z_index: ZIndex::Global(80),
                ..default()
            },
            DeckBuilderUIRoot,
        ))
        .with_children(|root| {
            // Title
            root.spawn(TextBundle::from_section(
                "BUILD YOUR DECK",
                TextStyle {
                    font_size: 48.0,
                    color: Color::rgb(1.0, 0.84, 0.0),
                    ..default()
                },
            ));

            // Subtitle
            root.spawn(TextBundle::from_section(
                "Select 8 cards — click to toggle",
                TextStyle {
                    font_size: 20.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 0.6),
                    ..default()
                },
            ));

            // Card grid (wrapping flex row)
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(90.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(12.0),
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|grid| {
                for (key, name, cost, type_tag) in &cards {
                    let cost_color = match cost {
                        1..=3 => Color::rgb(0.5, 0.85, 0.5),
                        4..=5 => Color::rgb(0.9, 0.75, 0.3),
                        _ => Color::rgb(1.0, 0.4, 0.4),
                    };
                    let type_color = if *type_tag == "Spell" {
                        Color::rgb(0.6, 0.4, 0.9)
                    } else {
                        Color::rgb(0.4, 0.7, 0.9)
                    };

                    grid.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(130.0),
                                height: Val::Px(80.0),
                                border: UiRect::all(Val::Px(2.0)),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(4.0),
                                ..default()
                            },
                            border_color: Color::rgba(1.0, 1.0, 1.0, 0.15).into(),
                            background_color: Color::rgb(0.15, 0.15, 0.2).into(),
                            ..default()
                        },
                        DeckBuilderCardSlot {
                            card_key: key.clone(),
                        },
                    ))
                    .with_children(|card| {
                        // Card name
                        card.spawn(TextBundle::from_section(
                            name.clone(),
                            TextStyle {
                                font_size: 15.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                        // Cost + type row
                        card.spawn(TextBundle::from_sections([
                            TextSection::new(
                                format!("{}\u{1f4a7} ", cost),
                                TextStyle {
                                    font_size: 13.0,
                                    color: cost_color,
                                    ..default()
                                },
                            ),
                            TextSection::new(
                                type_tag.to_string(),
                                TextStyle {
                                    font_size: 12.0,
                                    color: type_color,
                                    ..default()
                                },
                            ),
                        ]));
                    });
                }
            });

            // Bottom status bar
            root.spawn(TextBundle::from_section(
                "0 / 8 selected",
                TextStyle {
                    font_size: 24.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 0.7),
                    ..default()
                },
            ));

            // ESC hint
            root.spawn(TextBundle::from_section(
                "ESC = Back to Menu",
                TextStyle {
                    font_size: 16.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 0.3),
                    ..default()
                },
            ));
        });
}

/// Despawns the deck builder screen.
pub fn cleanup_deck_builder(
    mut commands: Commands,
    q: Query<Entity, With<DeckBuilderUIRoot>>,
) {
    for ent in q.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

/// Live-updates the deck builder: card highlight colors + status text.
pub fn sync_deck_builder_visuals(
    builder: Res<DeckBuilderState>,
    mut cards: Query<(&DeckBuilderCardSlot, &mut BackgroundColor, &mut BorderColor)>,
    mut text_query: Query<&mut Text, Without<DeckBuilderCardSlot>>,
    roots: Query<&Children, With<DeckBuilderUIRoot>>,
) {
    // Update card colors
    for (slot, mut bg, mut border) in cards.iter_mut() {
        let is_selected = builder.selected.contains(&slot.card_key);
        if is_selected {
            *bg = Color::rgb(0.15, 0.45, 0.25).into();
            *border = Color::rgb(0.3, 1.0, 0.4).into();
        } else {
            *bg = Color::rgb(0.15, 0.15, 0.2).into();
            *border = Color::rgba(1.0, 1.0, 1.0, 0.15).into();
        }
    }

    // Update the status text (second-to-last child of root)
    if let Ok(children) = roots.get_single() {
        let count = builder.selected.len();
        // Status text is the second-to-last child (index = len - 2)
        if children.len() >= 2 {
            let status_ent = children[children.len() - 2];
            if let Ok(mut text) = text_query.get_mut(status_ent) {
                text.sections[0].value = if count == 8 {
                    "\u{2705} 8/8 — Press SPACE to Start!".to_string()
                } else {
                    format!("{} / 8 selected", count)
                };
                text.sections[0].style.color = if count == 8 {
                    Color::rgb(0.3, 1.0, 0.4)
                } else {
                    Color::rgba(1.0, 1.0, 1.0, 0.7)
                };
            }
        }
    }
}