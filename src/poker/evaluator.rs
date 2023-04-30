use crate::poker::hand::RANK_COUNT;
use crate::poker::hand::SUIT_COUNT;
use crate::poker::Hand;

use std::cmp::min;
use std::cmp::Ordering;

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
        self.trip_hand = (0..RANK_COUNT)
            .rev()
            .map(|r| {
                suit3
                    .iter()
                    .map(|&mask| (mask as i64) << (r * SUIT_COUNT))
                    .collect()
            })
            .collect();
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
    fn get_rank_in(hand: &Hand, pool: &[Vec<i64>]) -> Option<(i64, usize)> {
        for (rank, arr) in pool.iter().enumerate() {
            if let Some(pattern) = arr.iter().find(|&&pattern| hand.matches(pattern)) {
                return Some((*pattern, rank));
            }
        }
        None
    }

    // major rank, minor rank, matched pattern
    // same as `get_strongest_5`, but has early return path
    fn get_stronger_than(&self, hand: &Hand, target: usize) -> (usize, usize, i64) {
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
            if major_rank > target {
                break;
            }
            if let Some((pattern, minor_rank)) = Self::get_rank_in(hand, pool) {
                return (major_rank, minor_rank, pattern);
            }
        }
        (pools.len(), 0, 0)
    }

    // major rank, minor rank, matched pattern
    fn get_strongest_5(&self, hand: &Hand) -> (usize, usize, i64) {
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
                return (major_rank, minor_rank, pattern);
            }
        }
        (pools.len(), 0, 0)
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

    pub async fn compare(&self, a: Hand, b: Hand) -> Ordering {
        let (rank_major_a, rank_minor_a, pattern_a) = self.get_strongest_5(&a);
        // let (rank_major_b, rank_minor_b, pattern_b) = self.get_strongest_5(&b);
        let (rank_major_b, rank_minor_b, pattern_b) = self.get_stronger_than(&b, rank_major_a);
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
        let k = (5 - min(5, pattern_a.count_ones())) as usize;
        let mut a = a;
        let mut b = b;
        Self::compare_high_card(
            a.remove(pattern_a).retain_highest_card(k),
            b.remove(pattern_b).retain_highest_card(k),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_flush_check() {
        let evaluator = Evaluator::new();
        let input = Hand::from_string("AsKsQsJsTs");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (0, 0, _)));
        let input = Hand::from_string("KsQsJsTs9s");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (0, 1, _)));
    }

    #[test]
    fn quad_check() {
        let evaluator = Evaluator::new();
        let input = Hand::from_string("AsAcAdAh2s");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (1, 0, _)));
        let input = Hand::from_string("KsKcKdKh6d");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (1, 1, _)));
    }
    #[test]
    fn full_house_check() {
        let evaluator = Evaluator::new();
        let input = Hand::from_string("AsAcAdKhKs");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (2, 0, _)));
        let input = Hand::from_string("AsAcAdQhQs");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (2, 1, _)));
    }
    #[test]
    fn flush_check() {
        let evaluator = Evaluator::new();
        let input = Hand::from_string("As2s6sTs4s");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (3, _, _)));
    }
    #[test]
    fn straight_check() {
        let evaluator = Evaluator::new();
        let input = Hand::from_string("As2c3d4h5d");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (4, _, _)));
        let input = Hand::from_string("2c3d4h5d6s");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (4, _, _)));
    }
    #[test]
    fn trip_check() {
        let evaluator = Evaluator::new();
        let input = Hand::from_string("AsAcAd");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (5, 0, _)));
        let input = Hand::from_string("KsKcKd");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (5, 1, _)));
    }
    #[test]
    fn pair2_check() {
        let evaluator = Evaluator::new();
        let input = Hand::from_string("AsAcKdKh");
        let ouput = evaluator.get_strongest_5(&input);
        assert!(matches!(ouput, (6, 0, _)));
    }
    #[test]
    fn pair_check() {
        let evaluator = Evaluator::new();
        let input = Hand::from_string("AsAc");
        let output = evaluator.get_strongest_5(&input);
        matches!(output, (7, _, _));
        let input = Hand::from_string("2s2c");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (7, _, _)));
    }

    #[test]
    fn high_card_check() {
        let evaluator = Evaluator::new();
        let input = Hand::from_string("AsKc5s8d9d");
        let output = evaluator.get_strongest_5(&input);
        assert!(matches!(output, (8, _, _)));
    }

    #[tokio::test]
    async fn vs_aaaa_kkkk() {
        let evaluator = Evaluator::new();
        let output = evaluator.compare(
            Hand::from_string("AsAcKdKhKsAdAh"),
            Hand::from_string("AsAcKdKhKsKc3d"),
        );
        assert_eq!(output.await, Ordering::Greater);
    }

    #[tokio::test]
    async fn vs_aaaaq_aaaak() {
        let evaluator = Evaluator::new();
        let output = evaluator.compare(
            Hand::from_string("AsAcAdAhQh3s4s"),
            Hand::from_string("AsAcAdAhQhKh3d"),
        );
        assert_eq!(output.await, Ordering::Less);
    }

    #[tokio::test]
    async fn vs_34567_flush() {
        let evaluator = Evaluator::new();
        let output = evaluator.compare(
            Hand::from_string("5d6d7hJd4c3sJc"),
            Hand::from_string("5d6d7hJd4cTdKd"),
        );
        assert_eq!(output.await, Ordering::Less);
    }

    #[tokio::test]
    async fn vs_34567_56789() {
        let evaluator = Evaluator::new();
        let output = evaluator.compare(
            Hand::from_string("5s6d7h3d4cTdJc"),
            Hand::from_string("5s6d7h9c8cTdJc"),
        );
        assert_eq!(output.await, Ordering::Less);
    }

    #[tokio::test]
    async fn vs_333kk_333kk() {
        let evaluator = Evaluator::new();
        let output = evaluator.compare(
            Hand::from_string("3s3d3hKdKc6d9c"),
            Hand::from_string("3s3d3hKhKs6d9c"),
        );
        assert_eq!(output.await, Ordering::Equal);
    }

    #[tokio::test]
    async fn vs_pair_pair2() {
        let evaluator = Evaluator::new();
        let output = evaluator.compare(
            Hand::from_string("2s4d5h8dTc4d5c"),
            Hand::from_string("2s4d5h8hTs6d2c"),
        );
        assert_eq!(output.await, Ordering::Greater);
    }

    #[tokio::test]
    async fn vs_pair_high_card() {
        let evaluator = Evaluator::new();
        let output = evaluator.compare(
            Hand::from_string("2s4d5h8dTcAdKc"),
            Hand::from_string("2s4d5h8hTs6d2c"),
        );
        assert_eq!(output.await, Ordering::Less);
    }

    #[tokio::test]
    async fn vs_high_card() {
        let evaluator = Evaluator::new();
        let output = evaluator.compare(
            Hand::from_string("2s4d5h8dTcAdKc"),
            Hand::from_string("2s4d5h8hTs6d9c"),
        );
        assert_eq!(output.await, Ordering::Greater);
    }

}
