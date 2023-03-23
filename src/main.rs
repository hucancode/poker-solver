mod poker;
use poker::HandComparer;
use poker::HandConverter;
use poker::Game;


fn main() {
    println!("Hello, world!");
    let comparer = HandComparer::new();
    let game = Game {
        hand_a: HandConverter::string_to_mask(&String::from("AsAd")),
        hand_b: HandConverter::string_to_mask(&String::from("")),
        community: HandConverter::string_to_mask(&String::from("2s3s4s")),
    };
    let (win, lose, tie) = game.solve(&comparer);
    println!("win {} lose {} tie {}", win, lose, tie);
}
