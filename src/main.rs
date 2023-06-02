mod poker;
mod utils;
use crate::poker::Game;
use crate::utils::prettify;
use std::env;
use std::io::stdout;
use std::io::Write;
use std::sync::Mutex;
use std::sync::OnceLock;

fn main() {
    let args: Vec<String> = env::args().collect();
    let empty = &String::new();
    let community = args.get(1).unwrap_or(empty);
    let hand_a = args.get(2).unwrap_or(empty);
    let hand_b = args.get(3).unwrap_or(empty);
    solve(hand_a, hand_b, community);
}

fn solve(hand_a: &str, hand_b: &str, community: &str) {
    static GAME_INSTANCE: OnceLock<Mutex<Game>> = OnceLock::new();
    let mutex = GAME_INSTANCE.get_or_init(|| Mutex::new(Game::new()));
    if let Ok(mut game) = mutex.lock() {
        print!(
            "\n\
            🎴 Community cards: {:>12}\n\
            🎴 Your hand:       {:>12}\n\
            🎴 Their hand:      {:>12}\n\
            \n\
            Running numbers...",
            prettify(community),
            prettify(hand_a),
            prettify(hand_b),
        );
        if stdout().flush().is_err() {
            return;
        }
        match game.solve_by(hand_a, hand_b, community) {
            Ok((win, lose, tie)) => {
                let win_rate = win as f32 / (win + lose + tie) as f32 * 100.0;
                println!(
                    "\r\
                    👑 Win:               {win:>10}\n\
                    💸 Lose:              {lose:>10}\n\
                    🤝 Tie:               {tie:>10}\n\
                    🧮 You win:           {win_rate:>9}%"
                );
            }
            Err(e) => {
                println!("\r{:^32}\n", e);
            }
        };
    } else {
        println!("could not get game solver instance");
    }
}
