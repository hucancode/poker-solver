use crate::poker::Evaluator;
use crate::poker::Hand;
use std::cmp::Ordering;

#[derive(Default)]
pub struct Game {
    pub hand_a: Hand,
    pub hand_b: Hand,
    pub community: Hand,
    evaluator: Evaluator,
}

fn pick<F: FnMut(u64)>(used: u64, k: u32, start: u32, picked: u64, f: &mut F) {
    if k == 0 {
        f(picked);
        return;
    }
    let mut i = start;
    while i + k <= 52 {
        let bit = 1u64 << i;
        if used & bit == 0 {
            pick(used | bit, k - 1, i + 1, picked | bit, f);
        }
        i += 1;
    }
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
    pub fn solve_by(&mut self, a: &str, b: &str, c: &str) -> Result<(usize, usize, usize), String> {
        self.hand_a = Hand::from_string(a);
        self.hand_b = Hand::from_string(b);
        self.community = Hand::from_string(c);
        self.solve()
    }
    pub fn solve(&mut self) -> Result<(usize, usize, usize), String> {
        if !self.is_valid() {
            return Err("Invalid game!".to_string());
        }
        let a_mask = self.hand_a.mask;
        let b_mask = self.hand_b.mask;
        let c_mask = self.community.mask;
        let used = a_mask | b_mask | c_mask;
        let need_b = 2 - self.hand_b.len();
        let need_c = 5 - self.community.len();
        let evaluator = &self.evaluator;

        let mut win = 0usize;
        let mut lose = 0usize;
        let mut tie = 0usize;

        pick(used, need_b, 0, 0, &mut |b_add: u64| {
            let used2 = used | b_add;
            pick(used2, need_c, 0, 0, &mut |c_add: u64| {
                let board = c_mask | c_add;
                let a_h = Hand::from_mask(a_mask | board);
                let b_h = Hand::from_mask(b_mask | b_add | board);
                match evaluator.compare(&a_h, &b_h) {
                    Ordering::Greater => win += 1,
                    Ordering::Less => lose += 1,
                    Ordering::Equal => tie += 1,
                }
            });
        });

        Ok((win, lose, tie))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_game() {
        let mut game = Game::new();
        assert!(game.solve_by("AsAd", "KsKd", "As3s7s").is_err());
    }

    #[test]
    fn revealed_game_1() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "KsKd", "2s3s7s3d6s").unwrap();
        assert_eq!((1, 0, 0), output);
    }

    #[test]
    fn revealed_game_2() {
        let mut game = Game::new();
        let output = game.solve_by("3s2d", "2s3d", "9dTs7s4d6s").unwrap();
        assert_eq!((0, 0, 1), output);
    }

    #[test]
    fn revealed_game_3() {
        let mut game = Game::new();
        let output = game.solve_by("3s2d", "2s3d", "9sTs7s4d6s").unwrap();
        assert_eq!((1, 0, 0), output);
    }

    #[test]
    fn board_237_aa_kk() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "KsKd", "2s3s7s").unwrap();
        assert_eq!((923, 67, 0), output);
    }

    #[test]
    fn board_23456_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "", "2s3s4s5s6s").unwrap();
        assert_eq!((0, 44, 946), output);
    }

    #[test]
    fn board_2345_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "", "2s3s4s5s").unwrap();
        assert_eq!((42570, 2024, 946), output);
    }

    #[test]
    #[ignore]
    fn board_tjq_ka_xx() {
        let mut game = Game::new();
        let output = game.solve_by("KsAs", "", "TsJsQs").unwrap();
        assert_eq!((1070190, 0, 0), output);
    }

    #[test]
    #[ignore]
    fn board_aak_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAc", "", "AdAhKs").unwrap();
        assert_eq!((1070160, 30, 0), output);
    }

    #[test]
    #[ignore]
    fn board_222_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "", "2c2s2d").unwrap();
        assert_eq!((1007026, 56410, 6754), output);
    }

    #[test]
    #[ignore]
    fn board_234_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "", "2s3s4s").unwrap();
        assert_eq!((913275, 136214, 20701), output);
    }

    #[test]
    #[ignore]
    fn board_268_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "", "2c6s8s").unwrap();
        assert_eq!((902562, 166683, 945), output);
    }

    #[test]
    #[ignore]
    fn board_268_tq_xx() {
        let mut game = Game::new();
        let output = game.solve_by("TdQh", "", "2c6s8s").unwrap();
        assert_eq!((400858, 657394, 11938), output);
    }

    #[test]
    #[ignore]
    fn board_8tq_6s2h_xx() {
        let mut game = Game::new();
        let output = game.solve_by("6s2h", "", "8cTdQh").unwrap();
        assert_eq!((139374, 818875, 111941), output);
    }
}
