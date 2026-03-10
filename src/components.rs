#![allow(dead_code)]
use bevy::prelude::*;

// The exact fixed-point coordinate of the unit
#[derive(Component, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// The health pool
#[derive(Component, Debug)]
pub struct Health(pub i32);

// Identifies which team owns the unit
#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub enum Team {
    Blue, // Player 1
    Red,  // Player 2
}

// The Event triggered when the UI asks to drop a card
#[derive(Event)]
pub struct SpawnRequest {
    pub card_key: String,
    pub team: Team,
    pub grid_x: i32,
    pub grid_y: i32,
}
