use std::cmp::min;
use std::collections::VecDeque;

const RANK_COUNT: i64 = 13;
const SUIT_COUNT: i64 = 4;

pub enum CompareResult {
    AWin,
    BWin,
    Tie,
}

#[derive(Default)]
pub struct HandComparer {
    straight_flush_hand: Vec<Vec<i64>>,
    quad_hand: Vec<Vec<i64>>,
    full_house_hand: Vec<Vec<i64>>,
    flush_hand: Vec<Vec<i64>>,
    straight_hand: Vec<Vec<i64>>,
    trip_hand: Vec<Vec<i64>>,
    two_pair_hand: Vec<Vec<i64>>,
    pair_hand: Vec<Vec<i64>>,
}

impl HandComparer {
    fn build_straight_hand(&mut self) {
        for rank in (3..RANK_COUNT).rev() {
            let mut arr: Vec<i64> = Vec::new();
            let mut arr_st: Vec<i64> = Vec::new();
            const SUIT_5: i64 = 0b11 << (2 * 5);
            for suit in 0..SUIT_5 {
                let mut mask: i64 = 0;
                let card = rank * SUIT_COUNT + (suit & 0b11);
                mask |= 1 << card;
                let card = (rank - 1) * SUIT_COUNT + (suit << 2 & 0b11);
                mask |= 1 << card;
                let card = (rank - 2) * SUIT_COUNT + (suit << 4 & 0b11);
                mask |= 1 << card;
                let card = (rank - 3) * SUIT_COUNT + (suit << 6 & 0b11);
                mask |= 1 << card;

                let low_ace = rank == 3;
                if low_ace {
                    let card = 12 * SUIT_COUNT + (suit << 8 & 0b11);
                    mask = mask | 1 << card;
                } else {
                    let card = (rank - 4) * SUIT_COUNT + (suit << 8 & 0b11);
                    mask = mask | 1 << card;
                }

                let same_suit = suit == 0b0000000000
                    || suit == 0b0101010101
                    || suit == 0b1010101010
                    || suit == 0b1111111111;
                if same_suit {
                    arr_st.push(mask);
                } else {
                    arr.push(mask);
                }
            }
            self.straight_hand.push(arr);
            self.straight_flush_hand.push(arr_st);
        }
    }
    fn build_flush_hand(&mut self) {
        for r0 in (4..RANK_COUNT).rev() {
            for r1 in (3..r0).rev() {
                for r2 in (2..r1).rev() {
                    for r3 in (1..r2).rev() {
                        for r4 in (0..r3).rev() {
                            let mut arr: Vec<i64> = Vec::new();
                            for suit in 0..4 {
                                let mut mask: i64 = 0;
                                let card = r0 * SUIT_COUNT + suit;
                                mask |= 1 << card;
                                let card = r1 * SUIT_COUNT + suit;
                                mask |= 1 << card;
                                let card = r2 * SUIT_COUNT + suit;
                                mask |= 1 << card;
                                let card = r3 * SUIT_COUNT + suit;
                                mask |= 1 << card;
                                let card = r4 * SUIT_COUNT + suit;
                                mask |= 1 << card;
                                arr.push(mask);
                            }
                            self.flush_hand.push(arr);
                        }
                    }
                }
            }
        }
    }
    fn build_quad_hand(&mut self) {
        for rank in (0..RANK_COUNT).rev() {
            let mask: i64 = 0b1111 << (rank * SUIT_COUNT);
            self.quad_hand.push(vec![mask]);
        }
    }
    fn build_full_house_hand(&mut self) {
        let suit2 = [0b1100, 0b1010, 0b1001, 0b0110, 0b0101, 0b0011];
        let suit3 = [0b1110, 0b1101, 0b1011, 0b0111];
        for r1 in (0..RANK_COUNT).rev() {
            for r2 in (0..RANK_COUNT).rev() {
                if r1 == r2 {
                    continue;
                }
                let mut arr: Vec<i64> = Vec::new();
                for s1 in &suit3 {
                    for s2 in &suit2 {
                        let mut mask: i64 = 0;
                        mask |= s1 << (r1 * SUIT_COUNT);
                        mask |= s2 << (r2 * SUIT_COUNT);
                        arr.push(mask);
                    }
                }
                self.full_house_hand.push(arr);
            }
        }
    }
    fn build_trip_hand(&mut self) {
        let suit3 = [0b1110, 0b1101, 0b1011, 0b0111];
        for rank in (0..RANK_COUNT).rev() {
            let mut arr: Vec<i64> = Vec::new();
            for suit in &suit3 {
                let mask: i64 = suit << (rank * SUIT_COUNT);
                arr.push(mask);
            }
            self.trip_hand.push(arr);
        }
    }
    fn build_two_pair_hand(&mut self) {
        let suit2 = [0b1100, 0b1010, 0b1001, 0b0110, 0b0101, 0b0011];
        for r1 in (0..RANK_COUNT).rev() {
            for r2 in (0..r1).rev() {
                let mut arr: Vec<i64> = Vec::new();
                for s1 in &suit2 {
                    for s2 in &suit2 {
                        let mut mask: i64 = 0;
                        mask |= s1 << (r1 * SUIT_COUNT);
                        mask |= s2 << (r2 * SUIT_COUNT);
                        arr.push(mask);
                    }
                }
                self.two_pair_hand.push(arr);
            }
        }
    }
    fn build_pair_hand(&mut self) {
        let suit2 = [0b1100, 0b1010, 0b1001, 0b0110, 0b0101, 0b0011];
        for rank in (0..RANK_COUNT).rev() {
            let mut arr: Vec<i64> = Vec::new();
            for suit in &suit2 {
                let mask: i64 = suit << (rank * SUIT_COUNT);
                arr.push(mask);
            }
            self.pair_hand.push(arr);
        }
    }
    // return `count` highest bit from `mask`
    // get_highest_bit(0b10101111111, 2) => 0b10100000000
    fn get_highest_bit(mask: &i64, count: usize) -> i64 {
        let mut count = count;
        let mut ret: i64 = 0;
        let mut k: i64 = 1 << 51;
        while k != 0 && count != 0 {
            if (mask & k) != 0 {
                ret |= k;
                count -= 1;
            }
            k = k >> 1;
        }
        return ret;
    }

