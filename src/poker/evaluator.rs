use crate::poker::hand::RANK_COUNT;
use crate::poker::hand::SUIT_COUNT;
use crate::poker::Hand;

use std::cmp::min;
use std::cmp::Ordering;
use std::collections::HashMap;

// pattern for 2 cards of the same rank
const SAME_RANK_2X: [i64; 6] = [0b1100, 0b1010, 0b1001, 0b0110, 0b0101, 0b0011];
// pattern for 3 cards of the same rank
const SAME_RANK_3X: [i64; 4] = [0b1110, 0b1101, 0b1011, 0b0111];

#[derive(Default)]
pub struct Evaluator {
    straight_flush_hand: Vec<Vec<i64>>,
    quad_hand: Vec<Vec<i64>>,
    full_house_hand: Vec<Vec<i64>>,
    flush_hand: Vec<Vec<i64>>,
    straight_hand: Vec<Vec<i64>>,
    trip_hand: Vec<Vec<i64>>,
    two_pair_hand: Vec<Vec<i64>>,
    pair_hand: Vec<Vec<i64>>,
    hash_map: HashMap<i64, (usize, usize, i64)>,
}

impl Evaluator {
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
                                let mask = [r0, r1, r2, r3, r4]
                                    .into_iter()
                                    .map(|x| x * SUIT_COUNT + suit)
                                    .map(|x| 1 << x)
                                    .sum();
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
        self.quad_hand = (0..RANK_COUNT)
            .rev()
            .map(|r| 0b1111 << (r * SUIT_COUNT))
            .map(|x| vec![x])
            .collect();
    }
    fn build_full_house_hand(&mut self) {
        for r1 in (0..RANK_COUNT).rev() {
            for r2 in (0..RANK_COUNT).rev() {
                if r1 == r2 {
                    continue;
                }
                let mut arr: Vec<i64> = Vec::new();
                for s1 in &SAME_RANK_3X {
                    for s2 in &SAME_RANK_2X {
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
        self.trip_hand = (0..RANK_COUNT)
            .rev()
            .map(|r| {
                SAME_RANK_3X
                    .iter()
                    .map(|&mask| (mask as i64) << (r * SUIT_COUNT))
                    .collect()
            })
            .collect();
    }
    fn build_two_pair_hand(&mut self) {
        for r1 in (0..RANK_COUNT).rev() {
            for r2 in (0..r1).rev() {
                let mut arr: Vec<i64> = Vec::new();
                for s1 in &SAME_RANK_2X {
                    for s2 in &SAME_RANK_2X {
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
        for rank in (0..RANK_COUNT).rev() {
            let mut arr: Vec<i64> = Vec::new();
            for suit in &SAME_RANK_2X {
                let mask: i64 = suit << (rank * SUIT_COUNT);
                arr.push(mask);
            }
            self.pair_hand.push(arr);
        }
    }

    // pattern, order
    fn get_rank_in(hand: &Hand, pool: &[Vec<i64>]) -> Option<(i64, usize)> {
        for (rank, arr) in pool.iter().enumerate() {
            if let Some(pattern) = arr.iter().find(|&&pattern| hand.matches(pattern)) {
                return Some((*pattern, rank));
            }
        }
        None
    }

    // major rank, minor rank, matched pattern
    fn get_strongest_5(&mut self, hand: &Hand) -> (usize, usize, i64) {
        if let Some(&res) = self.hash_map.get(&hand.mask) {
            return res;
        }
        let key = hand.mask;
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
        for (major_rank, pool) in pools.into_iter().enumerate() {
            if let Some((pattern, minor_rank)) = Self::get_rank_in(hand, pool) {
                let k = (5 - min(5, pattern.count_ones())) as usize;
                let res = (
                    major_rank,
                    minor_rank,
                    hand.get_highest_card_not_in(pattern, k),
                );
                self.hash_map.insert(key, res);
                return res;
            }
        }
        let res = (pools.len(), 0, hand.get_highest_card(5));
        self.hash_map.insert(key, res);
        res
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
        ret.hash_map = HashMap::new();
        ret
    }

    fn compare_high_card(a: &Hand, b: &Hand) -> Ordering {
        for rank in (0..RANK_COUNT).rev() {
            let a_matched = a.has_rank(rank);
            let b_matched = b.has_rank(rank);
            if a_matched == b_matched {
                continue;
            }
            if a_matched {
                return Ordering::Greater;
            }
            if b_matched {
                return Ordering::Less;
            }
        }
        Ordering::Equal
    }

    pub fn compare(&mut self, a: &Hand, b: &Hand) -> Ordering {
        let (rank_major_a, rank_minor_a, pattern_a) = self.get_strongest_5(a);
        let (rank_major_b, rank_minor_b, pattern_b) = self.get_strongest_5(b);
        if rank_major_a < rank_major_b {
            return Ordering::Greater;
        }
        if rank_major_b < rank_major_a {
            return Ordering::Less;
        }
        if rank_minor_a < rank_minor_b {
            return Ordering::Greater;
        }
        if rank_minor_b < rank_minor_a {
            return Ordering::Less;
        }
        Self::compare_high_card(&Hand::from(pattern_a), &Hand::from(pattern_b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_array_check() {
        let evaluator = Evaluator::new();
        assert_eq!(evaluator.straight_flush_hand.iter().flatten().count(), 40);
        assert_eq!(evaluator.quad_hand.iter().flatten().count(), 13);
        assert_eq!(evaluator.full_house_hand.iter().flatten().count(), 3744);
        assert_eq!(evaluator.flush_hand.iter().flatten().count(), 5148);
        assert_eq!(evaluator.straight_hand.iter().flatten().count(), 30680);
        assert_eq!(evaluator.trip_hand.iter().flatten().count(), 52);
        assert_eq!(evaluator.two_pair_hand.iter().flatten().count(), 2808);
        assert_eq!(evaluator.pair_hand.iter().flatten().count(), 78);
    }
    #[test]
    fn straight_flush_check() {
        let mut evaluator = Evaluator::new();
        let input = &Hand::from("AsKsQsJsTs");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (0, 0, _)));
        let input = &Hand::from("KsQsJsTs9s");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (0, 1, _)));
    }
    #[test]
    fn quad_check() {
        let mut evaluator = Evaluator::new();
        let input = &Hand::from("AsAcAdAh2s");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (1, 0, _)));
        let input = &Hand::from("KsKcKdKh6d");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (1, 1, _)));
    }
    #[test]
    fn full_house_check() {
        let mut evaluator = Evaluator::new();
        let input = &Hand::from("AsAcAdKhKs");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (2, 0, _)));
        let input = &Hand::from("AsAcAdQhQs");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (2, 1, _)));
    }
    #[test]
    fn flush_check() {
        let mut evaluator = Evaluator::new();
        let input = &Hand::from("As2s6sTs4s");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (3, _, _)));
    }
    #[test]
    fn straight_check() {
        let mut evaluator = Evaluator::new();
        let input = &Hand::from("As2c3d4h5d");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (4, _, _)));
        let input = &Hand::from("2c3d4h5d6s");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (4, _, _)));
    }
    #[test]
    fn trip_check() {
        let mut evaluator = Evaluator::new();
        let input = &Hand::from("AsAcAd");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (5, 0, _)));
        let input = &Hand::from("KsKcKd");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (5, 1, _)));
    }
    #[test]
    fn pair2_check() {
        let mut evaluator = Evaluator::new();
        let input = &Hand::from("AsAcKdKh");
        let ouput = evaluator.get_strongest_5(&input);
        assert!(matches!(ouput, (6, 0, _)));
    }
    #[test]
    fn pair_check() {
        let mut evaluator = Evaluator::new();
        let input = &Hand::from("AsAc");
        let output = evaluator.get_strongest_5(&input);
        matches!(output, (7, _, _));
        let input = &Hand::from("2s2c");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (7, _, _)));
    }
    #[test]
    fn high_card_check() {
        let mut evaluator = Evaluator::new();
        let input = &Hand::from("AsKc5s8d9d");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (8, _, _)));
    }
    #[test]
    fn aaaa_vs_kkkk() {
        let mut evaluator = Evaluator::new();
        let output =
            evaluator.compare(&Hand::from("AsAcKdKhKsAdAh"), &Hand::from("AsAcKdKhKsKc3d"));
        assert_eq!(output, Ordering::Greater);
    }
    #[test]
    fn aaaaq_vs_aaaak() {
        let mut evaluator = Evaluator::new();
        let output =
            evaluator.compare(&Hand::from("AsAcAdAhQh3s4s"), &Hand::from("AsAcAdAhQhKh3d"));
        assert_eq!(output, Ordering::Less);
    }
    #[test]
    fn _34567_vs_flush() {
        let mut evaluator = Evaluator::new();
        let output =
            evaluator.compare(&Hand::from("5d6d7hJd4c3sJc"), &Hand::from("5d6d7hJd4cTdKd"));
        assert_eq!(output, Ordering::Less);
    }
    #[test]
    fn _34567_vs_56789() {
        let mut evaluator = Evaluator::new();
        let output =
            evaluator.compare(&Hand::from("5s6d7h3d4cTdJc"), &Hand::from("5s6d7h9c8cTdJc"));
        assert_eq!(output, Ordering::Less);
    }
    #[test]
    fn _333kk_vs_333kk() {
        let mut evaluator = Evaluator::new();
        let output =
            evaluator.compare(&Hand::from("3s3d3hKdKc6d9c"), &Hand::from("3s3d3hKhKs6d9c"));
        assert_eq!(output, Ordering::Equal);
    }
    #[test]
    fn pair_vs_pair2() {
        let mut evaluator = Evaluator::new();
        let output =
            evaluator.compare(&Hand::from("2s4d5h8dTc4d5c"), &Hand::from("2s4d5h8hTs6d2c"));
        assert_eq!(output, Ordering::Greater);
    }
    #[test]
    fn pair_vs_high_card() {
        let mut evaluator = Evaluator::new();
        let output =
            evaluator.compare(&Hand::from("2s4d5h8dTcAdKc"), &Hand::from("2s4d5h8hTs6d2c"));
        assert_eq!(output, Ordering::Less);
    }
    #[test]
    fn high_card_vs_high_card() {
        let mut evaluator = Evaluator::new();
        let output =
            evaluator.compare(&Hand::from("2s4d5h8dTcAdKc"), &Hand::from("2s4d5h8hTs6d9c"));
        assert_eq!(output, Ordering::Greater);
    }
}
