use crate::poker::evaluator::eval;
use crate::poker::Hand;
use crate::poker::Range;

pub const EXACT_WORK_LIMIT: u128 = 5_000_000;
pub const DEFAULT_MC_ITERATIONS: u64 = 200_000;

#[derive(Debug, Default, Clone)]
pub struct EquityResult {
    pub iterations: u64,
    pub hero_win: f64,
    pub hero_tie: f64,
    pub hero_lose: f64,
    pub villain_equity: Vec<f64>,
}

pub struct Game {
    pub hero: Range,
    pub villains: Vec<Range>,
    pub community: Hand,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            hero: Range::default(),
            villains: Vec::new(),
            community: Hand::default(),
        }
    }

    pub fn solve_by(
        &mut self,
        hero: &str,
        villain: &str,
        community: &str,
    ) -> Result<(usize, usize, usize), String> {
        self.hero = parse_player(hero)?;
        let villain = parse_player(villain)?;
        if self.hero.is_empty() {
            return Err("Invalid game!".into());
        }
        self.villains = vec![villain];
        self.community = Hand::from_string(community);
        let result = self.solve(0, 1)?;
        let iters = result.iterations as f64;
        let w = (result.hero_win * iters).round() as usize;
        let l = (result.hero_lose * iters).round() as usize;
        let t = (result.hero_tie * iters).round() as usize;
        Ok((w, l, t))
    }

    pub fn solve(&self, max_iterations: u64, seed: u64) -> Result<EquityResult, String> {
        if self.hero.combos.len() != 1 {
            return Err("Game::solve expects exactly one hero combo; use solve_ranges for hero ranges".into());
        }
        let hero_mask = self.hero.combos[0].mask;
        solve_ranges(
            hero_mask,
            &self.villains,
            self.community.mask,
            max_iterations,
            seed,
        )
    }
}

fn parse_player(s: &str) -> Result<Range, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Range::any());
    }
    Range::from_notation(s)
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

fn binom(n: u32, k: u32) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut out: u128 = 1;
    for i in 0..k {
        out = out * (n - i) as u128 / (i + 1) as u128;
    }
    out
}

fn estimate_work(villain_live: &[u128], free_after_villains: u32, need_c: u32) -> u128 {
    let mut prod: u128 = 1;
    for &c in villain_live {
        prod = prod.saturating_mul(c);
        if prod > EXACT_WORK_LIMIT {
            return prod;
        }
    }
    prod.saturating_mul(binom(free_after_villains, need_c))
}

fn tally_outcome(
    scores: &[u32],
    acc: &mut [f64],
    hero_win: &mut f64,
    hero_tie: &mut f64,
    hero_lose: &mut f64,
    weight: f64,
) {
    let max = *scores.iter().max().unwrap();
    let winners = scores.iter().filter(|&&s| s == max).count() as f64;
    let share = weight / winners;
    for (i, &s) in scores.iter().enumerate() {
        if s == max {
            acc[i] += share;
        }
    }
    let hero = scores[0];
    if hero == max {
        if winners > 1.5 {
            *hero_tie += weight;
        } else {
            *hero_win += weight;
        }
    } else {
        *hero_lose += weight;
    }
}

struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let s = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
        Self { state: s }
    }
    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    fn range(&mut self, n: usize) -> usize {
        (self.next() as usize) % n
    }
}

fn pick_random_board(used: u64, need: u32, rng: &mut XorShift64) -> u64 {
    let mut free: Vec<u8> = (0..52u8).filter(|&i| used & (1u64 << i) == 0).collect();
    let mut out = 0u64;
    for _ in 0..need {
        let idx = rng.range(free.len());
        out |= 1u64 << free[idx];
        free.swap_remove(idx);
    }
    out
}

