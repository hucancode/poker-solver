use crate::poker::hand::SUIT_COUNT;

const RANKS: &[u8] = b"23456789TJQKA";
const SUITS: &[u8] = b"scdh";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Combo {
    pub mask: u64,
    pub weight: f32,
}

impl Combo {
    pub fn new(mask: u64) -> Self {
        Self { mask, weight: 1.0 }
    }
    pub fn with_weight(mask: u64, weight: f32) -> Self {
        Self { mask, weight }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Range {
    pub combos: Vec<Combo>,
}

fn rank_idx(c: u8) -> Option<u8> {
    RANKS.iter().position(|&x| x == c).map(|p| p as u8)
}

fn card_bit(rank: u8, suit: u8) -> u64 {
    1u64 << (rank as u32 * SUIT_COUNT + suit as u32)
}

fn pair_combos(rank: u8) -> Vec<Combo> {
    let mut out = Vec::with_capacity(6);
    for s1 in 0..4u8 {
        for s2 in (s1 + 1)..4u8 {
            let m = card_bit(rank, s1) | card_bit(rank, s2);
            out.push(Combo::new(m));
        }
    }
    out
}

fn suited_combos(r1: u8, r2: u8) -> Vec<Combo> {
    let mut out = Vec::with_capacity(4);
    for s in 0..4u8 {
        let m = card_bit(r1, s) | card_bit(r2, s);
        out.push(Combo::new(m));
    }
    out
}

fn offsuit_combos(r1: u8, r2: u8) -> Vec<Combo> {
    let mut out = Vec::with_capacity(12);
    for s1 in 0..4u8 {
        for s2 in 0..4u8 {
            if s1 == s2 {
                continue;
            }
            let m = card_bit(r1, s1) | card_bit(r2, s2);
            out.push(Combo::new(m));
        }
    }
    out
}

fn any_combos(r1: u8, r2: u8) -> Vec<Combo> {
    let mut out = suited_combos(r1, r2);
    out.extend(offsuit_combos(r1, r2));
    out
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Suitedness {
    Suited,
    Offsuit,
    Any,
}

fn parse_hand_head(s: &[u8]) -> Result<(u8, u8, Suitedness, usize), String> {
    if s.len() < 2 {
        return Err(format!("token too short: {:?}", String::from_utf8_lossy(s)));
    }
    let r1 = rank_idx(s[0]).ok_or_else(|| format!("bad rank: {}", s[0] as char))?;
    let r2 = rank_idx(s[1]).ok_or_else(|| format!("bad rank: {}", s[1] as char))?;
    let (sd, used) = if s.len() >= 3 {
        match s[2] {
            b's' => (Suitedness::Suited, 3),
            b'o' => (Suitedness::Offsuit, 3),
            _ => (Suitedness::Any, 2),
        }
    } else {
        (Suitedness::Any, 2)
    };
    Ok((r1, r2, sd, used))
}

fn build_combos(r1: u8, r2: u8, sd: Suitedness) -> Result<Vec<Combo>, String> {
    if r1 == r2 {
        if sd != Suitedness::Any {
            return Err("pair cannot be suited/offsuit".into());
        }
        return Ok(pair_combos(r1));
    }
    let (hi, lo) = if r1 > r2 { (r1, r2) } else { (r2, r1) };
    Ok(match sd {
        Suitedness::Suited => suited_combos(hi, lo),
        Suitedness::Offsuit => offsuit_combos(hi, lo),
        Suitedness::Any => any_combos(hi, lo),
    })
}

fn explicit_combo(token: &[u8]) -> Option<Combo> {
    if token.len() != 4 {
        return None;
    }
    let r1 = rank_idx(token[0])?;
    let s1 = SUITS.iter().position(|&c| c == token[1])? as u8;
    let r2 = rank_idx(token[2])?;
    let s2 = SUITS.iter().position(|&c| c == token[3])? as u8;
    let m = card_bit(r1, s1) | card_bit(r2, s2);
    if m.count_ones() != 2 {
        return None;
    }
    Some(Combo::new(m))
}

fn parse_token(token: &str) -> Result<Vec<Combo>, String> {
    let token = token.trim();
    if token.is_empty() {
        return Ok(vec![]);
    }
    if let Some(c) = explicit_combo(token.as_bytes()) {
        return Ok(vec![c]);
    }
    let bytes = token.as_bytes();
    let (r1, r2, sd, used) = parse_hand_head(bytes)?;

    let tail = &bytes[used..];
    if tail.is_empty() {
        return build_combos(r1, r2, sd);
    }
    if tail == b"+" {
        return expand_plus(r1, r2, sd);
    }
    if tail[0] == b'-' {
        let (er1, er2, esd, eused) = parse_hand_head(&tail[1..])?;
        if eused != tail.len() - 1 {
            return Err(format!("trailing chars in range: {}", token));
        }
        return expand_range(r1, r2, sd, er1, er2, esd);
    }
    Err(format!("cannot parse: {}", token))
}

fn expand_plus(r1: u8, r2: u8, sd: Suitedness) -> Result<Vec<Combo>, String> {
    let mut out = Vec::new();
    if r1 == r2 {
        for r in r1..13 {
            out.extend(build_combos(r, r, Suitedness::Any)?);
        }
        return Ok(out);
    }
    let (hi, lo) = if r1 > r2 { (r1, r2) } else { (r2, r1) };
    for k in lo..hi {
        out.extend(build_combos(hi, k, sd)?);
    }
    Ok(out)
}

fn expand_range(
    r1: u8,
    r2: u8,
    sd: Suitedness,
    er1: u8,
    er2: u8,
    esd: Suitedness,
) -> Result<Vec<Combo>, String> {
    if sd != esd {
        return Err("range endpoints suitedness mismatch".into());
    }
    let mut out = Vec::new();
    if r1 == r2 && er1 == er2 {
        let (hi, lo) = if r1 > er1 { (r1, er1) } else { (er1, r1) };
        for r in lo..=hi {
            out.extend(build_combos(r, r, Suitedness::Any)?);
        }
        return Ok(out);
    }
    let (hi_a, lo_a) = if r1 > r2 { (r1, r2) } else { (r2, r1) };
    let (hi_b, lo_b) = if er1 > er2 { (er1, er2) } else { (er2, er1) };
    if hi_a != hi_b {
        return Err("range endpoints high card mismatch".into());
    }
    let (lo_hi, lo_lo) = if lo_a > lo_b {
        (lo_a, lo_b)
    } else {
        (lo_b, lo_a)
    };
    for k in lo_lo..=lo_hi {
        out.extend(build_combos(hi_a, k, sd)?);
    }
    Ok(out)
}

impl Range {
    pub fn from_notation(s: &str) -> Result<Self, String> {
        let mut combos = Vec::new();
        for tok in s.split(',') {
            combos.extend(parse_token(tok)?);
        }
        combos.sort_by_key(|c| c.mask);
        combos.dedup_by_key(|c| c.mask);
        Ok(Self { combos })
    }

    pub fn any() -> Self {
        let mut combos = Vec::with_capacity(1326);
        for i in 0..52u8 {
            for j in (i + 1)..52u8 {
                combos.push(Combo::new((1u64 << i) | (1u64 << j)));
            }
        }
        Self { combos }
    }

    pub fn len(&self) -> usize {
        self.combos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.combos.is_empty()
    }

    pub fn live_combos<'a>(&'a self, dead: u64) -> impl Iterator<Item = &'a Combo> + 'a {
        self.combos.iter().filter(move |c| c.mask & dead == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pair() {
        let r = Range::from_notation("QQ").unwrap();
        assert_eq!(r.len(), 6);
    }

    #[test]
    fn parse_pair_plus() {
        let r = Range::from_notation("QQ+").unwrap();
        assert_eq!(r.len(), 18);
    }

    #[test]
    fn parse_pair_range() {
        let r = Range::from_notation("QQ-99").unwrap();
        assert_eq!(r.len(), 24);
    }

    #[test]
    fn parse_suited() {
        let r = Range::from_notation("AKs").unwrap();
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn parse_offsuit() {
        let r = Range::from_notation("AKo").unwrap();
        assert_eq!(r.len(), 12);
    }

    #[test]
    fn parse_any() {
        let r = Range::from_notation("AK").unwrap();
        assert_eq!(r.len(), 16);
    }

    #[test]
    fn parse_suited_plus() {
        let r = Range::from_notation("A2s+").unwrap();
        assert_eq!(r.len(), 12 * 4);
    }

    #[test]
    fn parse_combined() {
        let r = Range::from_notation("QQ+,AKs,AKo").unwrap();
        assert_eq!(r.len(), 18 + 4 + 12);
    }

    #[test]
    fn parse_explicit_combo() {
        let r = Range::from_notation("AsKd").unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn dedup_overlap() {
        let r = Range::from_notation("AKs,AKs").unwrap();
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn dead_filter() {
        let r = Range::from_notation("AA").unwrap();
        let dead = 1u64 << (12 * 4);
        let live: Vec<_> = r.live_combos(dead).collect();
        assert_eq!(live.len(), 3);
    }

    #[test]
    fn any_range() {
        let r = Range::any();
        assert_eq!(r.len(), 1326);
    }
}
