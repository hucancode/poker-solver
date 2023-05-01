# Poker Solver
[![Fast check](https://github.com/hucancode/poker-solver/actions/workflows/fast-check.yml/badge.svg)](https://github.com/hucancode/poker-solver/actions/workflows/fast-check.yml)
[![Full check](https://github.com/hucancode/poker-solver/actions/workflows/full-check.yml/badge.svg)](https://github.com/hucancode/poker-solver/actions/workflows/full-check.yml)

## Motivation

This tool will help you approximate your winning chance in a [Texas Holdem](https://en.wikipedia.org/wiki/Texas_hold_%27em) poker game.
There is a web front end for this tool at [here](https://github.com/hucancode/poker-simulator)

Without a doubt, luck plays a significant role here,
but stepping into the game without a good mathematical foundation is equivalent to doing charity.
I hope my codes can do a little help preparing you on that aspect ☺

## Install

### Build main program

Install `rustup` if you have not done so. Then build the program with

```bash
cargo build
```

### Build for the web

You need [wasm-pack](https://rustwasm.github.io/wasm-pack/)

```bash
wasm-pack build --target web
```

Output would be at `pkg` folder, install it using `npm install ./path/to/pkg/`

## Run

Run the program with

```bash
cargo run -- <Community Cards> <Your Hand> [Their Hand]
# Example
cargo run -- 2s3s4d6s7s AsAd KsQs
cargo run -- 2s3s4d AsAd
cargo run -- 2s3s4d3d AsAd
# or you can run it directly like this
./poker-solver 2s3s4d3d AsAd
```

### Input format

- Community cards consist of 3-5 cards
- Your hand consist of 2 cards
- Their hand consist of 0-2 cards

Card notation

- A card represented by 2 characters, card rank and card suit
- Card rank can accept `23456789TJQKA`
- Card suit can accept `scdh`, stand for `Spade ♠`, `Club ♣`, `Diamond ♦`, `Heart ♥`

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

## Methodology

### Hand evaluation

I use some [bit math](<https://en.wikipedia.org/wiki/Mask_(computing)>) to match given hand of cards to a set of precomputed patterns. Then rank the hand accordingly.
Kindly check `src/poker/evaluator.rs` for single game evaluation algorithm.
Kindly check `src/poker/game.rs` for outcome simulation algorithm.

### Performance

Here are the number of possible outcome in each given game phase

|                   | Flop (2 hidden cards) | Turn (1 hidden card) | River (All cards visible) |
| ----------------- | --------------------- | -------------------- | ------------------------- |
| Possible Outcomes | 1070190               | 45540                | 990                       |

Here are some tests I made, you can run those tests yourself with

```bash
cargo test
```

| Test Case                                    | Game Code        | Solve Time      |
| -------------------------------------------- | ---------------- | --------------- |
| Both hands revealed, Flop                    | AsAd,KsKd,2s3s7s | 84.6ms ± 2.2ms  |
| All community cards revealed                 | AsAd,,2s3s4s5s6s | 50.2ms ± 2.7ms  |
| Turn                                         | AsAd,,2s3s4s5s   | 290.0ms ± 2.9ms |
| Early royal straight-flush                   | KsAs,,TsJsQs     | 3.721s ± 0.069s |
| Early quad                                   | AsAc,,AdAhKs     | 4.334s ± 0.082s |
| Early full-house                             | AsAd,,2c2s2d     | 2.787s ± 0.033s |
| Early pair of Ace, +1 card to flush/straight | AsAd,,2s3s4s     | 3.999s ± 0.029s |
| Early pair of Ace                            | AsAd,,2c6s8s     | 4.412s ± 0.012s |
| High cards                                   | TdQh,,2c6s8s     | 4.415s ± 0.031s |
| Low cards                                    | 6s2h,,8cTdQh     | 4.282s ± 0.019s |

- The program perform relatively well given the large number of possibilites
- With `wasm` version, expect 2~4 times slower (which is ~16 seconds worse case, still acceptable by my standard)