    // pattern, order
    fn get_rank_in(mask: &i64, pool: &Vec<Vec<i64>>) -> Option<(i64, i32)> {
        for (rank, arr) in pool.iter().enumerate() {
            for pattern in arr.iter() {
                if (mask & (*pattern)) == *pattern {
                    return Some((*pattern, rank as i32));
                }
            }
        }
        return None;
    }

    // major rank, minor rank, matched pattern
    fn get_strongest_5(&self, mask: &i64) -> (i32, i32, i64) {
        let mut major_rank = 0;
        let pools = [
            &self.straight_flush_hand,
            &self.quad_hand,
            &self.full_house_hand,
            &self.flush_hand,
            &self.straight_hand,
            &self.trip_hand,
            &self.two_pair_hand,
            &self.pair_hand,
        ];
        for pool in &pools {
            match Self::get_rank_in(mask, pool) {
                Some((pattern, minor_rank)) => {
                    return (major_rank, minor_rank, pattern);
                }
                None => {
                    major_rank += 1;
                }
            }
        }
        return (major_rank, 0, 0);
    }

    pub fn new() -> HandComparer {
        let mut ret: HandComparer = Default::default();
        ret.build_straight_hand();
        ret.build_flush_hand();
        ret.build_quad_hand();
        ret.build_full_house_hand();
        ret.build_trip_hand();
        ret.build_two_pair_hand();
        ret.build_pair_hand();
        return ret;
    }

    fn compare_high_card(hand_a: &i64, hand_b: &i64) -> CompareResult {
        for rank in (0..RANK_COUNT).rev() {
            let a_matched = (hand_a & 0b1111 << (rank * SUIT_COUNT)) != 0;
            let b_matched = (hand_b & 0b1111 << (rank * SUIT_COUNT)) != 0;
            if a_matched == b_matched {
                continue;
            }
            if a_matched {
                return CompareResult::AWin;
            }
            if b_matched {
                return CompareResult::BWin;
            }
        }
        return CompareResult::Tie;
    }

