mod poker;
use poker::Game;
use poker::HandComparer;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let empty = &String::new();
    let community = args.get(1).unwrap_or(empty);
    let hand_a = args.get(2).unwrap_or(empty);
    let hand_b = args.get(3).unwrap_or(empty);
    println!(
        "Running numbers... \n\n\
        Community cards: {:>12}\n\
        Your hand:       {:>12}\n\
        Their hand:      {:>12}\n",
        community, hand_a, hand_b
    );
    let comparer = HandComparer::new();
    let game = Game::from_string(hand_a, hand_b, community);
    let (win, lose, tie) = game.solve(&comparer);
    let win_rate = win as f32 / (win + lose + tie) as f32 * 100.0;
    println!(
        "_____________________________\n\
        Win:               {:>10}\n\
        Lose:              {:>10}\n\
        Tie:               {:>10}\n\
        You win:           {:>9}%",
        win, lose, tie, win_rate
    );
}
