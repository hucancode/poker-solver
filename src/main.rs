mod poker;
mod utils;
use crate::poker::Game;
use crate::utils::pretify;
use std::env;
use std::io::stdout;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let empty = &String::new();
    let community = args.get(1).unwrap_or(empty);
    let hand_a = args.get(2).unwrap_or(empty);
    let hand_b = args.get(3).unwrap_or(empty);
    print!(
        "\n\
        🎴 Community cards: {:>12}\n\
        🎴 Your hand:       {:>12}\n\
        🎴 Their hand:      {:>12}\n\
        \n\
        Running numbers...",
        pretify(community),
        pretify(hand_a),
        pretify(hand_b),
    );
    if stdout().flush().is_err() {
        return;
    }
    let mut game = Game::new()
        .with_hand_a(hand_a.as_str())
        .with_hand_b(hand_b.as_str())
        .with_community(community.as_str());
    match game.solve() {
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
    }
}
