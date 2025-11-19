// src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn is_in_bounds(&self, map_size: i32) -> bool {
        self.x >= 0 && self.x < map_size && self.y >= 0 && self.y < map_size
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snake {
    pub id: usize,
    pub body: Vec<Position>,
    pub direction: Direction,
    pub alive: bool,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Food {
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub room_id: String,
    pub snakes: Vec<Snake>,
    pub foods: Vec<Food>,
    pub game_started: bool,
    pub game_over: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameMessage {
    PlayerInput(Direction),
    Ready,
    GameState(GameState),
    MatchingStatus { current: usize, required: usize },
    GameOver { rankings: Vec<(usize, u32)> },
}