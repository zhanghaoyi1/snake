// 导入所有必要依赖（包括 JsCast trait）
use yew::prelude::*;
use web_sys::{WebSocket, MessageEvent, CloseEvent, console};
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast; // 关键：导入 JsCast trait，提供 unchecked_ref 方法
// 只导入实际使用的类型，清理未使用导入
use crate::types::{GameState, GameMessage};

#[derive(Debug, Clone)]
pub struct WsClient {
    ws: Option<WebSocket>,
    on_game_state: Option<Callback<GameState>>,
    on_matching_status: Option<Callback<(usize, usize)>>,
    on_game_over: Option<Callback<Vec<(usize, u32)>>>,
}

impl WsClient {
    // 带 URL 参数的构造函数
    pub fn new(url: &str) -> Self {
        let ws = WebSocket::new(url)
            .expect("Failed to create WebSocket connection. Check URL or network.");
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
        // 监听连接成功事件
        let open_closure = Closure::wrap(Box::new(|| {
            console::log_1(&"WebSocket connection established successfully!".into());
        }) as Box<dyn FnMut()>);
        // 使用 unchecked_ref（依赖 JsCast trait）
        ws.set_onopen(Some(open_closure.as_ref().unchecked_ref()));
        open_closure.forget();

        Self {
            ws: Some(ws),
            on_game_state: None,
            on_matching_status: None,
            on_game_over: None,
        }
    }

    // 注册游戏状态回调
    pub fn on_game_state(mut self, callback: Callback<GameState>) -> Self {
        self.on_game_state = Some(callback);
        self
    }

    // 注册匹配状态回调
    pub fn on_matching_status(mut self, callback: Callback<(usize, usize)>) -> Self {
        self.on_matching_status = Some(callback);
        self
    }

    // 注册游戏结束回调
    pub fn on_game_over(mut self, callback: Callback<Vec<(usize, u32)>>) -> Self {
        self.on_game_over = Some(callback);
        self
    }

    // 发送消息到后端
    pub fn send(&self, msg: GameMessage) {
        if let Some(ws) = &self.ws {
            match serde_json::to_string(&msg) {
                Ok(msg_str) => {
                    if ws.send_with_str(&msg_str).is_err() {
                        console::error_1(&"Failed to send WebSocket message: connection closed.".into());
                    }
                }
                Err(e) => {
                    console::error_1(&format!("Failed to serialize message: {}", e).into());
                }
            }
        } else {
            console::error_1(&"WebSocket not initialized. Cannot send message.".into());
        }
    }

    // 启动 WebSocket 监听
    pub fn start_listening(&mut self) {
        let ws = match &self.ws {
            Some(ws) => ws,
            None => {
                console::error_1(&"Cannot start listening: WebSocket not initialized.".into());
                return;
            }
        };

        // 克隆回调
        let on_game_state = self.on_game_state.clone();
        let on_matching_status = self.on_matching_status.clone();
        let on_game_over = self.on_game_over.clone();

        // 监听后端消息
        let msg_closure = Closure::wrap(Box::new(move |e: MessageEvent| {
            let text = match e.data().as_string() {
                Some(t) => t,
                None => {
                    console::warn_1(&"Received non-string WebSocket message.".into());
                    return;
                }
            };

            match serde_json::from_str::<GameMessage>(&text) {
                Ok(GameMessage::GameState(state)) => {
                    if let Some(cb) = on_game_state.clone() {
                        cb.emit(state);
                    }
                }
                Ok(GameMessage::MatchingStatus { current, required }) => {
                    if let Some(cb) = on_matching_status.clone() {
                        cb.emit((current, required));
                    }
                }
                Ok(GameMessage::GameOver { rankings }) => {
                    if let Some(cb) = on_game_over.clone() {
                        cb.emit(rankings);
                    }
                }
                Ok(_) => {} // 忽略其他消息类型
                Err(e) => {
                    console::error_1(&format!("Failed to parse WebSocket message: {}", e).into());
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        // 监听连接关闭
        let close_closure = Closure::wrap(Box::new(move |e: CloseEvent| {
            console::warn_1(&format!(
                "WebSocket connection closed. Code: {}, Reason: {}",
                e.code(),
                e.reason()
            ).into());
        }) as Box<dyn FnMut(CloseEvent)>);

        // 使用 unchecked_ref（依赖 JsCast trait）
        ws.set_onmessage(Some(msg_closure.as_ref().unchecked_ref()));
        ws.set_onclose(Some(close_closure.as_ref().unchecked_ref()));
        
        msg_closure.forget();
        close_closure.forget();
    }
}