use bevy::prelude::*;
use rust_royale_core::arena::TileType;
use rust_royale_core::components::{
    AoEPayload, AttackStats, AttackTimer, DeathSpawn, DeathSpawnEvent, DeployTimer, Health,
    HealthValueText, MatchPhase, MatchState, MaxHealth, PhysicalBody, PlayerDeck, Position,
    SplashProfile, SpawnLane, SpawnRequest, SpellStrike, Target, TargetingProfile, Team,
    TowerFootprint, TowerStatus, TowerType, Velocity, WaypointPath,
};
use rust_royale_core::constants::TILE_SIZE;
use rust_royale_core::stats::{GlobalStats, SpeedTier, SpellType};

pub fn spawn_entity_system(
    mut commands: Commands,
    mut spawn_requests: EventReader<SpawnRequest>,
    global_stats: Res<GlobalStats>,
    mut match_state: ResMut<MatchState>,
    grid: Res<rust_royale_core::arena::ArenaGrid>,
    mut deck: ResMut<PlayerDeck>,
    towers: Query<(&Team, &TowerType, &TowerFootprint)>,
) {
    if match_state.phase == MatchPhase::GameOver {
        return;
    }

    for request in spawn_requests.read() {
        if let Some(troop_data) = global_stats.0.troops.get(&request.card_key) {
            // --- TERRAIN / BOUNDARY VALIDATION ---
            let fp_x = troop_data.footprint_x as i32;
            let fp_y = troop_data.footprint_y as i32;

            if request.grid_x < 0
                || request.grid_x + fp_x > rust_royale_core::constants::ARENA_WIDTH as i32
                || request.grid_y < 0
                || request.grid_y + fp_y > rust_royale_core::constants::ARENA_HEIGHT as i32
            {
                println!("ERROR: Cannot deploy outside the arena bounds! (footprint {}x{})", fp_x, fp_y);
                continue;
            }

            let mut terrain_valid = true;
            for y in request.grid_y..(request.grid_y + fp_y) {
                for x in request.grid_x..(request.grid_x + fp_x) {
                    let tile_index = (y * rust_royale_core::constants::ARENA_WIDTH as i32 + x) as usize;
                    let tile = &grid.tiles[tile_index];
                    if !matches!(tile, TileType::Grass | TileType::Bridge) {
                        terrain_valid = false;
                        break;
                    }
                }
                if !terrain_valid { break; }
            }

            if !terrain_valid {
                println!("ERROR: Cannot deploy on invalid terrain for footprint {}x{}!", fp_x, fp_y);
                continue;
            }

            // --- DYNAMIC POCKET / DEPLOYMENT ZONE VALIDATION ---
            let divider = rust_royale_core::constants::ARENA_WIDTH as i32 / 2;
            let is_left_lane = request.grid_x < divider;

            let mut red_left_alive = false;
            let mut red_right_alive = false;
            let mut blue_left_alive = false;
            let mut blue_right_alive = false;

            for (t_team, t_type, footprint) in towers.iter() {
                if matches!(t_type, TowerType::Princess) {
                    if *t_team == Team::Red {
                        if footprint.start_x < divider as usize {
                            red_left_alive = true;
                        } else {
                            red_right_alive = true;
                        }
                    } else if *t_team == Team::Blue {
                        if footprint.start_x < divider as usize {
                            blue_left_alive = true;
                        } else {
                            blue_right_alive = true;
                        }
                    }
                }
            }

            let (min_y, max_y) = match request.team {
                Team::Blue => {
                    let max_y = if is_left_lane {
                        if red_left_alive { 14 } else { 20 }
                    } else {
                        if red_right_alive { 14 } else { 20 }
                    };
                    (0, max_y)
                }
                Team::Red => {
                    let min_y = if is_left_lane {
                        if blue_left_alive { 17 } else { 11 }
                    } else {
                        if blue_right_alive { 17 } else { 11 }
                    };
                    (min_y, 31)
                }
            };

            if request.grid_y < min_y || request.grid_y + fp_y - 1 > max_y {
                println!(
                    "ERROR: Cannot deploy at Y={}! Zone restricted for Team {:?} (needs {}, Limits: [{}, {}])",
                    request.grid_y, request.team, fp_y, min_y, max_y
                );
                continue;
            }

            let cost = troop_data.elixir_cost as f32;

            let (current_elixir, team_name) = match request.team {
                Team::Blue => (match_state.blue_elixir, "Blue"),
                Team::Red => (match_state.red_elixir, "Red"),
            };

            if current_elixir < cost {
                println!(
                    "ERROR: {} Team needs {} Elixir, but only has {:.1}",
                    team_name, cost, current_elixir
                );
                continue;
            }

            if request.team == Team::Blue {
                match_state.blue_elixir -= cost;
            } else {
                match_state.red_elixir -= cost;
            }

            // --- DECK ROTATION LOGIC (per-team) ---
            let selected_idx = match request.team {
                Team::Blue => deck.blue_selected,
                Team::Red => deck.red_selected,
            };
            if let Some(sel_idx) = selected_idx {
                let team_deck = match request.team {
                    Team::Blue => &mut deck.blue,
                    Team::Red => &mut deck.red,
                };
                if let Some(played_card) = team_deck.hand[sel_idx].take() {
                    team_deck.queue.push(played_card);
                    team_deck.hand[sel_idx] = Some(team_deck.queue.remove(0));
                }
            }
            match request.team {
                Team::Blue => deck.blue_selected = None,
                Team::Red => deck.red_selected = None,
            }

            println!(
                "Spent {} Elixir from {} Team. Remaining: {:.1}",
                cost,
                team_name,
                if request.team == Team::Blue {
                    match_state.blue_elixir
                } else {
                    match_state.red_elixir
                }
            );

            // --- DETERMINE SPAWN LANE from deployment x coordinate ---
            // If deploying at the centre line with multiple units, we split
            // them across both lanes (the classic Clash Royale split).
            let count = troop_data.spawn_count.unwrap_or(1);
            let is_centre_deploy = count > 1
                && (request.grid_x == divider - 1 || request.grid_x == divider);

            let base_lane = if request.grid_x < divider {
                SpawnLane::Left
            } else {
                SpawnLane::Right
            };

            let mut entity_ids = Vec::new();

            for i in 0..count {
                let mut base_x = (request.grid_x * 1000) + (fp_x * 1000) / 2;
                let base_y = (request.grid_y * 1000) + (fp_y * 1000) / 2;

                if is_centre_deploy {
                    base_x = 10_000; // Snap exactly to the arena dividing line
                }

                let offset_x = if count > 1 { ((i as i32 % 2) * 400) - 200 } else { 0 };
                let offset_y = if count > 1 { (i as i32 / 2) * -400 } else { 0 };

                let fixed_x = base_x + offset_x;
                let fixed_y = base_y + offset_y;

                let unit_lane = if is_centre_deploy {
                    if fixed_x < 10_000 {
                        SpawnLane::Left
                    } else {
                        SpawnLane::Right
                    }
                } else {
                    base_lane
                };

                let math_speed = match troop_data.speed {
                    SpeedTier::VerySlow => 600,
                    SpeedTier::Slow => 900,
                    SpeedTier::Medium => 1200,
                    SpeedTier::Fast => 1800,
                    SpeedTier::VeryFast => 2400,
                };

                let collision_radius = (troop_data.footprint_x as i32 * 1000) / 2;

                let mut entity_cmds = commands.spawn((
                    Position { x: fixed_x, y: fixed_y },
                    Velocity(math_speed),
                    Health(troop_data.health),
                    MaxHealth(troop_data.health),
                    request.team,
                    Target(None),
                    PhysicalBody {
                        radius: collision_radius,
                        mass: troop_data.mass,
                    },
                    AttackStats {
                        damage: troop_data.damage,
                        range: troop_data.range,
                        hit_speed_ms: troop_data.hit_speed_ms,
                        first_attack_sec: troop_data.first_attack_sec,
                        projectile_speed: troop_data.projectile_speed.unwrap_or(6000),
                    },
                    AttackTimer(Timer::from_seconds(
                        troop_data.hit_speed_ms as f32 / 1000.0,
                        TimerMode::Repeating,
                    )),
                    DeployTimer(Timer::from_seconds(
                        troop_data.deploy_time_sec,
                        TimerMode::Once,
                    )),
                    TargetingProfile {
                        is_flying: troop_data.is_flying,
                        is_building: false,
                        targets_air: troop_data.targets_air,
                        targets_ground: troop_data.targets_ground,
                        preference: troop_data.target_preference.clone(),
                    },
                    WaypointPath(std::collections::VecDeque::new()),
                    // Per-unit lane — splits when deployed at centre
                    unit_lane,
                    SpriteBundle {
                        sprite: Sprite {
                            color: if request.team == Team::Blue {
                                Color::BLUE
                            } else {
                                Color::RED
                            },
                            custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                            ..default()
                        },
                        ..default()
                    },
                ));

                entity_cmds.with_children(|parent| {
                    parent.spawn((
                        Text2dBundle {
                            text: Text::from_section(
                                troop_data.health.to_string(),
                                TextStyle {
                                    font_size: 15.0,
                                    color: Color::BLACK,
                                    ..default()
                                },
                            ),
                            transform: Transform::from_xyz(0.0, TILE_SIZE * 0.5, 1.0),
                            ..default()
                        },
                        HealthValueText,
                    ));
                });

                if let Some(ds_card) = &troop_data.death_spawn {
                    entity_cmds.insert(DeathSpawn {
                        card_key: ds_card.clone(),
                        count: troop_data.death_spawn_count.unwrap_or(1),
                    });
                }

                // Add splash profile if the troop has splash stats
                if let (Some(radius), Some(ref splash_type)) = (troop_data.splash_radius, &troop_data.splash_type) {
                    entity_cmds.insert(SplashProfile {
                        splash_radius: radius,
                        splash_type: splash_type.clone(),
                    });
                }

                entity_ids.push(entity_cmds.id());
            }

            let lane_label = if is_centre_deploy { "SPLIT" } else { &format!("{:?}", base_lane) };
            println!(
                "SPAWNED: {} {}s (Entities {:?}) at Grid [{}, {}] Lane: {}",
                count, troop_data.name, entity_ids, request.grid_x, request.grid_y, lane_label
            );
        } else if let Some(spell_data) = global_stats.0.spells.get(&request.card_key) {
            if request.grid_x < 0
                || request.grid_x >= rust_royale_core::constants::ARENA_WIDTH as i32
                || request.grid_y < 0
                || request.grid_y >= rust_royale_core::constants::ARENA_HEIGHT as i32
            {
                continue;
            }

            let cost = spell_data.elixir_cost as f32;

            let current_elixir = if request.team == Team::Blue {
                match_state.blue_elixir
            } else {
                match_state.red_elixir
            };
            if current_elixir < cost {
                continue;
            }

            // Deduct elixir
            if request.team == Team::Blue {
                match_state.blue_elixir -= cost;
            } else {
                match_state.red_elixir -= cost;
            }

            // Deck rotation (per-team)
            let selected_idx = match request.team {
                Team::Blue => deck.blue_selected,
                Team::Red => deck.red_selected,
            };
            if let Some(sel_idx) = selected_idx {
                let team_deck = match request.team {
                    Team::Blue => &mut deck.blue,
                    Team::Red => &mut deck.red,
                };
                if let Some(played_card) = team_deck.hand[sel_idx].take() {
                    team_deck.queue.push(played_card);
                    team_deck.hand[sel_idx] = Some(team_deck.queue.remove(0));
                }
            }
            match request.team {
                Team::Blue => deck.blue_selected = None,
                Team::Red => deck.red_selected = None,
            }

            let fixed_x = (request.grid_x * 1000) + 500;
            let fixed_y = (request.grid_y * 1000) + 500;

            // Calculate travel time from King Tower to target
            let king_fixed_x = (rust_royale_core::constants::ARENA_WIDTH as i32 * 1000) / 2;
            let king_fixed_y = if request.team == Team::Blue { 3000 } else { 27000 };
            
            let dist = (((fixed_x - king_fixed_x) as f32).powi(2) + ((fixed_y - king_fixed_y) as f32).powi(2)).sqrt();
            let base_speed = spell_data.projectile_speed.unwrap_or(20000) as f32;
            let travel_time = dist / base_speed;

            match spell_data.spell_type {
                SpellType::Damage => {
                    let fixed_radius = (spell_data.radius * 1000.0) as i32;
                    let dmg = spell_data.damage.unwrap_or(0);
                    let tower_dmg = spell_data.crown_tower_damage.unwrap_or(dmg / 3);
                    let waves = spell_data.waves.unwrap_or(1);
                    let knockback_val = spell_data.knockback_force.unwrap_or(0);

                    commands.spawn((
                        Position { x: fixed_x, y: fixed_y },
                        request.team,
                        SpellStrike,
                        DeployTimer(Timer::from_seconds(travel_time.max(0.1), TimerMode::Once)),
                        AoEPayload {
                            damage: dmg / waves as i32,
                            tower_damage: tower_dmg / waves as i32,
                            radius: fixed_radius,
                            waves_total: waves,
                            waves_remaining: waves,
                            knockback: knockback_val,
                        },
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::ORANGE_RED,
                                custom_size: Some(Vec2::splat(TILE_SIZE * 0.5)),
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 0.0, 3.0),
                            ..default()
                        },
                    ));

                    println!(
                        "☄️ SPAWNED: {} Spell (Damage) at Grid [{}, {}]!",
                        spell_data.name, request.grid_x, request.grid_y
                    );
                }
                SpellType::Spawner => {
                    // Goblin Barrel etc. — find the troop to spawn
                    let spawn_count = spell_data.spawn_count.unwrap_or(3);
                    let spawn_troop_id = spell_data.spawns_troop_id.unwrap_or(0);

                    // Find the troop card_key by its ID
                    let troop_entry = global_stats.0.troops.iter()
                        .find(|(_, t)| t.id == spawn_troop_id);

                    if let Some((card_key, _)) = troop_entry {
                        // Spawn a visual "travel" indicator (the barrel) which spawns the troops on impact
                        commands.spawn((
                            Position { x: fixed_x, y: fixed_y },
                            request.team,
                            SpellStrike,
                            DeployTimer(Timer::from_seconds(travel_time.max(0.1), TimerMode::Once)),
                            AoEPayload {
                                damage: 0,
                                tower_damage: 0,
                                radius: (spell_data.radius * 1000.0) as i32,
                                waves_total: 1,
                                waves_remaining: 1,
                                knockback: 0,
                            },
                            DeathSpawn {
                                card_key: card_key.clone(),
                                count: spawn_count,
                            },
                            SpriteBundle {
                                sprite: Sprite {
                                    color: Color::DARK_GREEN,
                                    custom_size: Some(Vec2::splat(TILE_SIZE * 0.5)),
                                    ..default()
                                },
                                transform: Transform::from_xyz(0.0, 0.0, 3.0),
                                ..default()
                            },
                        ));

                        println!(
                            "💣 SPAWNED: {} Spell → {} {}s at Grid [{}, {}]!",
                            spell_data.name, spawn_count, card_key, request.grid_x, request.grid_y
                        );
                    } else {
                        println!(
                            "ERROR: Spawner spell '{}' references troop ID {} which doesn't exist!",
                            request.card_key, spawn_troop_id
                        );
                    }
                }
            }
        } else {
            println!(
                "ERROR: Card '{}' not found in troops or spells JSON!",
                request.card_key
            );
        }
    }
}

