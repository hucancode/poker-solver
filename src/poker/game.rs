use crate::poker::hand::RANK_COUNT;
use crate::poker::hand::SUIT_COUNT;
use crate::poker::Evaluator;
use crate::poker::Hand;
use std::cmp::Ordering;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Game {
    pub hand_a: Hand,
    pub hand_b: Hand,
    pub community: Hand,
    evaluator: Evaluator,
}
impl Game {
    pub fn new() -> Self {
        Self {
            hand_a: Hand::default(),
            hand_b: Hand::default(),
            community: Hand::default(),
            evaluator: Evaluator::new(),
        }
    }
    fn is_valid(&self) -> bool {
        let a = self.hand_a.len();
        let b = self.hand_b.len();
        let c = self.community.len();
        a == 2
            && b <= 2
            && (3..=5).contains(&c)
            && !self.hand_a.overlap(&self.hand_b)
            && !self.hand_a.overlap(&self.community)
            && !self.hand_b.overlap(&self.community)
    }
    pub fn with_hand_a(mut self, a: &str) -> Self {
        self.hand_a = Hand::from_string(a);
        self
    }
    pub fn with_hand_b(mut self, b: &str) -> Self {
        self.hand_b = Hand::from_string(b);
        self
    }
    pub fn with_community(mut self, c: &str) -> Self {
        self.community = Hand::from_string(c);
        self
    }
    pub fn solve(&mut self) -> Result<(usize, usize, usize), &str> {
        if !self.is_valid() {
            return Err("Invalid game!");
        }
        let mut tasks = Vec::new();
        let mut q = VecDeque::new();
        q.push_back((self.hand_a, self.hand_b, self.community));
        let deck = (0..(RANK_COUNT * SUIT_COUNT)).map(|x| Hand::from_mask(1 << x));
        while let Some((a, b, c)) = q.pop_front() {
            let board = a.merge(&b).merge(&c);
            if board.len() >= 9 {
                tasks.push((a.merge(&c), b.merge(&c)));
                continue;
            }
            let it = deck.clone().filter(|x| !x.overlap(&board));
            if b.len() < 2 {
                let mut bq = VecDeque::new();
                bq.push_back((it, b));
                while let Some((it, b)) = bq.pop_front() {
                    if b.len() == 2 {
                        q.push_back((a, b, c));
                        continue;
                    }
                    let mut it = it.clone();
                    while let Some(x) = it.next() {
                        bq.push_back((it.clone(), b.merge(&x)));
                    }
                }
                continue;
            }
            if c.len() < 5 {
                let mut cq = VecDeque::new();
                cq.push_back((it, c));
                while let Some((it, c)) = cq.pop_front() {
                    if c.len() == 5 {
                        q.push_back((a, b, c));
                        continue;
                    }
                    let mut it = it.clone();
                    while let Some(x) = it.next() {
                        cq.push_back((it.clone(), c.merge(&x)));
                    }
                }
                continue;
            }
        }

        Ok(tasks
            .iter()
            .map(|(a, b)| self.evaluator.compare(a, b))
            .fold((0, 0, 0), |(win, lose, tie), r| match r {
                Ordering::Greater => (win + 1, lose, tie),
                Ordering::Less => (win, lose + 1, tie),
                Ordering::Equal => (win, lose, tie + 1),
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_game() {
        let mut game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("KsKd")
            .with_community("As3s7s");
        assert!(game.solve().is_err());
    }

    #[test]
    fn board_237_aa_kk() {
        let mut game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("KsKd")
            .with_community("2s3s7s");
        let output = game.solve().unwrap();
        assert_eq!((923, 67, 0), output);
    }

    #[test]
    fn board_23456_aa_xx() {
        let mut game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("")
            .with_community("2s3s4s5s6s");
        let output = game.solve().unwrap();
        assert_eq!((0, 44, 946), output);
    }

    #[test]
    fn board_2345_aa_xx() {
        let mut game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("")
            .with_community("2s3s4s5s");
        let output = game.solve().unwrap();
        assert_eq!((42570, 2024, 946), output);
    }

    #[test]
    #[ignore]
    fn board_tjq_ka_xx() {
        let mut game = Game::new()
            .with_hand_a("KsAs")
            .with_hand_b("")
            .with_community("TsJsQs");
        let output = game.solve().unwrap();
        assert_eq!((1070190, 0, 0), output);
    }

    #[test]
    #[ignore]
    fn board_aak_aa_xx() {
        let mut game = Game::new()
            .with_hand_a("AsAc")
            .with_hand_b("")
            .with_community("AdAhKs");
        let output = game.solve().unwrap();
        assert_eq!((1070160, 30, 0), output);
    }

    #[test]
    #[ignore]
    fn board_222_aa_xx() {
        let mut game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("")
            .with_community("2c2s2d");
        let output = game.solve().unwrap();
        assert_eq!((1007026, 56410, 6754), output);
    }

    #[test]
    #[ignore]
    fn board_234_aa_xx() {
        let mut game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("")
            .with_community("2s3s4s");
        let output = game.solve().unwrap();
        assert_eq!((913275, 136214, 20701), output);
    }

    #[test]
    #[ignore]
    fn board_268_aa_xx() {
        let mut game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("")
            .with_community("2c6s8s");
        let output = game.solve().unwrap();
        assert_eq!((902562, 166683, 945), output);
    }

    #[test]
    #[ignore]
    fn board_268_tq_xx() {
        let mut game = Game::new()
            .with_hand_a("TdQh")
            .with_hand_b("")
            .with_community("2c6s8s");
        let output = game.solve().unwrap();
        assert_eq!((400858, 657394, 11938), output);
    }

    #[test]
    #[ignore]
    fn board_8tq_6s2h_xx() {
        let mut game = Game::new()
            .with_hand_a("6s2h")
            .with_hand_b("")
            .with_community("8cTdQh");
        let output = game.solve().unwrap();
        assert_eq!((139374, 818875, 111941), output);
    }
}
