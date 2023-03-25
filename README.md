# Poker Solver

## Install

Install `rustup` if you have not done so. Then build the program with
```bash
cargo build
```
## Run

Run the program with
```bash
cargo run -- <Community Cards> <Your Hand> [Their Hand]
```
Example
```bash
cargo run -- 2s3s4d6s7s AsAd KsQs
cargo run -- 2s3s4d AsAd 
cargo run -- 2s3s4d3d AsAd 

# or you can run it directly like this
./poker-solver 2s3s4d3d AsAd
```
Where
- Community cards consist of 3-5 cards
- Your hand consist of 2 cards
- Their hand consist of 0-2 cards
The program will output all possible game outcomes. Here is an example:

```
🎴 Community cards:       2♠3♠7♠
🎴 Your hand:               A♠A♦
🎴 Their hand:

👑 Win:                   976740
💸 Lose:                   92820
🤝 Tie:                      630
🧮 You win:            91.26791%
```

## Card notation

- A card represented by 2 characters, card rank and card suit
- Card rank can accept `23456789TJQKA`
- Card suit can accept `scdh`, stand for `Spade ♠`, `Club ♣`, `Diamond ♦`, `Heart ♥`
