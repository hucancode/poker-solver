use std::collections::HashMap;

pub fn prettify(hand: &str) -> String {
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

#[cfg(test)]
mod tests {
    #[test]
    fn prettify() {
        let output = super::prettify("3s2c4h6sAd");
        let expected = "3♠2♣4♥6♠A♦";
        assert_eq!(output, expected);
        let output = super::prettify("QsQcQhKsKd");
        let expected = "Q♠Q♣Q♥K♠K♦";
        assert_eq!(output, expected);
        let output = super::prettify("");
        let expected = "";
        assert_eq!(output, expected);
        let output = super::prettify("2s2c2d2h");
        let expected = "2♠2♣2♦2♥";
        assert_eq!(output, expected);
        let output = super::prettify("2s3s4s5s6s7s8s9sTsJsQsKsAs");
        let expected = "2♠3♠4♠5♠6♠7♠8♠9♠10♠J♠Q♠K♠A♠";
        assert_eq!(output, expected);
    }
}
