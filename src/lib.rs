pub mod poker;
pub use crate::poker::{solve_ranges, EquityResult, Game, Hand, Range};

use std::alloc::{alloc as raw_alloc, dealloc as raw_dealloc, Layout};
use std::cell::RefCell;
use std::slice;
use std::str;

thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

fn set_error(msg: &str) -> i32 {
    LAST_ERROR.with(|e| {
        let mut e = e.borrow_mut();
        e.clear();
        e.push_str(msg);
    });
    -1
}

/// # Safety
/// Returned pointer is valid until the next `last_error` write (any failing call).
#[no_mangle]
pub extern "C" fn last_error_ptr() -> *const u8 {
    LAST_ERROR.with(|e| e.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn last_error_len() -> usize {
    LAST_ERROR.with(|e| e.borrow().len())
}

/// Allocate `len` bytes, 8-byte aligned so callers can view the region as f64.
/// # Safety
/// Caller must release with `dealloc(ptr, len)`.
#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    let layout = Layout::from_size_align(len, 8).expect("bad layout");
    unsafe { raw_alloc(layout) }
}

/// # Safety
/// `ptr` must come from `alloc(len)` with the same `len`, released once.
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    let layout = Layout::from_size_align(len, 8).expect("bad layout");
    raw_dealloc(ptr, layout);
}

unsafe fn read_str<'a>(ptr: *const u8, len: usize) -> Result<&'a str, i32> {
    if len == 0 {
        return Ok("");
    }
    str::from_utf8(slice::from_raw_parts(ptr, len)).map_err(|_| set_error("invalid utf-8 input"))
}

/// Exhaustive head-to-head solve.
/// Writes [win, lose, tie] as 3 u32 to `out` (12 bytes). Returns 0 on success,
/// -1 on error (message via last_error_ptr/len).
/// # Safety
/// String pointers must reference `len` valid bytes; `out` must hold 12 bytes.
#[no_mangle]
pub unsafe extern "C" fn solve(
    hand_a_ptr: *const u8,
    hand_a_len: usize,
    hand_b_ptr: *const u8,
    hand_b_len: usize,
    community_ptr: *const u8,
    community_len: usize,
    out: *mut u32,
) -> i32 {
    let hand_a = match read_str(hand_a_ptr, hand_a_len) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let hand_b = match read_str(hand_b_ptr, hand_b_len) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let community = match read_str(community_ptr, community_len) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let mut game = Game::new();
    match game.solve_by(hand_a, hand_b, community) {
        Ok((win, lose, tie)) => {
            let out = slice::from_raw_parts_mut(out, 3);
            out[0] = win as u32;
            out[1] = lose as u32;
            out[2] = tie as u32;
            0
        }
        Err(e) => set_error(&e),
    }
}

/// Monte Carlo multi-way equity. Villain ranges arrive as one string separated
/// by '\n'; an empty segment means "any two cards".
/// Writes 4 + villain_count f64 to `out`:
/// [iterations, hero_win, hero_tie, hero_lose, villain_equity...].
/// Returns villain count on success, -1 on error.
/// # Safety
/// String pointers must reference `len` valid bytes; `out` must hold
/// (4 + villain_count) f64 and be 8-byte aligned (use `alloc`).
#[no_mangle]
pub unsafe extern "C" fn solve_multi(
    hero_ptr: *const u8,
    hero_len: usize,
    villains_ptr: *const u8,
    villains_len: usize,
    community_ptr: *const u8,
    community_len: usize,
    max_iterations: u32,
    seed: u32,
    out: *mut f64,
) -> i32 {
    let hero = match read_str(hero_ptr, hero_len) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let villains_str = match read_str(villains_ptr, villains_len) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let community = match read_str(community_ptr, community_len) {
        Ok(s) => s,
        Err(code) => return code,
    };

    let hero_range = match Range::from_notation(hero) {
        Ok(r) => r,
        Err(e) => return set_error(&format!("hero: {}", e)),
    };
    if hero_range.combos.len() != 1 {
        return set_error("hero must be a single combo (e.g. AsKs)");
    }
    let hero_mask = hero_range.combos[0].mask;

    let mut villains = Vec::new();
    for (i, s) in villains_str.split('\n').enumerate() {
        let r = if s.trim().is_empty() {
            Range::any()
        } else {
            match Range::from_notation(s) {
                Ok(r) => r,
                Err(e) => return set_error(&format!("villain {}: {}", i, e)),
            }
        };
        villains.push(r);
    }

    let community_mask = Hand::from_string(community).mask;
    let seed64 = if seed == 0 {
        0x9E37_79B9_7F4A_7C15
    } else {
        seed as u64
    };
    match solve_ranges(
        hero_mask,
        &villains,
        community_mask,
        max_iterations as u64,
        seed64,
    ) {
        Ok(r) => {
            let out = slice::from_raw_parts_mut(out, 4 + r.villain_equity.len());
            out[0] = r.iterations as f64;
            out[1] = r.hero_win;
            out[2] = r.hero_tie;
            out[3] = r.hero_lose;
            out[4..].copy_from_slice(&r.villain_equity);
            r.villain_equity.len() as i32
        }
        Err(e) => set_error(&e),
    }
}
