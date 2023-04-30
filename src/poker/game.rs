use crate::evaluator::CompareResult;
use crate::evaluator::HandEvaluator;
use crate::hand::from_string;
use crate::hand::RANK_COUNT;
use crate::hand::SUIT_COUNT;
use futures::future::join_all;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Game {
    pub hand_a: i64,
    pub hand_b: i64,
    pub community: i64,
    evaluator: HandEvaluator,
}
impl Game {
    pub fn new() -> Self {
        Self {
            hand_a: 0,
            hand_b: 0,
            community: 0,
            evaluator: HandEvaluator::new(),
        }
    }
    fn is_valid(&self) -> bool {
        let a = self.hand_a.count_ones();
        let b = self.hand_b.count_ones();
        let c = self.community.count_ones();
        a == 2
            && b <= 2
            && (3..=5).contains(&c)
            && self.hand_a & self.hand_b == 0
            && self.hand_a & self.community == 0
            && self.hand_b & self.community == 0
    }
    pub fn with_hand_a(mut self, a: &str) -> Self {
        self.hand_a = from_string(a);
        self
    }
    pub fn with_hand_b(mut self, b: &str) -> Self {
        self.hand_b = from_string(b);
        self
    }
    pub fn with_community(mut self, c: &str) -> Self {
        self.community = from_string(c);
        self
    }
    pub async fn solve(&self) -> Result<(i32, i32, i32), &str> {
        if !self.is_valid() {
            return Err("Invalid game!");
        }
        let mut tasks = Vec::new();
        let mut q = VecDeque::new();
        q.push_back((self.hand_a, self.hand_b, self.community));
        let deck = (0..(RANK_COUNT * SUIT_COUNT)).map(|x| 1 << x);
        while let Some((a, b, c)) = q.pop_front() {
            let board = a | b | c;
            if board.count_ones() >= 9 {
                tasks.push(self.evaluator.compare(a | c, b | c));
                continue;
            }
            let it = deck.clone().filter(|x| x & board == 0);
            if b.count_ones() < 2 {
                let mut bq = VecDeque::new();
                bq.push_back((it, b));
                while let Some((it, b)) = bq.pop_front() {
                    if b.count_ones() == 2 {
                        q.push_back((a, b, c));
                        continue;
                    }
                    let mut it = it.clone();
                    while let Some(x) = it.next() {
                        bq.push_back((it.clone(), b | x));
                    }
                }
                continue;
            }
            if c.count_ones() < 5 {
                let mut cq = VecDeque::new();
                cq.push_back((it, c));
                while let Some((it, c)) = cq.pop_front() {
                    if c.count_ones() == 5 {
                        q.push_back((a, b, c));
                        continue;
                    }
                    let mut it = it.clone();
                    while let Some(x) = it.next() {
                        cq.push_back((it.clone(), c | x));
                    }
                }
                continue;
            }
        }

        Ok(join_all(tasks)
            .await
            .iter()
            .fold((0, 0, 0), |(win, lose, tie), r| match r {
                CompareResult::AWin => (win + 1, lose, tie),
                CompareResult::BWin => (win, lose + 1, tie),
                CompareResult::Tie => (win, lose, tie + 1),
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn invalid_game() {
        let game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("KsKd")
            .with_community("As3s7s");
        assert!(game.solve().await.is_err());
    }

    #[tokio::test]
    async fn vs_237_aa_kk() {
        let game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("KsKd")
            .with_community("2s3s7s");
        let output = game.solve().await.unwrap();
        assert_eq!((923, 67, 0), output);
    }

    #[tokio::test]
    async fn vs_23456_aa_empty() {
        let game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("")
            .with_community("2s3s4s5s6s");
        let output = game.solve().await.unwrap();
        assert_eq!((0, 44, 946), output);
    }

    #[tokio::test]
    async fn vs_2345_aa_empty() {
        let game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("")
            .with_community("2s3s4s5s");
        let output = game.solve().await.unwrap();
        assert_eq!((42570, 2024, 946), output);
    }

    #[tokio::test]
    #[ignore]
    async fn vs_234_aa_empty() {
        let game = Game::new()
            .with_hand_a("AsAd")
            .with_hand_b("")
            .with_community("2s3s4s");
        let output = game.solve().await.unwrap();
        assert_eq!((913275, 136214, 20701), output);
    }
}
