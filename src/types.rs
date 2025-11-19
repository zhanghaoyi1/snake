use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snake {
    pub id: usize,
    pub body: Vec<Position>,
    pub direction: Direction,
    pub alive: bool,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Food {
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameMessage {
    PlayerInput(Direction),
    Ready,
    GameState(GameState),
    MatchingStatus { current: usize, required: usize },
    GameOver { rankings: Vec<(usize, u32)> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameState {
    pub room_id: String,
    pub snakes: Vec<Snake>,
    pub foods: Vec<Food>,
    pub game_started: bool,
    pub game_over: bool,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn is_in_bounds(&self, map_size: i32) -> bool {
        self.x >= 0 && self.x < map_size && self.y >= 0 && self.y < map_size
    }
}

impl Snake {
    pub fn head(&self) -> Option<&Position> {
        self.body.first()
    }

    pub fn hits_self(&self) -> bool {
        if let Some(head) = self.head() {
            self.body[1..].contains(head)
        } else {
            false
        }
    }

    pub fn hits_other(&self, other: &Snake) -> bool {
        if self.id == other.id {
            return false;
        }
        if let Some(head) = self.head() {
            other.body.contains(head)
        } else {
            false
        }
    }
}