pub fn deployment_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DeployTimer), Without<SpellStrike>>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).remove::<DeployTimer>();
            println!("Entity {:?} finished deploying and woke up!", entity);
        }
    }
}

pub fn spawn_towers_system(mut commands: Commands, global_stats: Res<GlobalStats>) {
    let princess_data = global_stats.0.buildings.get("princess_tower").unwrap();
    let king_data = global_stats.0.buildings.get("king_tower").unwrap();

    let towers = vec![
        ("princess_tower", Team::Blue,  3,  5, princess_data, TowerType::Princess),
        ("princess_tower", Team::Blue, 14,  5, princess_data, TowerType::Princess),
        ("king_tower",     Team::Blue,  8,  1, king_data,     TowerType::King),
        ("princess_tower", Team::Red,   3, 24, princess_data, TowerType::Princess),
        ("princess_tower", Team::Red,  14, 24, princess_data, TowerType::Princess),
        ("king_tower",     Team::Red,   8, 27, king_data,     TowerType::King),
    ];

    for (name, team, start_x, start_y, data, tower_type) in towers {
        let size_x = data.footprint_x as f32;
        let size_y = data.footprint_y as f32;

        let center_float_x = start_x as f32 + (size_x / 2.0);
        let center_float_y = start_y as f32 + (size_y / 2.0);

        let fixed_x = (center_float_x * 1000.0) as i32;
        let fixed_y = (center_float_y * 1000.0) as i32;

        let collision_radius = (data.footprint_x as i32 * 1000) / 2;
        let footprint_size = data.footprint_x;

        let initial_status = match tower_type {
            TowerType::Princess => TowerStatus::Active,
            TowerType::King => TowerStatus::Sleeping,
        };

        let tower_id = commands
            .spawn((
                Position { x: fixed_x, y: fixed_y },
                Health(data.health),
                MaxHealth(data.health),
                team,
                Target(None),
                PhysicalBody {
                    radius: collision_radius,
                    mass: 99_999,
                },
                AttackStats {
                    damage: data.damage,
                    range: data.range_max,
                    hit_speed_ms: data.hit_speed_ms,
                    first_attack_sec: data.first_attack_sec,
                    projectile_speed: data.projectile_speed.unwrap_or(8000),
                },
                AttackTimer(Timer::from_seconds(
                    data.hit_speed_ms as f32 / 1000.0,
                    TimerMode::Repeating,
                )),
                TargetingProfile {
                    is_flying: false,
                    is_building: true,
                    targets_air: true,
                    targets_ground: true,
                    preference: rust_royale_core::stats::TargetPreference::Any,
                },
                tower_type,
                initial_status,
                TowerFootprint {
                    start_x: start_x as usize,
                    start_y: start_y as usize,
                    size: footprint_size,
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: if team == Team::Blue { Color::BLUE } else { Color::RED },
                        custom_size: Some(Vec2::splat(TILE_SIZE * footprint_size as f32)),
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            data.health.to_string(),
                            TextStyle {
                                font_size: 20.0,
                                color: Color::BLACK,
                                ..default()
                            },
                        ),
                        transform: Transform::from_xyz(
                            0.0,
                            TILE_SIZE * footprint_size as f32 * 0.5 + 5.0,
                            1.0,
                        ),
                        ..default()
                    },
                    HealthValueText,
                ));
            })
            .id();

        if let Some(ds_card) = &data.death_spawn {
            commands.entity(tower_id).insert(DeathSpawn {
                card_key: ds_card.clone(),
                count: data.death_spawn_count.unwrap_or(1),
            });
        }

        println!(
            "SPAWNED: {} (Team: {:?}) at Center Grids [{}, {}]!",
            name, team, center_float_x, center_float_y
        );
    }
}

