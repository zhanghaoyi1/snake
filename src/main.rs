use yew::prelude::*;
use web_sys::{KeyboardEvent, MouseEvent};
use snake_game::types::{GameMessage, Direction, GameState};
use snake_game::websocket::WsClient;
// 从 game.rs 导入正式组件（匹配状态、游戏地图、游戏结束排名）
use snake_game::game::{GameMap, MatchingStatus, GameOver};

#[function_component(App)]
fn app() -> Html {
    // 状态管理：WebSocket客户端、游戏状态、匹配状态、游戏结束排名、是否准备
    let ws_client = use_state(|| None::<WsClient>);
    let game_state = use_state(|| None::<GameState>);
    let matching_status = use_state(|| (0, 2)); // (当前玩家数, 所需玩家数)
    let game_over_rankings = use_state(|| None::<Vec<(usize, u32)>>);
    let is_ready = use_state(|| false);

    // 初始化WebSocket连接（组件挂载时执行一次）
    {
        let ws_client = ws_client.clone();
        let game_state_clone = game_state.clone();
        let matching_status_clone = matching_status.clone();
        let game_over_rankings_clone = game_over_rankings.clone();
        
        use_effect_with((), move |_| {
            // 连接后端WebSocket服务（IP和端口与后端一致）
            let mut client = WsClient::new("ws://47.100.220.180:3000/ws");
            
            // 注册游戏状态回调（接收后端推送的游戏状态，更新前端渲染）
            let game_state_cb = Callback::from(move |state: GameState| {
                game_state_clone.set(Some(state));
            });
            client = client.on_game_state(game_state_cb);

            // 注册匹配状态回调（更新当前在线玩家数）
            let matching_cb = Callback::from(move |(current, required): (usize, usize)| {
                matching_status_clone.set((current, required));
            });
            client = client.on_matching_status(matching_cb);

            // 注册游戏结束回调（接收排名信息）
            let game_over_cb = Callback::from(move |rankings: Vec<(usize, u32)>| {
                game_over_rankings_clone.set(Some(rankings));
            });
            client = client.on_game_over(game_over_cb);

            // 启动WebSocket监听（接收后端消息）
            client.start_listening();
            ws_client.set(Some(client));

            // 组件卸载时无清理操作（WebSocket自动关闭）
            || ()
        });
    }

    // 发送消息到后端（封装通用发送逻辑）
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
            is_ready.set(true); // 标记为已准备，禁用按钮
        })
    };

    // 处理“重新开始”按钮点击
    let handle_restart = {
        let send_message = send_message.clone();
        let game_over_rankings = game_over_rankings.clone();
        let is_ready = is_ready.clone();
        Callback::from(move |_: MouseEvent| {
            send_message(GameMessage::Ready); // 重新发送准备指令
            game_over_rankings.set(None); // 清除排名信息
            is_ready.set(true); // 标记为已准备
        })
    };

    // 处理键盘输入（方向键控制蛇移动）
    let handle_keydown = {
        let send_message = send_message.clone();
        let game_state = game_state.clone();
        Callback::from(move |e: KeyboardEvent| {
            // 游戏未开始/已结束时，忽略键盘输入
            let state = game_state.as_ref();
            if state.map_or(true, |s| !s.game_started || s.game_over) {
                return;
            }

            // 方向键映射为后端识别的Direction类型，发送到后端
            match e.key().as_str() {
                "ArrowUp" => send_message(GameMessage::PlayerInput(Direction::Up)),
                "ArrowDown" => send_message(GameMessage::PlayerInput(Direction::Down)),
                "ArrowLeft" => send_message(GameMessage::PlayerInput(Direction::Left)),
                "ArrowRight" => send_message(GameMessage::PlayerInput(Direction::Right)),
                _ => {} // 忽略其他按键
            }
        })
    };

    // 主页面结构（整合所有组件，添加全局样式）
    html! {
        <div class="app" onkeydown={handle_keydown} tabindex="0" style="outline: none;">
            <h1>{"多人联机贪吃蛇"}</h1>
            
            // 匹配状态组件：显示当前玩家数、所需玩家数，提供准备按钮
            <MatchingStatus 
                current={matching_status.0} 
                required={matching_status.1} 
                on_ready={handle_ready}
                is_ready={*is_ready}
            />
            
            // 游戏地图组件：接收后端游戏状态，渲染蛇、食物
            <GameMap state={(*game_state).clone()} />
            
            // 游戏结束组件：显示排名，提供重新开始按钮（仅游戏结束时显示）
            if let Some(rankings) = &*game_over_rankings {
                <GameOver 
                    rankings={rankings.clone()} 
                    on_restart={handle_restart}
                />
            }

            // 全局样式（结合游戏组件内置样式，统一页面风格）
            <style>
                { format!(
                    r#"
                        /* 继承游戏组件样式 */
                        {}

                        /* 页面全局样式 */
                        .app {{
                            text-align: center;
                            margin: 20px auto;
                            max-width: 500px;
                            font-family: Arial, sans-serif;
                        }}
                        h1 {{
                            color: #333;
                            margin-bottom: 30px;
                        }}
                    "#,
                    GameMap::styles() // 导入 game.rs 中定义的组件样式
                )}
            </style>
        </div>
    }
}

fn main() {
    // 初始化错误捕获（便于前端调试，打印panic信息到浏览器控制台）
    console_error_panic_hook::set_once();
    // Yew 0.21+ 正确的渲染方式：将应用挂载到页面body
    yew::Renderer::<App>::new().render();
}