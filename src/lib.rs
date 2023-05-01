mod poker;
use crate::poker::Game;
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
    let mut game = Game::new()
        .with_hand_a(hand_a.as_str())
        .with_hand_b(hand_b.as_str())
        .with_community(community.as_str());
    match game.solve() {
        Ok((win, lose, tie)) => GameResult { win, lose, tie },
        _ => GameResult::default(),
    }
}
