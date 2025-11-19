use yew::prelude::*;
use web_sys::KeyboardEvent; // 补充键盘事件导入（之前遗漏导致警告）

// 导出模块
pub mod game;
pub mod websocket;
pub mod types;

// 导入联机组件和类型
use game::{GameMap, MatchingStatus, GameOver};
use websocket::WsClient;
use types::{GameMessage, Direction, GameState};

#[function_component(App)]
fn app() -> Html {
    // 状态管理
    let ws_client = use_state(|| None::<WsClient>);
    let game_state = use_state(|| None::<GameState>);
    let matching_status = use_state(|| (0, 2)); // (当前玩家数, 所需玩家数)
    let game_over_rankings = use_state(|| None::<Vec<(usize, u32)>>);
    let is_ready = use_state(|| false);

    // 初始化WebSocket连接
    {
        let ws_client = ws_client.clone();
        let game_state_clone = game_state.clone();
        let matching_status_clone = matching_status.clone();
        let game_over_rankings_clone = game_over_rankings.clone();
        
        use_effect_with((), move |_| {
            // 连接后端WebSocket服务
            let mut client = WsClient::new("ws://47.100.220.180:3000/ws");
            
            // 注册游戏状态回调
            let game_state_cb = Callback::from(move |state: GameState| {
                game_state_clone.set(Some(state));
            });
            client = client.on_game_state(game_state_cb);

            // 注册匹配状态回调
            let matching_cb = Callback::from(move |(current, required): (usize, usize)| {
                matching_status_clone.set((current, required));
            });
            client = client.on_matching_status(matching_cb);

            // 注册游戏结束回调
            let game_over_cb = Callback::from(move |rankings: Vec<(usize, u32)>| {
                game_over_rankings_clone.set(Some(rankings));
            });
            client = client.on_game_over(game_over_cb);

            // 启动WebSocket监听
            client.start_listening();
            ws_client.set(Some(client));

            || ()
        });
    }

    // 发送消息到后端
    let send_message = {
        let ws_client = ws_client.clone();
        move |msg: GameMessage| {
            if let Some(client) = &*ws_client {
                client.send(msg);
            }
        }
    };

    // 处理“准备开始”按钮点击
    let handle_ready = {
        let send_message = send_message.clone();
        let is_ready = is_ready.clone();
        Callback::from(move |_: MouseEvent| {
            send_message(GameMessage::Ready);
            is_ready.set(true);
        })
    };

    // 处理“重新开始”按钮点击
    let handle_restart = {
        let send_message = send_message.clone();
        let game_over_rankings = game_over_rankings.clone();
        let is_ready = is_ready.clone();
        Callback::from(move |_: MouseEvent| {
            send_message(GameMessage::Ready);
            game_over_rankings.set(None);
            is_ready.set(true);
        })
    };

    // 处理键盘输入（方向控制）
    let handle_keydown = {
        let send_message = send_message.clone();
        let game_state = game_state.clone();
        Callback::from(move |e: KeyboardEvent| {
            let state = game_state.as_ref();
            if state.map_or(true, |s| !s.game_started || s.game_over) {
                return;
            }

            match e.key().as_str() {
                "ArrowUp" => send_message(GameMessage::PlayerInput(Direction::Up)),
                "ArrowDown" => send_message(GameMessage::PlayerInput(Direction::Down)),
                "ArrowLeft" => send_message(GameMessage::PlayerInput(Direction::Left)),
                "ArrowRight" => send_message(GameMessage::PlayerInput(Direction::Right)),
                _ => {}
            }
        })
    };

    html! {
        <div class="app" onkeydown={handle_keydown} tabindex="0" style="outline: none;">
            <h1>{"多人贪吃蛇游戏"}</h1>
            
            // 匹配状态组件
            <MatchingStatus 
                current={matching_status.0} 
                required={matching_status.1} 
                on_ready={handle_ready}
                is_ready={*is_ready}
            />
            
            // 游戏地图组件
            <GameMap state={(*game_state).clone()} />
            
            // 游戏结束组件
            if let Some(rankings) = &*game_over_rankings {
                <GameOver 
                    rankings={rankings.clone()} 
                    on_restart={handle_restart}
                />
            }

            // 全局样式
            <style>
                { r#"
                    .app {
                        text-align: center;
                        margin: 20px auto;
                        max-width: 500px;
                        font-family: Arial, sans-serif;
                    }
                    h1 {
                        color: #333;
                        margin-bottom: 30px;
                    }
                "# }
                { game::GameMap::styles() }
            </style>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    // 兼容旧版本 Yew 的渲染方式（替换 render_to_body()）
    yew::Renderer::<App>::new().render();
}