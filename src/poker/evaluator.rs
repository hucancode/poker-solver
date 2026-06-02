use crate::poker::Hand;
use std::cmp::Ordering;

const RANK_COUNT: u32 = 13;

fn pack_suit(mask: u64, s: u32) -> u16 {
    let mut r = 0u16;
    for i in 0..RANK_COUNT {
        if (mask >> (i * 4 + s)) & 1 != 0 {
            r |= 1u16 << i;
        }
    }
    r
}

fn straight_top(rmask: u16) -> i32 {
    let s = rmask & (rmask >> 1) & (rmask >> 2) & (rmask >> 3) & (rmask >> 4);
    if s != 0 {
        return 15 - s.leading_zeros() as i32 + 4;
    }
    if (rmask & 0xF) == 0xF && (rmask & (1 << 12)) != 0 {
        return 3;
    }
    -1
}

fn top_n(rmask: u16, n: u32) -> u32 {
    let mut result = 0u32;
    let mut r = rmask;
    let mut taken = 0u32;
    while taken < n && r != 0 {
        let hi = 15 - r.leading_zeros();
        result = (result << 4) | hi;
        r &= !(1u16 << hi);
        taken += 1;
    }
    while taken < n {
        result <<= 4;
        taken += 1;
    }
    result
}

fn top_n_excl(rmask: u16, exclude: u16, n: u32) -> u32 {
    top_n(rmask & !exclude, n)
}

pub fn eval(mask: u64) -> u32 {
    let s0 = pack_suit(mask, 0);
    let s1 = pack_suit(mask, 1);
    let s2 = pack_suit(mask, 2);
    let s3 = pack_suit(mask, 3);
    let rmask = s0 | s1 | s2 | s3;

    let flush_mask: u16 = if s0.count_ones() >= 5 {
        s0
    } else if s1.count_ones() >= 5 {
        s1
    } else if s2.count_ones() >= 5 {
        s2
    } else if s3.count_ones() >= 5 {
        s3
    } else {
        0
    };

    if flush_mask != 0 {
        let sf = straight_top(flush_mask);
        if sf >= 0 {
            return (8u32 << 24) | sf as u32;
        }
    }

    let mut quad: i32 = -1;
    let mut trips: [i32; 2] = [-1, -1];
    let mut pairs: [i32; 3] = [-1, -1, -1];
    for i in (0..RANK_COUNT as i32).rev() {
        let cnt = ((mask >> (i as u32 * 4)) & 0xF).count_ones();
        if cnt == 4 {
            quad = i;
        } else if cnt == 3 {
            if trips[0] < 0 {
                trips[0] = i;
            } else if trips[1] < 0 {
                trips[1] = i;
            }
        } else if cnt == 2 {
            if pairs[0] < 0 {
                pairs[0] = i;
            } else if pairs[1] < 0 {
                pairs[1] = i;
            } else if pairs[2] < 0 {
                pairs[2] = i;
            }
        }
    }

    if quad >= 0 {
        let kicker = top_n_excl(rmask, 1u16 << quad, 1);
        return (7u32 << 24) | ((quad as u32) << 4) | kicker;
    }

    if trips[0] >= 0 {
        let mut pair_rank: i32 = -1;
        if trips[1] >= 0 {
            pair_rank = trips[1];
        }
        if pairs[0] >= 0 && pairs[0] > pair_rank {
            pair_rank = pairs[0];
        }
        if pair_rank >= 0 {
            return (6u32 << 24) | ((trips[0] as u32) << 4) | pair_rank as u32;
        }
    }

    if flush_mask != 0 {
        return (5u32 << 24) | top_n(flush_mask, 5);
    }

    let st = straight_top(rmask);
    if st >= 0 {
        return (4u32 << 24) | st as u32;
    }

    if trips[0] >= 0 {
        let k = top_n_excl(rmask, 1u16 << trips[0], 2);
        return (3u32 << 24) | ((trips[0] as u32) << 8) | k;
    }

    if pairs[0] >= 0 && pairs[1] >= 0 {
        let excl = (1u16 << pairs[0]) | (1u16 << pairs[1]);
        let k = top_n_excl(rmask, excl, 1);
        return (2u32 << 24) | ((pairs[0] as u32) << 8) | ((pairs[1] as u32) << 4) | k;
    }

    if pairs[0] >= 0 {
        let k = top_n_excl(rmask, 1u16 << pairs[0], 3);
        return (1u32 << 24) | ((pairs[0] as u32) << 12) | k;
    }

    top_n(rmask, 5)
}

#[derive(Default)]
pub struct Evaluator;

