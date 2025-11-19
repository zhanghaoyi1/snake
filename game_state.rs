use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use rand::Rng;
// 修正导入路径：从后端的 types 模块导入（与前端类型定义一致）
use crate::types::{Direction, Position, Snake, Food, GameState, GameMessage};

#[derive(Debug, Clone)]
pub struct GameRoom {
    room_id: String,
    snakes: HashMap<usize, Snake>,
    foods: Vec<Food>,
    map_size: (i32, i32),
    next_snake_id: usize,
    game_started: bool,
    game_over: bool,
    ready_players: HashSet<usize>,
    required_players: usize,
}

impl GameRoom {
    pub fn new(room_id: String) -> Self {
        eprintln!("[房间{}] 创建新房间，所需玩家数：{}", room_id, 2);
        Self {
            room_id,
            snakes: HashMap::new(),
            foods: Vec::new(),
            map_size: (20, 20),
            next_snake_id: 1,
            game_started: false,
            game_over: false,
            ready_players: HashSet::new(),
            required_players: 2,
        }
    }

    pub fn add_player(&mut self) -> usize {
        let snake_id = self.next_snake_id;
        self.next_snake_id += 1;

        let start_pos = self.generate_safe_start_position();
        let body = vec![
            start_pos,
            Position { x: start_pos.x - 1, y: start_pos.y },
            Position { x: start_pos.x - 2, y: start_pos.y },
        ];

        self.snakes.insert(
            snake_id,
            Snake {
                id: snake_id,
                body,
                direction: Direction::Right,
                alive: true,
                score: 0,
            }
        );

        eprintln!("[房间{}] 玩家加入，分配蛇ID：{}", self.room_id, snake_id);
        snake_id
    }

