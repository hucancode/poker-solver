use std::collections::HashMap;
pub const RANK_COUNT: i64 = 13;
pub const SUIT_COUNT: i64 = 4;

#[derive(Debug, Default, Clone, Copy)]
pub struct Hand {
    pub mask: i64,
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

    pub fn from_mask(x: i64) -> Self {
        Self { mask: x }
    }

    pub fn from_string(hand: &str) -> Self {
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
        return ret;
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
        return ret;
    }

    pub fn pretify(hand: &str) -> String {
        let map = HashMap::from([
            ('T', String::from("10")),
            ('s', String::from("♠")),
            ('c', String::from("♣")),
            ('d', String::from("♦")),
            ('h', String::from("♥")),
        ]);
        return hand.chars().fold(String::new(), |acc, x| {
            acc + map.get(&x).unwrap_or(&String::from(x))
        });
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
    fn prettify() {
        let output = Hand::pretify("QsQcQhKsKd");
        let expected = "Q♠Q♣Q♥K♠K♦";
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_qqqkk() {
        let output = Hand::from_string("QsQcQhKsKc");
        let expected = [40, 41, 43, 44, 45].iter().map(|x| 1 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn parse_23456() {
        let output = Hand::from_string("2s3c4d5h6h");
        let expected = [0, 5, 10, 15, 19].iter().map(|x| 1 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn parse_aaaaq() {
        let output = Hand::from_string("AsAcAdAhQh");
        let expected = [48, 49, 50, 51, 43].iter().map(|x| 1 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn highcard_23456() {
        let mut output = Hand::from_mask([0, 5, 10, 15, 19].iter().map(|x| 1 << x).sum());
        output.retain_highest_card(2);
        let expected = [15, 19].iter().map(|x| 1 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn highcard_aaaaq() {
        let mut output = Hand::from_mask([48, 49, 50, 51, 43].iter().map(|x| 1 << x).sum());
        output.retain_highest_card(3);
        let expected = [49, 50, 51].iter().map(|x| 1 << x).sum();
        assert_eq!(output.mask, expected);
    }
}
