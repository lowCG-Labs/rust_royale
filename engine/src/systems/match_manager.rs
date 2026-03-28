use bevy::prelude::*;
use rust_royale_core::components::{
    Health, MaxHealth, MatchPhase, MatchState, Team, TowerFootprint, TowerType,
};

// Helper function to handle crowns and phase shifts for a destroyed tower.
pub fn apply_tower_destruction_rules(
    match_state: &mut MatchState,
    team: Team,
    tower_type: TowerType,
) -> (Option<Team>, Option<Team>) {
    let crowns = match tower_type {
        TowerType::Princess => 1,
        TowerType::King => 3,
    };

    let mut king_destroyed_team = None;
    let mut wake_king_team = None;

    if matches!(tower_type, TowerType::King) {
        king_destroyed_team = Some(team);
    } else if matches!(tower_type, TowerType::Princess) {
        wake_king_team = Some(team);
    }

    if team == Team::Red {
        if crowns == 3 {
            match_state.blue_crowns = 3;
        } else {
            match_state.blue_crowns = (match_state.blue_crowns + crowns).min(3);
        }
    } else {
        if crowns == 3 {
            match_state.red_crowns = 3;
        } else {
            match_state.red_crowns = (match_state.red_crowns + crowns).min(3);
        }
    }

    if matches!(tower_type, TowerType::King) || match_state.phase == MatchPhase::Overtime {
        match_state.phase = MatchPhase::GameOver;
    }

    (king_destroyed_team, wake_king_team)
}

pub fn match_manager_system(
    mut commands: Commands,
    time: Res<Time>,
    mut match_state: ResMut<MatchState>,
    mut grid: ResMut<rust_royale_core::arena::ArenaGrid>,
    towers: Query<(Entity, &Health, &MaxHealth, &Team, &TowerType, &TowerFootprint)>,
) {
    if match_state.phase == MatchPhase::GameOver {
        return;
    }

    let delta = time.delta_seconds();
    match_state.clock_seconds -= delta;

    if match_state.phase == MatchPhase::Regular && match_state.clock_seconds <= 60.0 {
        match_state.phase = MatchPhase::DoubleElixir;
        println!("🕒 60 SECONDS LEFT: DOUBLE ELIXIR!");
    } else if match_state.clock_seconds <= 0.0 {
        if match_state.phase == MatchPhase::DoubleElixir {
            if match_state.blue_crowns == match_state.red_crowns {
                match_state.phase = MatchPhase::Overtime;
                match_state.clock_seconds = 60.0; // 1 Minute of Overtime
                println!("⚔️ OVERTIME! SUDDEN DEATH!");
            } else {
                match_state.phase = MatchPhase::GameOver;
                match_state.clock_seconds = 0.0;
                println!(
                    "🛑 MATCH OVER! Final Score: {}-{}",
                    match_state.blue_crowns, match_state.red_crowns
                );
            }
        } else if match_state.phase == MatchPhase::Overtime {
            // --- TIEBREAKER: Compare per-team lowest HP PERCENTAGE ---
            match_state.clock_seconds = 0.0;

            // Find the lowest HP% tower per team
            let mut blue_lowest_pct: f32 = f32::MAX;
            let mut red_lowest_pct: f32 = f32::MAX;
            let mut blue_lowest_ent: Option<Entity> = None;
            let mut red_lowest_ent: Option<Entity> = None;

            for (entity, health, max_health, team, _, _) in towers.iter() {
                // Calculate HP percentage 
                let hp_pct = health.0 as f32 / max_health.0 as f32;
                match team {
                    Team::Blue => {
                        if hp_pct < blue_lowest_pct {
                            blue_lowest_pct = hp_pct;
                            blue_lowest_ent = Some(entity);
                        }
                    }
                    Team::Red => {
                        if hp_pct < red_lowest_pct {
                            red_lowest_pct = hp_pct;
                            red_lowest_ent = Some(entity);
                        }
                    }
                }
            }

            // The team whose lowest-HP tower has LESS absolute HP loses that tower
            let loser_ent = if blue_lowest_pct < red_lowest_pct {
                blue_lowest_ent
            } else if red_lowest_pct < blue_lowest_pct {
                red_lowest_ent
            } else {
                // Exact same HP — it's a draw, destroy both
                None
            };

            if let Some(destroy_ent) = loser_ent {
                if let Ok((entity, health, _, team, tower_type, footprint)) = towers.get(destroy_ent) {
                    commands.entity(entity).despawn_recursive();
                    grid.clear_tower(footprint.start_x, footprint.start_y, footprint.size);

                    let (king_destroyed_team, _) = apply_tower_destruction_rules(
                        &mut match_state,
                        *team,
                        *tower_type,
                    );

                    println!(
                        "💥 TIEBREAKER! {:?} tower with lowest HP ({}) destroyed! Score: {}-{}",
                        team, health.0, match_state.blue_crowns, match_state.red_crowns
                    );

                    if let Some(losing_team) = king_destroyed_team {
                        for (entity, _, _, team, tower_type, footprint) in towers.iter() {
                            if *team == losing_team && matches!(tower_type, TowerType::Princess) {
                                commands.entity(entity).despawn_recursive();
                                grid.clear_tower(footprint.start_x, footprint.start_y, footprint.size);
                            }
                        }
                    }
                }
            } else {
                println!(
                    "⚖️ TIEBREAKER: Both teams have equal lowest HP — it's a DRAW!"
                );
            }

            match_state.phase = MatchPhase::GameOver;
            println!(
                "🛑 MATCH OVER! Final Score: {}-{}",
                match_state.blue_crowns, match_state.red_crowns
            );
        }
    }

    // Elixir Generation
    let multiplier = match match_state.phase {
        MatchPhase::Regular => 1.0,
        MatchPhase::GameOver => 0.0,
        _ => 2.0, // DoubleElixir and Overtime are both 2x
    };

    let elixir_gain = (1.0 / 2.8) * multiplier * delta;

    match_state.blue_elixir = (match_state.blue_elixir + elixir_gain).min(10.0);
    match_state.red_elixir = (match_state.red_elixir + elixir_gain).min(10.0);
}
