pub const RANK_COUNT: i64 = 13;
pub const SUIT_COUNT: i64 = 4;

#[derive(Debug, Default, Clone, Copy)]
pub struct Hand {
    pub mask: i64,
}

impl From<&str> for Hand {
    fn from(hand: &str) -> Self {
        let ranks = String::from("23456789TJQKA");
        let suits = String::from("scdh");
        let mut ret: i64 = 0;
        let mut i = hand.chars();
        while let Some((cr, cs)) = i.next().zip(i.next()) {
            let r = ranks.find(cr).unwrap_or_default();
            let s = suits.find(cs).unwrap_or_default();
            ret |= 1 << (r * SUIT_COUNT as usize + s);
        }
        Self { mask: ret }
    }
}

impl From<i64> for Hand {
    fn from(x: i64) -> Self {
        Self { mask: x }
    }
}

impl Hand {
    pub fn len(&self) -> u32 {
        self.mask.count_ones()
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            mask: self.mask | other.mask,
        }
    }

    pub fn get_highest_card_not_in(&self, pattern: i64, count: usize) -> i64 {
        let mask = self.mask & (!pattern);
        let mut count = count;
        let mut ret: i64 = 0;
        let mut k: i64 = 1 << (RANK_COUNT * SUIT_COUNT - 1);
        while k != 0 && count != 0 {
            if (mask & k) != 0 {
                ret |= k;
                count -= 1;
            }
            k >>= 1;
        }
        ret
    }

    pub fn get_highest_card(&self, count: usize) -> i64 {
        let mut count = count;
        let mut ret: i64 = 0;
        let mut k: i64 = 1 << (RANK_COUNT * SUIT_COUNT - 1);
        while k != 0 && count != 0 {
            if (self.mask & k) != 0 {
                ret |= k;
                count -= 1;
            }
            k >>= 1;
        }
        ret
    }

    pub fn overlap(&self, other: &Self) -> bool {
        self.mask & other.mask != 0
    }

    pub fn matches(&self, pattern: i64) -> bool {
        self.mask & pattern == pattern
    }

    pub fn has_rank(&self, rank: i64) -> bool {
        (self.mask & 0b1111 << (rank * SUIT_COUNT)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_highest_card() {
        let hand1 = Hand::from("2s3c4hKsKcAdAs");
        let hand2 = Hand::from("KcAdAs");
        let hand3 = Hand::from("4hKs");
        assert_eq!(hand1.get_highest_card(3), hand2.mask);
        assert_eq!(hand1.get_highest_card_not_in(hand2.mask, 2), hand3.mask);
    }
    #[test]
    fn merge() {
        let hand1 = Hand::from("2s3c4hKsKc");
        let hand2 = Hand::from("KcQsAcTs");
        let hand3 = Hand::from("2s3c4hKsKcQsAcTs");
        assert_eq!(hand1.merge(&hand2).mask, hand3.mask);
    }
    #[test]
    fn overlap() {
        let hand1 = Hand::from("2s3c4hKsKc");
        let hand2 = Hand::from("KcQsAcTs");
        let hand3 = Hand::from("2d3d4dKdQd");
        assert!(hand1.overlap(&hand2));
        assert!(!hand1.overlap(&hand3));
        assert!(!hand2.overlap(&hand3));
    }
    #[test]
    fn has_rank() {
        let hand = Hand::from("2s3c4hKsKc");
        assert_eq!(hand.has_rank(0), true);
        assert_eq!(hand.has_rank(8), false);
        assert_eq!(hand.has_rank(12), false);
    }
    #[test]
    fn parse_qqqkk() {
        let output = Hand::from("QsQcQhKsKc");
        let expected = [40, 41, 43, 44, 45].into_iter().map(|x| 1 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn parse_23456() {
        let output = Hand::from("2s3c4d5h6h");
        let expected = [0, 5, 10, 15, 19].into_iter().map(|x| 1 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn parse_aaaaq() {
        let output = Hand::from("AsAcAdAhQh");
        let expected = [48, 49, 50, 51, 43].into_iter().map(|x| 1 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn highcard_23456() {
        let output = Hand::from([0, 5, 10, 15, 19].into_iter().map(|x| 1 << x).sum::<i64>())
            .get_highest_card(2);
        let expected = [15, 19].iter().map(|x| 1 << x).sum();
        assert_eq!(output, expected);
    }
    #[test]
    fn highcard_aaaaq() {
        let output = Hand::from(
            [48, 49, 50, 51, 43]
                .into_iter()
                .map(|x| 1 << x)
                .sum::<i64>(),
        )
        .get_highest_card(3);
        let expected = [49, 50, 51].iter().map(|x| 1 << x).sum();
        assert_eq!(output, expected);
    }
}
