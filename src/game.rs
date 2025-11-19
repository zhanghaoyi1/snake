use yew::prelude::*;
use web_sys::KeyboardEvent;
use crate::types::{GameState, Position, Direction, Snake, Food};

// ---------------- 匹配状态组件（原 MatchingStatus）----------------
#[function_component(MatchingStatus)]
pub fn matching_status(props: &MatchingStatusProps) -> Html {
    let MatchingStatusProps { current, required, on_ready, is_ready } = props;
    html! {
        <div class="matching">
            <p>{"匹配玩家: "}{current}{"/"}{required}</p>
            <button class="ready-btn" onclick={on_ready} disabled={*is_ready || current == required}>
                {if *is_ready { "已准备" } else { "准备开始" }}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MatchingStatusProps {
    pub current: usize,
    pub required: usize,
    pub on_ready: Callback<MouseEvent>,
    pub is_ready: bool,
}

// ---------------- 游戏地图组件（核心，接收后端 GameState 渲染）----------------
#[function_component(GameMap)]
pub fn game_map(props: &GameMapProps) -> Html {
    let GameMapProps { state } = props;
    
    // 从后端状态中获取当前游戏数据
    let (snakes, foods, game_started, game_over) = match state {
        Some(s) => (s.snakes.clone(), s.foods.clone(), s.game_started, s.game_over),
        None => (vec![], vec![], false, false),
    };

    html! {
        <div class="game-container">
            <div class="game-map" style="border: 2px solid #333; width: 400px; height: 400px; position: relative; margin: 0 auto;">
                // 渲染所有玩家的蛇（不同蛇用不同颜色区分）
                { for snakes.iter().map(|snake| render_snake(snake)) }
                // 渲染食物
                { for foods.iter().map(|food| render_food(food)) }
                // 游戏未开始提示
                if !game_started && snakes.is_empty() {
                    <div class="game-tip">{"等待玩家加入..."}</div>
                }
                // 游戏结束提示（后端控制结束逻辑）
                if game_over {
                    <div class="game-over-tip">{"游戏结束！"}</div>
                }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct GameMapProps {
    pub state: Option<GameState>, // 接收后端传递的游戏状态
}

// ---------------- 游戏结束排名组件（原 GameOver）----------------
#[function_component(GameOver)]
pub fn game_over(props: &GameOverProps) -> Html {
    let GameOverProps { rankings, on_restart } = props;
    html! {
        <div class="game-over-modal">
            <h3>{"游戏结束"}</h3>
            <div class="rankings">
                { rankings.iter().enumerate().map(|(i, (snake_id, score))| {
                    html! {
                        <div class="rank-item">
                            {"第"}{i+1}{"名: 蛇"}{snake_id}{" - "}{score}{"分"}
                        </div>
                    }
                }).collect::<Html>() }
            </div>
            <button class="restart-btn" onclick={on_restart}>{"重新开始"}</button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct GameOverProps {
    pub rankings: Vec<(usize, u32)>,
    pub on_restart: Callback<MouseEvent>,
}

// ---------------- 辅助渲染函数 ----------------
/// 渲染单条蛇（不同蛇ID对应不同颜色）
fn render_snake(snake: &Snake) -> Html {
    let color = get_snake_color(snake.id); // 根据蛇ID生成唯一颜色
    html! {
        <>
        { for snake.body.iter().enumerate().map(|(idx, segment)| {
            // 蛇头用深色，身体用浅色
            let bg_color = if idx == 0 { darken_color(color.clone()) } else { color.clone() };
            html! {
                <div 
                    key={format!("snake-{}-{}", snake.id, idx)}
                    style={format!(
                        "position: absolute; width: 20px; height: 20px; background: {}; border-radius: 3px; left: {}px; top: {}px;",
                        bg_color,
                        segment.x * 20, // 地图格子大小：20px/格
                        segment.y * 20
                    )}
                ></div>
            }
        })}
        </>
    }
}

/// 渲染单个食物
fn render_food(food: &Food) -> Html {
    html! {
        <div
            key={format!("food-{}-{}", food.position.x, food.position.y)}
            style={format!(
                "position: absolute; width: 20px; height: 20px; background: #ff3333; border-radius: 50%; left: {}px; top: {}px;",
                food.position.x * 20,
                food.position.y * 20
            )}
        ></div>
    }
}

/// 根据蛇ID生成唯一颜色（避免重复）
fn get_snake_color(snake_id: usize) -> String {
    let colors = [
        "#4CAF50", "#2196F3", "#FFC107", "#9C27B0", "#FF9800", 
        "#00BCD4", "#8BC34A", "#FF5722", "#607D8B", "#795548"
    ];
    let color_idx = snake_id % colors.len();
    colors[color_idx].to_string()
}

/// 加深颜色（用于蛇头）
fn darken_color(color: String) -> String {
    // 简单实现：将RGB值减少30（确保不小于0）
    let r = i32::from_str_radix(&color[1..3], 16).unwrap();
    let g = i32::from_str_radix(&color[3..5], 16).unwrap();
    let b = i32::from_str_radix(&color[5..7], 16).unwrap();
    
    let r_dark = (r - 30).max(0);
    let g_dark = (g - 30).max(0);
    let b_dark = (b - 30).max(0);
    
    format!("#{:02X}{:02X}{:02X}", r_dark, g_dark, b_dark)
}

// ---------------- 组件样式（内联样式已包含，可补充全局样式）----------------
impl GameMap {
    pub fn styles() -> &'static str {
        r#"
            .game-container {
                margin: 20px 0;
            }
            .game-tip {
                position: absolute;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                font-size: 18px;
                color: #666;
            }
            .game-over-tip {
                position: absolute;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                font-size: 24px;
                color: #ff3333;
                font-weight: bold;
                background: rgba(255, 255, 255, 0.8);
                padding: 10px 20px;
                border-radius: 8px;
            }
            .matching {
                margin: 20px 0;
                padding: 20px;
                border: 1px solid #ddd;
                border-radius: 8px;
                text-align: center;
            }
            .ready-btn, .restart-btn {
                padding: 8px 16px;
                font-size: 16px;
                cursor: pointer;
                background: #4CAF50;
                color: white;
                border: none;
                border-radius: 4px;
                margin-top: 10px;
            }
            .ready-btn:disabled {
                background: #cccccc;
                cursor: not-allowed;
            }
            .game-over-modal {
                position: fixed;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                background: white;
                padding: 30px;
                border: 2px solid #333;
                border-radius: 8px;
                z-index: 100;
                text-align: center;
            }
            .rankings {
                margin: 20px 0;
                min-width: 200px;
            }
            .rank-item {
                margin: 10px 0;
                font-size: 18px;
            }
        "#
    }
}