pub fn solve_ranges(
    hero: u64,
    villains: &[Range],
    community: u64,
    max_iterations: u64,
    seed: u64,
) -> Result<EquityResult, String> {
    if hero.count_ones() != 2 {
        return Err("hero must have exactly 2 cards".into());
    }
    if community & hero != 0 {
        return Err("community overlaps hero".into());
    }
    let community_count = community.count_ones();
    if !(3..=5).contains(&community_count) {
        return Err("community must have 3-5 cards".into());
    }
    if villains.is_empty() {
        return Err("need at least one villain".into());
    }

    let need_c = 5 - community_count;
    let dead0 = hero | community;

    let mut live_counts: Vec<u128> = Vec::with_capacity(villains.len());
    for v in villains {
        let n = v.live_combos(dead0).count() as u128;
        if n == 0 {
            return Err("villain range has no live combos".into());
        }
        live_counts.push(n);
    }
    let free_after = 52u32 - dead0.count_ones() - 2 * villains.len() as u32;
    let work = estimate_work(&live_counts, free_after, need_c);

    if work <= EXACT_WORK_LIMIT {
        solve_exact(hero, villains, community, dead0, need_c)
    } else {
        let iters = if max_iterations == 0 {
            DEFAULT_MC_ITERATIONS
        } else {
            max_iterations
        };
        solve_mc(hero, villains, community, dead0, need_c, iters, seed)
    }
}

fn solve_exact(
    hero: u64,
    villains: &[Range],
    community: u64,
    dead0: u64,
    need_c: u32,
) -> Result<EquityResult, String> {
    let n = villains.len();
    let mut acc = vec![0.0f64; n + 1];
    let mut hero_win = 0.0;
    let mut hero_tie = 0.0;
    let mut hero_lose = 0.0;
    let mut iters: u64 = 0;
    let mut villain_masks = vec![0u64; n];

    enumerate_villains(
        0,
        dead0,
        villains,
        &mut villain_masks,
        &mut |used, masks| {
            pick(used, need_c, 0, 0, &mut |board_add| {
                let board = community | board_add;
                let mut scores: Vec<u32> = Vec::with_capacity(n + 1);
                scores.push(eval(hero | board));
                for &vm in masks.iter() {
                    scores.push(eval(vm | board));
                }
                tally_outcome(
                    &scores,
                    &mut acc,
                    &mut hero_win,
                    &mut hero_tie,
                    &mut hero_lose,
                    1.0,
                );
                iters += 1;
            });
        },
    );

    finalize(iters, acc, hero_win, hero_tie, hero_lose)
}

fn enumerate_villains<F: FnMut(u64, &[u64])>(
    i: usize,
    used: u64,
    villains: &[Range],
    masks: &mut Vec<u64>,
    f: &mut F,
) {
    if i == villains.len() {
        f(used, masks);
        return;
    }
    let combos: Vec<u64> = villains[i].live_combos(used).map(|c| c.mask).collect();
    for m in combos {
        masks[i] = m;
        enumerate_villains(i + 1, used | m, villains, masks, f);
    }
}