    fn generate_safe_start_position(&self) -> Position {
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(3..self.map_size.0 - 3);
            let y = rng.gen_range(3..self.map_size.1 - 3);
            let pos = Position { x, y };

            let is_overlapping = self.snakes.values()
                .any(|s| s.body.contains(&pos));

            if !is_overlapping {
                return pos;
            }
        }
    }

    pub fn player_ready(&mut self, snake_id: usize) -> bool {
        self.ready_players.insert(snake_id);
        eprintln!("[房间{}] 蛇{} 标记为准备", self.room_id, snake_id);
        self.check_start_condition()
    }

    fn check_start_condition(&mut self) -> bool {
        if self.snakes.len() == self.required_players && self.ready_players.len() == self.required_players {
            self.game_started = true;
            self.generate_foods();
            eprintln!("[房间{}] 所有玩家准备就绪，游戏开始！生成食物数：{}", self.room_id, self.foods.len());
            true
        } else {
            eprintln!("[房间{}] 等待玩家准备：当前准备数{} / 所需{}", self.room_id, self.ready_players.len(), self.required_players);
            false
        }
    }

    fn generate_foods(&mut self) {
        let snake_positions = self.get_all_snake_positions();
        self.generate_foods_with_positions(snake_positions, 3);
    }

    fn get_all_snake_positions(&self) -> HashSet<Position> {
        self.snakes.values()
            .flat_map(|s| s.body.iter().cloned())
            .collect()
    }

    fn generate_foods_with_positions(&mut self, snake_positions: HashSet<Position>, required: usize) {
        let max_attempts = 100;
        let mut attempts = 0;
        
        while self.foods.len() < required && attempts < max_attempts {
            attempts += 1;
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(0..self.map_size.0);
            let y = rng.gen_range(0..self.map_size.1);
            let pos = Position { x, y };

            if !snake_positions.contains(&pos) && !self.foods.iter().any(|f| f.position == pos) {
                self.foods.push(Food { position: pos });
            }
        }

        // 调试日志：确保食物生成正常
        if self.foods.len() < required {
            eprintln!("[房间{}] 警告：食物生成不足（需要{}，实际生成{}）", self.room_id, required, self.foods.len());
        }
    }

    pub fn handle_input(&mut self, snake_id: usize, direction: Direction) {
        if !self.game_started || self.game_over {
            eprintln!("[房间{}] 忽略输入：游戏未开始或已结束（蛇{}）", self.room_id, snake_id);
            return;
        }

        let snake = self.snakes.get_mut(&snake_id);
        if let Some(snake) = snake {
            if !snake.alive {
                eprintln!("[房间{}] 忽略输入：蛇{} 已死亡", self.room_id, snake_id);
                return;
            }

            let can_change = match (snake.direction, direction) {
                (Direction::Up, Direction::Down) | (Direction::Down, Direction::Up) => false,
                (Direction::Left, Direction::Right) | (Direction::Right, Direction::Left) => false,
                _ => true,
            };

            if can_change {
                eprintln!("[房间{}] 蛇{} 转向：{:?} → {:?}", self.room_id, snake_id, snake.direction, direction);
                snake.direction = direction;
            } else {
                eprintln!("[房间{}] 蛇{} 转向无效：当前{:?}，尝试转向{:?}", self.room_id, snake_id, snake.direction, direction);
            }
        } else {
            eprintln!("[房间{}] 输入失败：未找到蛇{}", self.room_id, snake_id);
        }
    }

    pub fn update(&mut self) {
        if !self.game_started || self.game_over {
            return;
        }

        let initial_snake_positions = self.get_all_snake_positions();
        let mut updated_snakes = HashMap::new();
        let mut new_foods = self.foods.clone();

        for (snake_id, snake) in &self.snakes {
            let mut new_snake = snake.clone();
            if !new_snake.alive {
                updated_snakes.insert(*snake_id, new_snake);
                continue;
            }

            let head = new_snake.body.first().cloned().unwrap();
            let new_head = match new_snake.direction {
                Direction::Up => Position { x: head.x, y: head.y - 1 },
                Direction::Down => Position { x: head.x, y: head.y + 1 },
                Direction::Left => Position { x: head.x - 1, y: head.y },
                Direction::Right => Position { x: head.x + 1, y: head.y },
            };

            // 边界检测
            let hit_wall = new_head.x < 0 || new_head.x >= self.map_size.0 || 
                          new_head.y < 0 || new_head.y >= self.map_size.1;
            let hit_body = initial_snake_positions.contains(&new_head);

            if hit_wall || hit_body {
                eprintln!("[房间{}] 蛇{} 死亡（撞墙：{}，撞身体：{}）", self.room_id, snake_id, hit_wall, hit_body);
                new_snake.alive = false;
                updated_snakes.insert(*snake_id, new_snake);
                continue;
            }

            new_snake.body.insert(0, new_head);

            let mut ate_food = false;
            for (i, food) in new_foods.iter().enumerate() {
                if new_head == food.position {
                    new_snake.score += 1;
                    new_foods.remove(i);
                    ate_food = true;
                    eprintln!("[房间{}] 蛇{} 吃食物！当前得分：{}", self.room_id, snake_id, new_snake.score);
                    break;
                }
            }

            if !ate_food {
                new_snake.body.pop();
            }

            updated_snakes.insert(*snake_id, new_snake);
        }

        self.snakes = updated_snakes;
        self.foods = new_foods;

        // 补充食物
        let current_positions = self.get_all_snake_positions();
        self.generate_foods_with_positions(current_positions, 3);

        // 检查游戏结束
        let alive_count = self.snakes.values().filter(|s| s.alive).count();
        if alive_count <= 1 {
            self.game_over = true;
            let rankings = self.get_rankings();
            eprintln!("[房间{}] 游戏结束！存活蛇数：{}，排名：{:?}", self.room_id, alive_count, rankings);
        }
    }

    pub fn get_state(&self) -> GameState {
        GameState {
            room_id: self.room_id.clone(),
            snakes: self.snakes.values().cloned().collect(),
            foods: self.foods.clone(),
            game_started: self.game_started,
            game_over: self.game_over,
        }
    }

    pub fn get_matching_status(&self) -> (usize, usize) {
        (self.snakes.len(), self.required_players)
    }

    pub fn get_rankings(&self) -> Vec<(usize, u32)> {
        let mut rankings: Vec<(usize, u32)> = self.snakes.values()
            .map(|s| (s.id, s.score))
            .collect();
        rankings.sort_by(|a, b| b.1.cmp(&a.1));
        rankings
    }

    pub fn reset(&mut self) {
        let snake_ids: Vec<usize> = self.snakes.keys().cloned().collect();
        let mut new_positions = Vec::with_capacity(snake_ids.len());
        
        for _ in &snake_ids {
            new_positions.push(self.generate_safe_start_position());
        }

        let mut new_snakes = HashMap::new();
        for (i, &snake_id) in snake_ids.iter().enumerate() {
            let start_pos = new_positions[i];
            new_snakes.insert(
                snake_id,
                Snake {
                    id: snake_id,
                    body: vec![
                        start_pos,
                        Position { x: start_pos.x - 1, y: start_pos.y },
                        Position { x: start_pos.x - 2, y: start_pos.y },
                    ],
                    direction: Direction::Right,
                    alive: true,
                    score: 0,
                }
            );
        }

        self.snakes = new_snakes;
        self.foods.clear();
        self.game_started = false;
        self.game_over = false;
        self.ready_players.clear();

        eprintln!("[房间{}] 重置游戏状态，等待玩家重新准备", self.room_id);
    }

    pub fn remove_player(&mut self, snake_id: usize) -> Option<Snake> {
        eprintln!("[房间{}] 蛇{} 离开房间", self.room_id, snake_id);
        let removed = self.snakes.remove(&snake_id);
        self.ready_players.remove(&snake_id);
        removed
    }
}

