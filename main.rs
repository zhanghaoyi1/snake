use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::from_str;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

// 导入本地模块
mod types;
use types::GameMessage;

mod game_state;
use game_state::GameManager;

#[derive(Clone)]
struct AppState {
    connections: Arc<Mutex<HashMap<usize, broadcast::Sender<String>>>>,
    room_broadcasters: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    game_manager: GameManager,
    player_rooms: Arc<Mutex<HashMap<usize, String>>>,
    player_snakes: Arc<Mutex<HashMap<usize, usize>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            room_broadcasters: Arc::new(Mutex::new(HashMap::new())),
            game_manager: GameManager::new(),
            player_rooms: Arc::new(Mutex::new(HashMap::new())),
            player_snakes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_room_broadcaster(&self, room_id: &str) -> broadcast::Sender<String> {
        let mut room_bcs = self.room_broadcasters.lock().unwrap();
        room_bcs.entry(room_id.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            })
            .clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new();

    // 游戏循环：固定间隔更新并广播状态
    let game_state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(120));
        loop {
            interval.tick().await;
            let room_states = game_state_clone.game_manager.update_all_rooms();
            
            for (room_id, game_state) in &room_states {
                let room_bc = game_state_clone.get_room_broadcaster(room_id);
                
                // 广播当前游戏状态
                let msg = GameMessage::GameState(game_state.clone());
                if let Ok(msg_str) = serde_json::to_string(&msg) {
                    let _ = room_bc.send(msg_str);
                }

                // 游戏结束时广播排名
                if game_state.game_over {
                    if let Some(rankings) = game_state_clone.game_manager.get_rankings(room_id) {
                        let msg = GameMessage::GameOver { rankings };
                        if let Ok(msg_str) = serde_json::to_string(&msg) {
                            let _ = room_bc.send(msg_str);
                        }
                    }
                }
            }
        }
    });

    // 路由设置
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state);

    // 启动服务器（直接绑定地址，axum 自动处理 TcpListener）
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("WebSocket server running on ws://0.0.0.0:3000");
    
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, mut state: AppState) {
    // 生成唯一玩家ID
    static NEXT_PLAYER_ID: std::sync::Mutex<usize> = std::sync::Mutex::new(0);
    let player_id = {
        let mut id = NEXT_PLAYER_ID.lock().unwrap();
        *id += 1;
        *id
    };

    // 玩家加入房间或创建新房间
    let (room_id, snake_id) = state.game_manager.join_or_create_room();
    state.player_rooms.lock().unwrap().insert(player_id, room_id.clone());
    state.player_snakes.lock().unwrap().insert(player_id, snake_id);

    // 初始化WebSocket收发通道
    let (mut sender, mut receiver) = socket.split();
    let room_bc = state.get_room_broadcaster(&room_id);
    let mut room_rx = room_bc.subscribe();
    state.connections.lock().unwrap().insert(player_id, room_bc);

    tracing::info!("Player {} joined room {} (snake ID: {})", player_id, room_id, snake_id);

    // 发送初始匹配状态
    if let Some((current, required)) = state.game_manager.get_matching_status(&room_id) {
        let msg = GameMessage::MatchingStatus { current, required };
        if let Ok(msg_str) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(msg_str)).await;
        }
    }

    // 接收客户端消息
    let recv_state = state.clone();
    let player_id_copy = player_id;
    let room_id_copy = room_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    tracing::info!("收到玩家 {} 消息: {}", player_id_copy, text);
                    
                    // 处理方向输入
                    if let Ok(GameMessage::PlayerInput(direction)) = from_str(&text) {
                        let snake_id = {
                            let snakes = recv_state.player_snakes.lock().unwrap();
                            snakes.get(&player_id_copy).copied()
                        };

                        if let Some(snake_id) = snake_id {
                            recv_state.game_manager.handle_input(&room_id_copy, snake_id, direction);
                            tracing::info!("蛇 {} 方向更新为 {:?}", snake_id, direction);
                        }
                    }
                    // 处理准备状态
                    else if let Ok(GameMessage::Ready) = from_str(&text) {
                        let snake_id = {
                            let snakes = recv_state.player_snakes.lock().unwrap();
                            snakes.get(&player_id_copy).copied()
                        };

                        if let Some(snake_id) = snake_id {
                            // 重置房间（如果游戏已结束）
                            let room_state = recv_state.game_manager.get_room_state(&room_id_copy);
                            if room_state.map_or(false, |s| s.game_over) {
                                recv_state.game_manager.reset_room(&room_id_copy);
                                tracing::info!("房间 {} 游戏已结束，重置房间", room_id_copy);
                            }

                            // 标记玩家为准备状态
                            let game_started = recv_state.game_manager.player_ready(&room_id_copy, snake_id);
                            
                            // 广播更新后的匹配状态
                            if let Some((current, required)) = recv_state.game_manager.get_matching_status(&room_id_copy) {
                                let msg = GameMessage::MatchingStatus { current, required };
                                let room_bc = recv_state.get_room_broadcaster(&room_id_copy);
                                if let Ok(msg_str) = serde_json::to_string(&msg) {
                                    let _ = room_bc.send(msg_str);
                                }
                            }

                            if game_started {
                                tracing::info!("房间 {} 所有玩家准备就绪，游戏开始", room_id_copy);
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    tracing::info!("Player {} disconnected", player_id_copy);
                    break;
                }
                _ => {}
            }
        }
    });

    // 发送服务器消息到客户端
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = room_rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                tracing::warn!("发送消息给玩家 {} 失败（可能已断开连接）", player_id);
                break;
            }
        }
    });

    // 等待任务结束
    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    }

    // 清理玩家资源
    state.connections.lock().unwrap().remove(&player_id);
    if let (Some(room_id), Some(snake_id)) = (
        state.player_rooms.lock().unwrap().remove(&player_id),
        state.player_snakes.lock().unwrap().remove(&player_id)
    ) {
        state.game_manager.leave_room(&room_id, snake_id);
        tracing::info!("Player {} left room {}", player_id, room_id);
    }
}