    pub fn compare(&self, hand_a: i64, hand_b: i64) -> CompareResult {
        let (rank_major_a, rank_minor_a, pattern_a) = self.get_strongest_5(&hand_a);
        let (rank_major_b, rank_minor_b, pattern_b) = self.get_strongest_5(&hand_b);
        if rank_major_a < rank_major_b {
            return CompareResult::AWin;
        }
        if rank_major_b < rank_major_a {
            return CompareResult::BWin;
        }
        if rank_minor_a < rank_minor_b {
            return CompareResult::AWin;
        }
        if rank_minor_b < rank_minor_a {
            return CompareResult::BWin;
        }
        let k = (5 - min(5, pattern_a.count_ones())) as usize;
        let hand_a = hand_a & (!pattern_a);
        let hand_a = Self::get_highest_bit(&hand_a, k);
        let hand_b = hand_b & (!pattern_b);
        let hand_b = Self::get_highest_bit(&hand_b, k);
        return Self::compare_high_card(&hand_a, &hand_b);
    }
}
pub struct HandConverter;
impl HandConverter {
    pub fn string_to_mask(hand: &String) -> i64 {
        let ranks: String = String::from("23456789TJQKA");
        let suits: String = String::from("scdh");
        let mut ret: i64 = 0;
        let mut i = hand.chars();
        while let Some((cr, cs)) = i.next().zip(i.next()) {
            let r = ranks.find(cr).unwrap_or_default();
            let s = suits.find(cs).unwrap_or_default();
            ret |= 1 << (r * SUIT_COUNT as usize + s);
        }
        return ret;
    }
}
#[derive(Default)]
pub struct Game {
    pub hand_a: i64,
    pub hand_b: i64,
    pub community: i64,
}
impl Game {
    pub fn from_string(a: &String, b: &String, c: &String) -> Game {
        Game {
            hand_a: HandConverter::string_to_mask(a),
            hand_b: HandConverter::string_to_mask(b),
            community: HandConverter::string_to_mask(c),
        }
    }
    pub fn solve(&self, comparer: &HandComparer) -> (i32, i32, i32) {
        if self.hand_a.count_ones() < 2 {
            return (0, 0, 0);
        }
        if self.community.count_ones() < 3 {
            return (0, 0, 0);
        }
        let mut win = 0;
        let mut lose = 0;
        let mut tie = 0;
        let mut q = VecDeque::new();
        q.push_back((self.hand_a, self.hand_b, self.community));
        let deck = (0..52).into_iter().map(|x| 1 << x);
        while let Some((a, b, c)) = q.pop_front() {
            let board = a | b | c;
            if board.count_ones() >= 9 {
                match comparer.compare(a | c, b | c) {
                    CompareResult::AWin => win += 1,
                    CompareResult::BWin => lose += 1,
                    CompareResult::Tie => tie += 1,
                }
                continue;
            }
            let it = deck.clone().filter(|x| x & board == 0);
            if b.count_ones() < 2 {
                let mut it = it.clone();
                while let Some(x) = it.next() {
                    let b = b | x;
                    if b.count_ones() >= 2 {
                        q.push_back((a, b, c));
                        continue;
                    }
                    let mut it = it.clone();
                    while let Some(x) = it.next() {
                        let b = b | x;
                        q.push_back((a, b, c));
                    }
                }
                continue;
            }
            if c.count_ones() < 5 {
                let mut it = it.clone();
                while let Some(x) = it.next() {
                    let c = c | x;
                    if c.count_ones() >= 5 {
                        q.push_back((a, b, c));
                        continue;
                    }
                    let mut it = it.clone();
                    while let Some(x) = it.next() {
                        let c = c | x;
                        if c.count_ones() >= 5 {
                            q.push_back((a, b, c));
                            continue;
                        }
                        let mut it = it.clone();
                        while let Some(x) = it.next() {
                            let c = c | x;
                            q.push_back((a, b, c));
                        }
                    }
                }
            }
        }
        return (win, lose, tie);
    }
}