impl Evaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn compare(&self, a: &Hand, b: &Hand) -> Ordering {
        eval(a.mask).cmp(&eval(b.mask))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat(mask: u64) -> u32 {
        eval(mask) >> 24
    }

    #[test]
    fn straight_flush_cat() {
        assert_eq!(cat(Hand::from_string("AsKsQsJsTs").mask), 8);
        assert_eq!(cat(Hand::from_string("KsQsJsTs9s").mask), 8);
    }
    #[test]
    fn quad_cat() {
        assert_eq!(cat(Hand::from_string("AsAcAdAh2s").mask), 7);
    }
    #[test]
    fn full_house_cat() {
        assert_eq!(cat(Hand::from_string("AsAcAdKhKs").mask), 6);
    }
    #[test]
    fn flush_cat() {
        assert_eq!(cat(Hand::from_string("As2s6sTs4s").mask), 5);
    }
    #[test]
    fn straight_cat() {
        assert_eq!(cat(Hand::from_string("As2c3d4h5d").mask), 4);
        assert_eq!(cat(Hand::from_string("2c3d4h5d6s").mask), 4);
        assert_eq!(cat(Hand::from_string("Ts9c8d7h6d").mask), 4);
    }
    #[test]
    fn trip_cat() {
        assert_eq!(cat(Hand::from_string("AsAcAd2h5d").mask), 3);
    }
    #[test]
    fn two_pair_cat() {
        assert_eq!(cat(Hand::from_string("AsAcKdKh2s").mask), 2);
    }
    #[test]
    fn pair_cat() {
        assert_eq!(cat(Hand::from_string("AsAc5h8d9c").mask), 1);
    }
    #[test]
    fn high_card_cat() {
        assert_eq!(cat(Hand::from_string("AsKc5s8d9d").mask), 0);
    }

    #[test]
    fn aaaa_vs_kkkk() {
        let e = Evaluator::new();
        let out = e.compare(
            &Hand::from_string("AsAcKdKhKsAdAh"),
            &Hand::from_string("AsAcKdKhKsKc3d"),
        );
        assert_eq!(out, Ordering::Greater);
    }

    #[test]
    fn aaaaq_vs_aaaak() {
        let e = Evaluator::new();
        let out = e.compare(
            &Hand::from_string("AsAcAdAhQh3s4s"),
            &Hand::from_string("AsAcAdAhQhKh3d"),
        );
        assert_eq!(out, Ordering::Less);
    }

    #[test]
    fn straight_vs_flush() {
        let e = Evaluator::new();
        let out = e.compare(
            &Hand::from_string("5d6d7hJd4c3sJc"),
            &Hand::from_string("5d6d7hJd4cTdKd"),
        );
        assert_eq!(out, Ordering::Less);
    }

    #[test]
    fn _34567_vs_56789() {
        let e = Evaluator::new();
        let out = e.compare(
            &Hand::from_string("5s6d7h3d4cTdJc"),
            &Hand::from_string("5s6d7h9c8cTdJc"),
        );
        assert_eq!(out, Ordering::Less);
    }

    #[test]
    fn _333kk_vs_333kk() {
        let e = Evaluator::new();
        let out = e.compare(
            &Hand::from_string("3s3d3hKdKc6d9c"),
            &Hand::from_string("3s3d3hKhKs6d9c"),
        );
        assert_eq!(out, Ordering::Equal);
    }

    #[test]
    fn pair_vs_pair2() {
        let e = Evaluator::new();
        let out = e.compare(
            &Hand::from_string("2s4d5h8dTc4d5c"),
            &Hand::from_string("2s4d5h8hTs6d2c"),
        );
        assert_eq!(out, Ordering::Greater);
    }

    #[test]
    fn pair_vs_high_card() {
        let e = Evaluator::new();
        let out = e.compare(
            &Hand::from_string("2s4d5h8dTcAdKc"),
            &Hand::from_string("2s4d5h8hTs6d2c"),
        );
        assert_eq!(out, Ordering::Less);
    }

    #[test]
    fn high_card_vs_high_card() {
        let e = Evaluator::new();
        let out = e.compare(
            &Hand::from_string("2s4d5h8dTcAdKc"),
            &Hand::from_string("2s4d5h8hTs6d9c"),
        );
        assert_eq!(out, Ordering::Greater);
    }

    #[test]
    fn wheel_straight() {
        let e = Evaluator::new();
        let wheel = Hand::from_string("As2c3d4h5d");
        let six_high = Hand::from_string("2s3c4d5h6h");
        assert_eq!(e.compare(&wheel, &six_high), Ordering::Less);
    }
}
