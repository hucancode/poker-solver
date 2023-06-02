mod poker;
use crate::poker::Game;
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
static GAME_INSTANCE: OnceLock<Mutex<Game>> = OnceLock::new();

#[wasm_bindgen]
pub fn solve(hand_a: String, hand_b: String, community: String) -> GameResult {
    let mutex = GAME_INSTANCE.get_or_init(|| Mutex::new(Game::new()));
    if let Ok(mut game) = mutex.lock() {
        match game.solve_by(hand_a.as_str(), hand_b.as_str(), community.as_str()) {
            Ok((win, lose, tie)) => GameResult { win, lose, tie },
            _ => GameResult::default(),
        }
    } else {
        GameResult::default()
    }
}
