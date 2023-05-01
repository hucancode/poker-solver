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
        let output = super::prettify("QsQcQhKsKd");
        let expected = "Q♠Q♣Q♥K♠K♦";
        assert_eq!(output, expected);
    }
}
