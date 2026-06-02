pub const SUIT_COUNT: u32 = 4;

#[derive(Debug, Default, Clone, Copy)]
pub struct Hand {
    pub mask: u64,
}

impl Hand {
    pub fn len(&self) -> u32 {
        self.mask.count_ones()
    }

    pub fn from_mask(x: u64) -> Self {
        Self { mask: x }
    }

    pub fn from_string(hand: &str) -> Self {
        const RANKS: &[u8] = b"23456789TJQKA";
        const SUITS: &[u8] = b"scdh";
        let bytes = hand.as_bytes();
        let mut ret: u64 = 0;
        let mut i = 0;
        while i + 1 < bytes.len() {
            let r = RANKS.iter().position(|&c| c == bytes[i]).unwrap_or(0);
            let s = SUITS.iter().position(|&c| c == bytes[i + 1]).unwrap_or(0);
            ret |= 1u64 << (r * SUIT_COUNT as usize + s);
            i += 2;
        }
        Self { mask: ret }
    }

    pub fn overlap(&self, other: &Self) -> bool {
        self.mask & other.mask != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_qqqkk() {
        let output = Hand::from_string("QsQcQhKsKc");
        let expected: u64 = [40, 41, 43, 44, 45].iter().map(|x| 1u64 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn parse_23456() {
        let output = Hand::from_string("2s3c4d5h6h");
        let expected: u64 = [0, 5, 10, 15, 19].iter().map(|x| 1u64 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn parse_aaaaq() {
        let output = Hand::from_string("AsAcAdAhQh");
        let expected: u64 = [48, 49, 50, 51, 43].iter().map(|x| 1u64 << x).sum();
        assert_eq!(output.mask, expected);
    }
    #[test]
    fn parse_empty() {
        let output = Hand::from_string("");
        assert_eq!(output.mask, 0);
        assert_eq!(output.len(), 0);
    }
}
