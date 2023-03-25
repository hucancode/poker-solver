use std::collections::HashMap;
pub const RANK_COUNT: i64 = 13;
pub const SUIT_COUNT: i64 = 4;

pub fn from_string(hand: &str) -> i64 {
    let ranks = String::from("23456789TJQKA");
    let suits = String::from("scdh");
    let mut ret: i64 = 0;
    let mut i = hand.chars();
    while let Some((cr, cs)) = i.next().zip(i.next()) {
        let r = ranks.find(cr).unwrap_or_default();
        let s = suits.find(cs).unwrap_or_default();
        ret |= 1 << (r * SUIT_COUNT as usize + s);
    }
    ret
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
// return `count` highest bit from `mask`
pub fn get_highest_card(mask: i64, count: usize) -> i64 {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_qqqkk() {
        let output = from_string("QsQcQhKsKc");
        let expected = [40, 41, 43, 44, 45].iter().fold(0, |acc, x| acc | 1 << x);
        assert_eq!(output, expected);
    }
    #[test]
    fn parse_23456() {
        let output = from_string("2s3c4d5h6h");
        let expected = [0, 5, 10, 15, 19].iter().fold(0, |acc, x| acc | 1 << x);
        assert_eq!(output, expected);
    }
    #[test]
    fn parse_aaaaq() {
        let output = from_string("AsAcAdAhQh");
        let expected = [48, 49, 50, 51, 43].iter().fold(0, |acc, x| acc | 1 << x);
        assert_eq!(output, expected);
    }
    #[test]
    fn highcard_23456() {
        let input = [0, 5, 10, 15, 19].iter().fold(0, |acc, x| acc | 1 << x);
        let output = get_highest_card(input, 2);
        let expected = [15, 19].iter().fold(0, |acc, x| acc | 1 << x);
        assert_eq!(output, expected);
    }
    #[test]
    fn highcard_aaaaq() {
        let input = [48, 49, 50, 51, 43].iter().fold(0, |acc, x| acc | 1 << x);
        let output = get_highest_card(input, 3);
        let expected = [49, 50, 51].iter().fold(0, |acc, x| acc | 1 << x);
        assert_eq!(output, expected);
    }
}
