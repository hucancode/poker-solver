mod poker;
use poker::Game;
use poker::HandComparer;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let empty = &String::from("");
    let community = args.get(1).unwrap_or(&empty);
    let hand_a = args.get(2).unwrap_or(&empty);
    let hand_b = args.get(3).unwrap_or(&empty);
    println!(
        "Solving game with the following configuration: 
             \n Community cards: {}
             \n Your hand: {}
             \n Their hand: {}",
        community, hand_a, hand_b
    );
    let comparer = HandComparer::new();
    let game = Game::from_string(&hand_a, &hand_b, &community);
    let (win, lose, tie) = game.solve(&comparer);
    let win_rate = win as f32 / (win + lose + tie) as f32 * 100.0;
    println!(
        "Win:{}/Lose:{}/Tie:{} (You win {}%)",
        win, lose, tie, win_rate
    );
}
