use crate::hand;
use crate::hand::RANK_COUNT;
use crate::hand::SUIT_COUNT;

use std::cmp::min;

pub enum CompareResult {
    AWin,
    BWin,
    Tie,
}

#[derive(Default)]
pub struct HandEvaluator {
    straight_flush_hand: Vec<Vec<i64>>,
    quad_hand: Vec<Vec<i64>>,
    full_house_hand: Vec<Vec<i64>>,
    flush_hand: Vec<Vec<i64>>,
    straight_hand: Vec<Vec<i64>>,
    trip_hand: Vec<Vec<i64>>,
    two_pair_hand: Vec<Vec<i64>>,
    pair_hand: Vec<Vec<i64>>,
}

impl HandEvaluator {
    fn build_straight_hand(&mut self) {
        for rank in (3..RANK_COUNT).rev() {
            let mut arr: Vec<i64> = Vec::new();
            let mut arr_st: Vec<i64> = Vec::new();
            const SUIT_5: i64 = 0b11 << (2 * 5);
            for suit in 0..SUIT_5 {
                let mut mask: i64 = 0;
                let card = rank * SUIT_COUNT + (suit & 0b11);
                mask |= 1 << card;
                let card = (rank - 1) * SUIT_COUNT + (suit >> 2 & 0b11);
                mask |= 1 << card;
                let card = (rank - 2) * SUIT_COUNT + (suit >> 4 & 0b11);
                mask |= 1 << card;
                let card = (rank - 3) * SUIT_COUNT + (suit >> 6 & 0b11);
                mask |= 1 << card;

                let low_ace = rank == 3;
                if low_ace {
                    let card = 12 * SUIT_COUNT + (suit >> 8 & 0b11);
                    mask |= 1 << card;
                } else {
                    let card = (rank - 4) * SUIT_COUNT + (suit >> 8 & 0b11);
                    mask |= 1 << card;
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

    // pattern, order
    fn get_rank_in(mask: &i64, pool: &[Vec<i64>]) -> Option<(i64, i32)> {
        for (rank, arr) in pool.iter().enumerate() {
            if let Some(pattern) = arr.iter().find(|&&x| (mask & x) == x) {
                return Some((*pattern, rank as i32));
            }
        }
        None
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
            if let Some((pattern, minor_rank)) = Self::get_rank_in(mask, pool) {
                return (major_rank, minor_rank, pattern);
            }
            major_rank += 1;
        }
        (major_rank, 0, 0)
    }

    pub fn new() -> Self {
        let mut ret: Self = Default::default();
        ret.build_straight_hand();
        ret.build_flush_hand();
        ret.build_quad_hand();
        ret.build_full_house_hand();
        ret.build_trip_hand();
        ret.build_two_pair_hand();
        ret.build_pair_hand();
        ret
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
        CompareResult::Tie
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
        let hand_a = hand::get_highest_card(hand_a, k);
        let hand_b = hand_b & (!pattern_b);
        let hand_b = hand::get_highest_card(hand_b, k);
        Self::compare_high_card(&hand_a, &hand_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hand;

    #[test]
    fn vs_aaaaq_aaaak() {
        let evaluator = HandEvaluator::new();
        let output = evaluator.compare(
            hand::from_string("AsAcAdAhQh3s4s"),
            hand::from_string("AsAcAdAhQhKh3d"),
        );
        assert!(matches!(output, CompareResult::BWin));
    }

    #[test]
    fn vs_34567_56789() {
        let evaluator = HandEvaluator::new();
        let output = evaluator.compare(
            hand::from_string("5s6d7h3d4cTdJc"),
            hand::from_string("5s6d7h9c8cTdJc"),
        );
        assert!(matches!(output, CompareResult::BWin));
    }

    #[test]
    fn vs_333kk_333kk() {
        let evaluator = HandEvaluator::new();
        let output = evaluator.compare(
            hand::from_string("3s3d3hKdKc6d9c"),
            hand::from_string("3s3d3hKhKs6d9c"),
        );
        assert!(matches!(output, CompareResult::Tie));
    }
}
