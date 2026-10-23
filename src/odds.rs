//! Exact (integer) odds for 5-of-N + 1-of-M matrices.

/// Binomial coefficient C(n, k) in u128 (exact for lottery sizes).
pub fn comb(n: u64, k: u64) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut r: u128 = 1;
    for i in 0..k {
        r = r * (n - i) as u128 / (i + 1) as u128;
    }
    r
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Tier {
    pub white: u32,
    pub bonus: bool,
    /// Number of (white set, bonus) outcomes that produce exactly this match.
    pub ways: u128,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Matrix {
    pub white_count: u32,
    pub white_max: u32,
    pub bonus_max: u32,
}

/// The nine prize tiers shared by the current Powerball and Mega Millions prize charts.
pub const PRIZE_TIERS: [(u32, bool); 9] = [
    (5, true),
    (5, false),
    (4, true),
    (4, false),
    (3, true),
    (3, false),
    (2, true),
    (1, true),
    (0, true),
];

impl Matrix {
    pub fn new(white_count: u32, white_max: u32, bonus_max: u32) -> Self {
        Self {
            white_count,
            white_max,
            bonus_max,
        }
    }
    /// Total equally likely outcomes: C(white_max, white_count) * bonus_max.
    pub fn total(&self) -> u128 {
        comb(self.white_max as u64, self.white_count as u64) * self.bonus_max as u128
    }
    /// Ways to match exactly `white` whites and (`bonus` ? the : not the) bonus ball.
    pub fn ways(&self, white: u32, bonus: bool) -> u128 {
        let k = self.white_count as u64;
        let n = self.white_max as u64;
        let w = comb(k, white as u64) * comb(n - k, k - white as u64);
        if bonus {
            w
        } else {
            w * (self.bonus_max as u128 - 1)
        }
    }
    pub fn tier(&self, white: u32, bonus: bool) -> Tier {
        Tier {
            white,
            bonus,
            ways: self.ways(white, bonus),
        }
    }
    pub fn prize_tiers(&self) -> Vec<Tier> {
        PRIZE_TIERS.iter().map(|&(w, b)| self.tier(w, b)).collect()
    }
    /// Odds "1 in X" as f64 for display.
    pub fn one_in(&self, white: u32, bonus: bool) -> f64 {
        self.total() as f64 / self.ways(white, bonus) as f64
    }
    /// "1 in X" rounded half-up to `decimals` places, as an exact scaled integer
    /// (e.g. decimals=2 -> hundredths). Pure integer arithmetic, no float error.
    pub fn one_in_scaled(&self, white: u32, bonus: bool, decimals: u32) -> u128 {
        round_div(self.total() * 10u128.pow(decimals), self.ways(white, bonus))
    }
    /// Overall odds of any prize tier (sum of PRIZE_TIERS ways), scaled like above.
    pub fn overall_scaled(&self, decimals: u32) -> u128 {
        let w: u128 = self.prize_tiers().iter().map(|t| t.ways).sum();
        round_div(self.total() * 10u128.pow(decimals), w)
    }
}

/// Round-half-up integer division.
pub fn round_div(num: u128, den: u128) -> u128 {
    (2 * num + den) / (2 * den)
}
