pub mod poker;
pub use crate::poker::{solve_ranges, EquityResult, Game, Hand, Range};

use std::fmt::Write as _;
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
    let mut game = Game::new();
    if let Ok((win, lose, tie)) =
        game.solve_by(hand_a.as_str(), hand_b.as_str(), community.as_str())
    {
        return GameResult { win, lose, tie };
    }
    GameResult::default()
}

fn equity_to_json(r: &EquityResult) -> String {
    let mut s = String::new();
    write!(
        s,
        "{{\"iterations\":{},\"hero_win\":{},\"hero_tie\":{},\"hero_lose\":{},\"villain_equity\":[",
        r.iterations, r.hero_win, r.hero_tie, r.hero_lose
    )
    .unwrap();
    for (i, v) in r.villain_equity.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        write!(s, "{}", v).unwrap();
    }
    s.push_str("]}");
    s
}

fn error_to_json(msg: &str) -> String {
    let escaped: String = msg.chars().flat_map(|c| match c {
        '"' => vec!['\\', '"'],
        '\\' => vec!['\\', '\\'],
        '\n' => vec!['\\', 'n'],
        c => vec![c],
    }).collect();
    format!("{{\"error\":\"{}\"}}", escaped)
}

#[wasm_bindgen]
pub fn solve_multi(
    hero: String,
    villain_ranges: Vec<String>,
    community: String,
    max_iterations: u32,
    seed: u32,
) -> String {
    let hero_range = match Range::from_notation(&hero) {
        Ok(r) => r,
        Err(e) => return error_to_json(&format!("hero: {}", e)),
    };
    if hero_range.combos.len() != 1 {
        return error_to_json("hero must be a single combo (e.g. AsKs)");
    }
    let hero_mask = hero_range.combos[0].mask;

    let mut villains: Vec<Range> = Vec::with_capacity(villain_ranges.len());
    for (i, s) in villain_ranges.iter().enumerate() {
        let r = if s.trim().is_empty() {
            Range::any()
        } else {
            match Range::from_notation(s) {
                Ok(r) => r,
                Err(e) => return error_to_json(&format!("villain {}: {}", i, e)),
            }
        };
        villains.push(r);
    }

    let community_mask = Hand::from_string(&community).mask;
    let seed64 = if seed == 0 {
        0x9E37_79B9_7F4A_7C15
    } else {
        seed as u64
    };
    match solve_ranges(
        hero_mask,
        &villains,
        community_mask,
        max_iterations as u64,
        seed64,
    ) {
        Ok(r) => equity_to_json(&r),
        Err(e) => error_to_json(&e),
    }
}
