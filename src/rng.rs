//! Seeded, reproducible RNG (ChaCha20). Same seed -> same output on every platform.
use crate::odds::Matrix;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

pub fn seeded(seed: u64) -> ChaCha20Rng {
    ChaCha20Rng::seed_from_u64(seed)
}

/// Uniform integer in [0, n) by rejection sampling (no modulo bias).
pub fn below(rng: &mut ChaCha20Rng, n: u32) -> u32 {
    assert!(n > 0);
    let zone = u32::MAX - (u32::MAX % n);
    loop {
        let x = next_u32(rng);
        if x < zone {
            return x % n;
        }
    }
}

fn next_u32(rng: &mut ChaCha20Rng) -> u32 {
    use rand_core::Rng;
    rng.next_u32()
}

/// One random play/draw for the matrix: sorted distinct whites in 1..=white_max and a bonus.
pub fn pick(rng: &mut ChaCha20Rng, m: &Matrix) -> (Vec<u32>, u32) {
    let mut pool: Vec<u32> = (1..=m.white_max).collect();
    // Partial Fisher-Yates.
    for i in 0..m.white_count as usize {
        let j = i + below(rng, (pool.len() - i) as u32) as usize;
        pool.swap(i, j);
    }
    let mut whites = pool[..m.white_count as usize].to_vec();
    whites.sort_unstable();
    let bonus = 1 + below(rng, m.bonus_max);
    (whites, bonus)
}

/// Monte Carlo: `n` random tickets vs random draws. Returns counts indexed like PRIZE_TIERS
/// plus a final "no prize" bucket. Deterministic for a given seed.
pub fn simulate(m: &Matrix, n: u64, seed: u64) -> Vec<u64> {
    let mut rng = seeded(seed);
    let tiers = crate::odds::PRIZE_TIERS;
    let mut counts = vec![0u64; tiers.len() + 1];
    for _ in 0..n {
        let (tw, tb) = pick(&mut rng, m);
        let (dw, db) = pick(&mut rng, m);
        let wm = tw.iter().filter(|x| dw.contains(x)).count() as u32;
        let bm = tb == db;
        match tiers.iter().position(|&(w, b)| w == wm && b == bm) {
            Some(i) => counts[i] += 1,
            None => *counts.last_mut().unwrap() += 1,
        }
    }
    counts
}