pub fn handle_death_spawns_system(
    mut commands: Commands,
    mut events: EventReader<DeathSpawnEvent>,
    global_stats: Res<GlobalStats>,
) {
    for event in events.read() {
        if let Some(troop_data) = global_stats.0.troops.get(&event.card_key) {
            // Death-spawn children inherit the parent's lane
            let divider = rust_royale_core::constants::ARENA_WIDTH as i32 / 2;
            let parent_lane = if (event.fixed_x / 1000) < divider {
                SpawnLane::Left
            } else {
                SpawnLane::Right
            };

            for i in 0..event.count {
                // If it's exactly the 3rd unit (like the 3rd goblin), put it perfectly in the middle X-axis
                let offset_x = if i == 2 {
                    0
                } else if event.count > 1 {
                    (i as i32 % 2) * 400 - 200
                } else {
                    0
                };
                
                let offset_y = if event.count > 1 { (i as i32 / 2) * -400 } else { 0 };

                let math_speed = match troop_data.speed {
                    SpeedTier::VerySlow => 600,
                    SpeedTier::Slow => 900,
                    SpeedTier::Medium => 1200,
                    SpeedTier::Fast => 1800,
                    SpeedTier::VeryFast => 2400,
                };
                let collision_radius = (troop_data.footprint_x as i32 * 1000) / 2;

                commands
                    .spawn((
                        Position {
                            x: event.fixed_x + offset_x,
                            y: event.fixed_y + offset_y,
                        },
                        Velocity(math_speed),
                        Health(troop_data.health),
                        MaxHealth(troop_data.health),
                        event.team,
                        Target(None),
                        PhysicalBody {
                            radius: collision_radius,
                            mass: troop_data.mass,
                        },
                        AttackStats {
                            damage: troop_data.damage,
                            range: troop_data.range,
                            hit_speed_ms: troop_data.hit_speed_ms,
                            first_attack_sec: troop_data.first_attack_sec,
                            projectile_speed: troop_data.projectile_speed.unwrap_or(6000),
                        },
                        AttackTimer(Timer::from_seconds(
                            troop_data.hit_speed_ms as f32 / 1000.0,
                            TimerMode::Repeating,
                        )),
                        DeployTimer(Timer::from_seconds(troop_data.deploy_time_sec.max(0.1), TimerMode::Once)),
                        TargetingProfile {
                            is_flying: troop_data.is_flying,
                            is_building: false,
                            targets_air: troop_data.targets_air,
                            targets_ground: troop_data.targets_ground,
                            preference: troop_data.target_preference.clone(),
                        },
                        WaypointPath(std::collections::VecDeque::new()),
                        // *** Golemites etc. inherit the Golem's lane ***
                        parent_lane,
                        SpriteBundle {
                            sprite: Sprite {
                                color: if event.team == Team::Blue { Color::BLUE } else { Color::RED },
                                custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                                ..default()
                            },
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text2dBundle {
                                text: Text::from_section(
                                    troop_data.health.to_string(),
                                    TextStyle {
                                        font_size: 15.0,
                                        color: Color::BLACK,
                                        ..default()
                                    },
                                ),
                                transform: Transform::from_xyz(0.0, TILE_SIZE * 0.5, 1.0),
                                ..default()
                            },
                            HealthValueText,
                        ));
                    });
            }
            println!(
                "💀 DEATH SPAWN: {} {}s popped out in {:?} lane!",
                event.count, troop_data.name, parent_lane
            );
        }
    }
}