#[derive(Clone)]
pub struct GameManager {
    rooms: Arc<Mutex<HashMap<String, GameRoom>>>,
    next_room_id: u32,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            next_room_id: 1,
        }
    }

    pub fn join_or_create_room(&mut self) -> (String, usize) {
        let mut rooms = self.rooms.lock().unwrap();

        // 查找未满的房间
        for (room_id, room) in rooms.iter_mut() {
            if room.snakes.len() < room.required_players && !room.game_started {
                let snake_id = room.add_player();
                return (room_id.clone(), snake_id);
            }
        }

        // 创建新房间
        let room_id = format!("room_{}", self.next_room_id);
        self.next_room_id += 1;
        let mut room = GameRoom::new(room_id.clone());
        let snake_id = room.add_player();
        rooms.insert(room_id.clone(), room);

        (room_id, snake_id)
    }

    pub fn player_ready(&self, room_id: &str, snake_id: usize) -> bool {
        let mut rooms = self.rooms.lock().unwrap();
        rooms.get_mut(room_id).map_or(false, |room| room.player_ready(snake_id))
    }

    pub fn handle_input(&self, room_id: &str, snake_id: usize, direction: Direction) {
        let mut rooms = self.rooms.lock().unwrap();
        if let Some(room) = rooms.get_mut(room_id) {
            room.handle_input(snake_id, direction);
        } else {
            eprintln!("[警告] 处理输入失败：未找到房间{}", room_id);
        }
    }

    pub fn update_all_rooms(&self) -> HashMap<String, GameState> {
        let mut rooms = self.rooms.lock().unwrap();
        let mut states = HashMap::new();
        
        for (room_id, room) in rooms.iter_mut() {
            room.update();
            states.insert(room_id.clone(), room.get_state());
        }
        
        states
    }

    pub fn get_room_state(&self, room_id: &str) -> Option<GameState> {
        let rooms = self.rooms.lock().unwrap();
        rooms.get(room_id).map(|room| room.get_state())
    }

    pub fn get_matching_status(&self, room_id: &str) -> Option<(usize, usize)> {
        let rooms = self.rooms.lock().unwrap();
        rooms.get(room_id).map(|room| room.get_matching_status())
    }

    pub fn get_rankings(&self, room_id: &str) -> Option<Vec<(usize, u32)>> {
        let rooms = self.rooms.lock().unwrap();
        rooms.get(room_id).map(|room| room.get_rankings())
    }

    pub fn reset_room(&self, room_id: &str) {
        let mut rooms = self.rooms.lock().unwrap();
        if let Some(room) = rooms.get_mut(room_id) {
            room.reset();
        } else {
            eprintln!("[警告] 重置失败：未找到房间{}", room_id);
        }
    }

    pub fn leave_room(&self, room_id: &str, snake_id: usize) {
        let mut rooms = self.rooms.lock().unwrap();
        if let Some(room) = rooms.get_mut(room_id) {
            room.remove_player(snake_id);
            if room.snakes.is_empty() {
                eprintln!("[房间{}] 所有玩家离开，删除房间", room_id);
                rooms.remove(room_id);
            }
        } else {
            eprintln!("[警告] 离开房间失败：未找到房间{}", room_id);
        }
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new()
    }
}