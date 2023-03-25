mod poker;
use poker::Game;
use poker::HandConverter;
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
        HandConverter::pretify(community),
        HandConverter::pretify(hand_a),
        HandConverter::pretify(hand_b),
    );
    if stdout().flush().is_err() {
        return;
    }
    let game = Game::new()
        .with_hand_a(hand_a)
        .with_hand_b(hand_b)
        .with_community(community);
    let (win, lose, tie) = game.solve();
    let win_rate = win as f32 / (win + lose + tie) as f32 * 100.0;
    println!(
        "\r\
        👑 Win:               {:>10}\n\
        💸 Lose:              {:>10}\n\
        🤝 Tie:               {:>10}\n\
        🧮 You win:           {:>9}%",
        win, lose, tie, win_rate
    );
}
