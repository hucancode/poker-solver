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

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown
```

Output is `target/wasm32-unknown-unknown/release/poker_solver.wasm`.
Load it with a small hand-written glue (see `solver.js` in the consuming
project): `alloc`/`dealloc` for buffers, `solve` writes `[win, lose, tie]`
as 3×u32, `solve_multi` writes `[iterations, win, tie, lose, equity...]`
as f64, errors via `last_error_ptr`/`last_error_len`.

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

👑 Win:                   963174
💸 Lose:                  105684
🤝 Tie:                     1332
🧮 You win:            90.000275%
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

Timings below are whole-process runs of the release binary, measured with
[hyperfine](https://github.com/sharkdp/hyperfine) (8 runs each, ~0.5ms
process startup included):

```bash
cargo build --release
hyperfine "./target/release/poker-solver 2c6s8s AsAd"
```

| Test Case                                    | Game Code        | Solve Time       | Possibility |
| -------------------------------------------- | ---------------- | ---------------- | ----------- |
| Both hands revealed, Flop                    | AsAd,KsKd,2s3s7s | 0.6ms ± 0.1ms    | 990         |
| All community cards revealed                 | AsAd,,2s3s4s5s6s | 0.6ms ± 0.2ms    | 990         |
| Turn                                         | AsAd,,2s3s4s5s   | 4.3ms ± 0.1ms    | 45540       |
| Early royal straight-flush                   | KsAs,,TsJsQs     | 96.9ms ± 6.5ms   | 1070190     |
| Early quad                                   | AsAc,,AdAhKs     | 96.2ms ± 2.6ms   | 1070190     |
| Early full-house                             | AsAd,,2c2s2d     | 102.4ms ± 1.4ms  | 1070190     |
| Early pair of Ace, +1 card to flush/straight | AsAd,,2s3s4s     | 104.3ms ± 1.5ms  | 1070190     |
| Early pair of Ace                            | AsAd,,2c6s8s     | 109.4ms ± 2.8ms  | 1070190     |
| High cards                                   | TdQh,,2c6s8s     | 110.1ms ± 1.1ms  | 1070190     |
| Low cards                                    | 6s2h,,8cTdQh     | 110.9ms ± 1.6ms  | 1070190     |

- Blind case (flop, both villain cards hidden) enumerates all 1,070,190 outcomes in ~110ms
- With `wasm` version, expect ~2x slower

### Tests

Run the correctness test suite with

```bash
cargo test
```

Run the [criterion](https://github.com/bheisler/criterion.rs) benchmark suite with

```bash
cargo bench
```