fn solve_mc(
    hero: u64,
    villains: &[Range],
    community: u64,
    dead0: u64,
    need_c: u32,
    target: u64,
    seed: u64,
) -> Result<EquityResult, String> {
    let n = villains.len();
    let live: Vec<Vec<u64>> = villains
        .iter()
        .map(|r| r.live_combos(dead0).map(|c| c.mask).collect())
        .collect();

    let mut acc = vec![0.0f64; n + 1];
    let mut hero_win = 0.0;
    let mut hero_tie = 0.0;
    let mut hero_lose = 0.0;
    let mut rng = XorShift64::new(seed);

    let mut iters: u64 = 0;
    let mut attempts: u64 = 0;
    let attempt_cap = target.saturating_mul(50);

    while iters < target && attempts < attempt_cap {
        attempts += 1;
        let mut used = dead0;
        let mut villain_masks: Vec<u64> = Vec::with_capacity(n);
        let mut ok = true;
        for combos in live.iter() {
            let mut tries = 0;
            let mut found: Option<u64> = None;
            while tries < 32 {
                let m = combos[rng.range(combos.len())];
                if m & used == 0 {
                    found = Some(m);
                    break;
                }
                tries += 1;
            }
            let chosen = found.or_else(|| combos.iter().copied().find(|&m| m & used == 0));
            match chosen {
                Some(m) => {
                    villain_masks.push(m);
                    used |= m;
                }
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if !ok {
            continue;
        }
        let board_add = pick_random_board(used, need_c, &mut rng);
        let board = community | board_add;
        let mut scores: Vec<u32> = Vec::with_capacity(n + 1);
        scores.push(eval(hero | board));
        for &vm in villain_masks.iter() {
            scores.push(eval(vm | board));
        }
        tally_outcome(
            &scores,
            &mut acc,
            &mut hero_win,
            &mut hero_tie,
            &mut hero_lose,
            1.0,
        );
        iters += 1;
    }

    if iters == 0 {
        return Err("Monte Carlo could not draw a valid sample".into());
    }
    finalize(iters, acc, hero_win, hero_tie, hero_lose)
}

fn finalize(
    iters: u64,
    acc: Vec<f64>,
    hero_win: f64,
    hero_tie: f64,
    hero_lose: f64,
) -> Result<EquityResult, String> {
    let denom = iters as f64;
    let villain_equity = acc[1..].iter().map(|x| x / denom).collect();
    Ok(EquityResult {
        iterations: iters,
        hero_win: hero_win / denom,
        hero_tie: hero_tie / denom,
        hero_lose: hero_lose / denom,
        villain_equity,
    })
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
    fn board_tjq_ka_xx() {
        let mut game = Game::new();
        let output = game.solve_by("KsAs", "", "TsJsQs").unwrap();
        assert_eq!((1070190, 0, 0), output);
    }

    #[test]
    fn board_aak_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAc", "", "AdAhKs").unwrap();
        assert_eq!((1070160, 30, 0), output);
    }

    #[test]
    fn board_222_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "", "2c2s2d").unwrap();
        assert_eq!((1007026, 56410, 6754), output);
    }

    #[test]
    fn board_234_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "", "2s3s4s").unwrap();
        assert_eq!((913275, 136214, 20701), output);
    }

    #[test]
    fn board_268_aa_xx() {
        let mut game = Game::new();
        let output = game.solve_by("AsAd", "", "2c6s8s").unwrap();
        assert_eq!((902562, 166683, 945), output);
    }

    #[test]
    fn board_268_tq_xx() {
        let mut game = Game::new();
        let output = game.solve_by("TdQh", "", "2c6s8s").unwrap();
        assert_eq!((400858, 657394, 11938), output);
    }

    #[test]
    fn board_8tq_6s2h_xx() {
        let mut game = Game::new();
        let output = game.solve_by("6s2h", "", "8cTdQh").unwrap();
        assert_eq!((139374, 818875, 111941), output);
    }

    #[test]
    fn ranges_two_villains_revealed() {
        let hero = Hand::from_string("AsAd").mask;
        let v1 = Range::from_notation("KsKd").unwrap();
        let v2 = Range::from_notation("QsQd").unwrap();
        let community = Hand::from_string("2c3d7h4h6h").mask;
        let r = solve_ranges(hero, &[v1, v2], community, 0, 1).unwrap();
        assert_eq!(r.iterations, 1);
        assert!((r.hero_win - 1.0).abs() < 1e-9);
        assert!((r.villain_equity[0] - 0.0).abs() < 1e-9);
        assert!((r.villain_equity[1] - 0.0).abs() < 1e-9);
    }

    #[test]
    fn ranges_three_way_tie() {
        let hero = Hand::from_string("AsKs").mask;
        let v1 = Range::from_notation("AcKc").unwrap();
        let v2 = Range::from_notation("AhKh").unwrap();
        let community = Hand::from_string("2d3d4d5d6d").mask;
        let r = solve_ranges(hero, &[v1, v2], community, 0, 1).unwrap();
        assert!((r.hero_tie - 1.0).abs() < 1e-9);
        assert!((r.hero_win - 0.0).abs() < 1e-9);
    }

    #[test]
    fn ranges_aa_vs_pair_range() {
        let hero = Hand::from_string("AsAd").mask;
        let v = Range::from_notation("KK").unwrap();
        let community = Hand::from_string("2c3d7h").mask;
        let r = solve_ranges(hero, &[v], community, 0, 1).unwrap();
        assert!(r.hero_win > 0.85);
        assert!(r.hero_lose < 0.1);
    }
}
