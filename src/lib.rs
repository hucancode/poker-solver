pub mod poker;
pub use crate::poker::Game;
use std::sync::Mutex;
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct GameResult {
    pub win: usize,
    pub lose: usize,
    pub tie: usize,
}

#[wasm_bindgen]
pub fn solve(hand_a: String, hand_b: String, community: String) -> GameResult {
    static GAME_INSTANCE: OnceLock<Mutex<Game>> = OnceLock::new();
    let mutex = GAME_INSTANCE.get_or_init(|| Mutex::new(Game::new()));
    if let Ok(mut game) = mutex.lock() {
        if let Ok((win, lose, tie)) =
            game.solve_by(hand_a.as_str(), hand_b.as_str(), community.as_str())
        {
            return GameResult { win, lose, tie };
        }
    }
    GameResult::default()
}
