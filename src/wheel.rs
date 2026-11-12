//! v0.2 Step 3: fixed-cost "at least 2 whites" coverage set (D-033).
//!
//! Guarantee (and nothing more): **at least one ticket matches 2 or more whites for every
//! possible draw.** The 1..N white numbers are split into at most 4 groups and every pair of
//! numbers inside a group appears together on some ticket. Any 5 drawn whites put 2 numbers in
//! the same group (pigeonhole: 5 numbers, at most 4 groups), so some ticket holds both. This is
//! a statement about matches, not prizes: 2 whites without the special ball pays $0 in both
//! games, expected value stays negative, and no ticket's odds change. Draws are independent.
//!
//! The base designs below were found offline (simulated annealing over within-group pair
//! coverage, D-033) and are pinned here; [`verify`] re-checks them, and every relabelled set,
//! before anything is printed or entered. No draw history, frequency or prediction is used.

use crate::crowd;
use crate::data::Draw;
use crate::eras::Era;
use anyhow::{Result, bail};
use rand_chacha::ChaCha20Rng;
use serde::Serialize;

/// Printed verbatim with every result (text and JSON).
pub const GUARANTEE: &str = "At least one ticket matches 2 or more whites for every possible draw.";
pub const CAVEATS: [&str; 5] = [
    "This guarantees no prize: 2 whites without the special ball pays $0 (Powerball and Mega Millions).",
    "Expected value is negative: the set costs more than it is expected to return.",
    "No better odds: every ticket has exactly the same odds as any other ticket.",
    "Draws are independent: past results do not affect this or any draw.",
    "Special balls are chosen at random (seeded) and carry no guarantee.",
];

/// Pigeonhole needs at most 4 groups when 5 whites are drawn.
pub const MAX_GROUPS: usize = 4;

/// A pinned base design: group sizes (consecutive ranges starting at 1) and tickets.
#[derive(Debug, Clone, Copy)]
pub struct Design {
    pub white_max: u32,
    pub group_sizes: &'static [u32],
    pub tickets: &'static [[u8; 5]],
}

/// Powerball 1..69: groups 1-21, 22-42, 43-53, 54-69 (sizes 21, 21, 11, 16), 63 tickets.
pub const PB_DESIGN: Design = Design {
    white_max: 69,
    group_sizes: &[21, 21, 11, 16],
    tickets: &PB_TICKETS,
};
/// Mega Millions 1..70: groups 1-21, 22-42, 43-55, 56-70 (sizes 21, 21, 13, 15), 65 tickets.
pub const MM_DESIGN: Design = Design {
    white_max: 70,
    group_sizes: &[21, 21, 13, 15],
    tickets: &MM_TICKETS,
};

pub fn design_for(white_max: u32) -> Option<Design> {
    [PB_DESIGN, MM_DESIGN]
        .into_iter()
        .find(|d| d.white_max == white_max)
}

/// A concrete set: groups (lists of numbers) and tickets (sorted whites).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wheel {
    pub n: u32,
    pub groups: Vec<Vec<u32>>,
    pub tickets: Vec<[u32; 5]>,
}

impl Design {
    pub fn wheel(&self) -> Wheel {
        let mut groups = Vec::new();
        let mut next = 1;
        for &s in self.group_sizes {
            groups.push((next..next + s).collect());
            next += s;
        }
        let tickets = self
            .tickets
            .iter()
            .map(|t| {
                let mut w = t.map(u32::from);
                w.sort_unstable();
                w
            })
            .collect();
        Wheel {
            n: self.white_max,
            groups,
            tickets,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Verified {
    pub groups: usize,
    pub group_sizes: Vec<usize>,
    pub required_pairs: usize,
    pub tickets: usize,
}

/// Check the construction: at most 4 groups, the groups partition 1..=n exactly, every ticket
/// has 5 distinct whites in 1..=n, and every pair inside a group is on some ticket.
pub fn verify(w: &Wheel) -> Result<Verified> {
    let n = w.n as usize;
    if w.groups.is_empty() || w.groups.len() > MAX_GROUPS {
        bail!(
            "{} groups: pigeonhole needs 1..={MAX_GROUPS} groups for 5 drawn whites",
            w.groups.len()
        );
    }
    let mut seen = vec![false; n + 1];
    let mut gid = vec![usize::MAX; n + 1];
    for (g, grp) in w.groups.iter().enumerate() {
        for &x in grp {
            if x < 1 || x as usize > n {
                bail!("group {g}: number {x} outside 1..={n}");
            }
            if seen[x as usize] {
                bail!("number {x} is in more than one group");
            }
            seen[x as usize] = true;
            gid[x as usize] = g;
        }
    }
    if let Some(x) = (1..=n).find(|&x| !seen[x]) {
        bail!("number {x} is in no group; groups must cover 1..={n}");
    }
    let mut covered = vec![false; (n + 1) * (n + 1)];
    for (i, t) in w.tickets.iter().enumerate() {
        for (j, &x) in t.iter().enumerate() {
            if x < 1 || x as usize > n {
                bail!("ticket {}: white {x} outside 1..={n}", i + 1);
            }
            if t[..j].contains(&x) {
                bail!("ticket {}: duplicate white {x}", i + 1);
            }
        }
        for a in 0..5 {
            for b in 0..5 {
                covered[t[a] as usize * (n + 1) + t[b] as usize] = true;
            }
        }
    }
    let mut required = 0;
    for grp in &w.groups {
        for (i, &a) in grp.iter().enumerate() {
            for &b in &grp[i + 1..] {
                required += 1;
                if !covered[a as usize * (n + 1) + b as usize] {
                    bail!("pair {a}-{b} (same group) is on no ticket; coverage fails");
                }
            }
        }
    }
    Ok(Verified {
        groups: w.groups.len(),
        group_sizes: w.groups.iter().map(|g| g.len()).collect(),
        required_pairs: required,
        tickets: w.tickets.len(),
    })
}

/// Exhaustive check over all C(n,5) draws: some ticket shares 2+ whites with every draw.
/// Returns the first failing draw, if any. (Used by an ignored test; ~C(69,5) = 11.2M draws.)
pub fn exhaustive_failure(w: &Wheel) -> Option<[u32; 5]> {
    let masks: Vec<u128> = w
        .tickets
        .iter()
        .map(|t| t.iter().fold(0u128, |m, &x| m | 1 << x))
        .collect();
    let n = w.n;
    for a in 1..=n - 4 {
        for b in a + 1..=n - 3 {
            for c in b + 1..=n - 2 {
                for d in c + 1..=n - 1 {
                    let m4 = 1u128 << a | 1 << b | 1 << c | 1 << d;
                    for e in d + 1..=n {
                        let m = m4 | 1 << e;
                        if !masks.iter().any(|t| (t & m).count_ones() >= 2) {
                            return Some([a, b, c, d, e]);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Seeded uniform permutation of 1..=n: `perm[x - 1]` is the new label of number x.
pub fn random_permutation(rng: &mut ChaCha20Rng, n: u32) -> Vec<u32> {
    let mut p: Vec<u32> = (1..=n).collect();
    for i in (1..p.len()).rev() {
        let j = crate::rng::below(rng, i as u32 + 1) as usize;
        p.swap(i, j);
    }
    p
}

/// Rename every number through `perm` (groups and tickets alike). Coverage is invariant under
/// renaming, but callers still re-run [`verify`] on the result.
pub fn relabel(w: &Wheel, perm: &[u32]) -> Wheel {
    let map = |x: u32| perm[x as usize - 1];
    Wheel {
        n: w.n,
        groups: w
            .groups
            .iter()
            .map(|g| {
                let mut v: Vec<u32> = g.iter().map(|&x| map(x)).collect();
                v.sort_unstable();
                v
            })
            .collect(),
        tickets: w
            .tickets
            .iter()
            .map(|t| {
                let mut v = t.map(map);
                v.sort_unstable();
                v
            })
            .collect(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedTicket {
    pub whites: [u32; 5],
    pub special: u32,
    pub crowd_score: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub seed: u64,
    pub permutations_tried: u32,
    /// 0-based index of the chosen permutation.
    pub chosen: u32,
    /// `permutation[x - 1]` = label of base number x (private: recorded under ledger/ only).
    pub permutation: Vec<u32>,
    pub tickets: Vec<PlannedTicket>,
    pub mean_crowd_score: f64,
    pub max_crowd_score: u32,
    pub verified: Verified,
}

/// Build a plan: K seeded permutations of the base design; each is re-verified, gets seeded
/// random specials, and every ticket is crowd-scored. The plan with the lowest **mean** crowd
/// score wins (ties: lower max, then the earlier permutation). Draws are passed to the crowd
/// scorer only for its past-winner flag.
pub fn plan(
    game: &str,
    era: &Era,
    design: &Design,
    seed: u64,
    k: u32,
    draws: &[Draw],
) -> Result<Plan> {
    if k == 0 {
        bail!("--permutations must be at least 1");
    }
    if era.white_max != design.white_max || era.white_count != 5 {
        bail!(
            "era {} is {}/{}; the pinned design is for 5/{}",
            era.id,
            era.white_count,
            era.white_max,
            design.white_max
        );
    }
    let base = design.wheel();
    verify(&base)?;
    let mut rng = crate::rng::seeded(seed);
    let mut best: Option<Plan> = None;
    for i in 0..k {
        let perm = random_permutation(&mut rng, base.n);
        let w = relabel(&base, &perm);
        let verified = verify(&w)?;
        let mut tickets = Vec::with_capacity(w.tickets.len());
        for t in &w.tickets {
            let special = 1 + crate::rng::below(&mut rng, era.bonus_max);
            let r = crowd::score(game, era, t, Some(special), draws)?;
            tickets.push(PlannedTicket {
                whites: *t,
                special,
                crowd_score: r.score,
            });
        }
        let sum: u32 = tickets.iter().map(|t| t.crowd_score).sum();
        let mean = sum as f64 / tickets.len() as f64;
        let max = tickets.iter().map(|t| t.crowd_score).max().unwrap_or(0);
        let better = match &best {
            None => true,
            Some(b) => (mean, max) < (b.mean_crowd_score, b.max_crowd_score),
        };
        if better {
            best = Some(Plan {
                seed,
                permutations_tried: k,
                chosen: i,
                permutation: perm,
                tickets,
                mean_crowd_score: mean,
                max_crowd_score: max,
                verified,
            });
        }
    }
    Ok(best.expect("k >= 1"))
}

/// Total cost in whole USD (ticket count x per-ticket price from eras.toml).
pub fn total_cost(tickets: usize, per_ticket: u32) -> u64 {
    tickets as u64 * per_ticket as u64
}

/// Refuse below the minimum: no partial or "almost" sets.
pub fn check_budget(max_spend: u64, total: u64, tickets: usize) -> Result<()> {
    if max_spend < total {
        bail!(
            "--max-spend ${max_spend} is below the minimum budget ${total} for this set ({tickets} tickets); \
             the guarantee needs every ticket, so no partial or \"almost\" set is offered"
        );
    }
    Ok(())
}

pub const PB_TICKETS: [[u8; 5]; 63] = [
    [1, 2, 4, 9, 18],
    [1, 3, 6, 12, 21],
    [1, 5, 7, 15, 20],
    [1, 8, 10, 13, 16],
    [1, 11, 14, 17, 19],
    [2, 3, 13, 19, 20],
    [2, 5, 8, 11, 12],
    [2, 6, 7, 10, 14],
    [2, 15, 16, 17, 21],
    [3, 4, 7, 11, 16],
    [3, 5, 10, 17, 18],
    [3, 8, 9, 14, 15],
    [4, 5, 13, 14, 21],
    [4, 6, 8, 17, 20],
    [4, 10, 12, 15, 19],
    [5, 6, 9, 16, 19],
    [6, 11, 13, 15, 18],
    [7, 8, 18, 19, 21],
    [7, 9, 12, 13, 17],
    [9, 10, 11, 20, 21],
    [12, 14, 16, 18, 20],
    [22, 23, 29, 30, 31],
    [22, 24, 26, 38, 39],
    [22, 25, 32, 33, 41],
    [22, 27, 35, 36, 42],
    [22, 28, 34, 37, 40],
    [23, 24, 27, 32, 34],
    [23, 25, 36, 39, 40],
    [23, 26, 28, 33, 42],
    [23, 35, 37, 38, 41],
    [24, 25, 29, 37, 42],
    [24, 28, 30, 36, 41],
    [24, 31, 33, 35, 40],
    [25, 26, 30, 34, 35],
    [25, 27, 28, 31, 38],
    [26, 27, 29, 40, 41],
    [26, 31, 32, 36, 37],
    [26, 60, 62, 64, 69],
    [27, 30, 33, 37, 39],
    [28, 29, 32, 35, 39],
    [29, 33, 34, 36, 38],
    [30, 32, 38, 40, 42],
    [31, 34, 39, 41, 42],
    [43, 44, 48, 51, 53],
    [43, 45, 47, 48, 50],
    [43, 46, 48, 49, 52],
    [44, 45, 52, 58, 63],
    [44, 46, 47, 49, 50],
    [45, 46, 49, 51, 53],
    [47, 50, 51, 52, 53],
    [54, 55, 64, 66, 67],
    [54, 56, 58, 62, 68],
    [54, 57, 58, 61, 69],
    [54, 59, 60, 63, 65],
    [55, 56, 59, 66, 69],
    [55, 57, 60, 66, 68],
    [55, 58, 61, 65, 66],
    [55, 61, 62, 63, 66],
    [56, 57, 63, 64, 65],
    [56, 58, 60, 61, 67],
    [57, 59, 62, 65, 67],
    [58, 59, 61, 64, 68],
    [63, 65, 67, 68, 69],
];

pub const MM_TICKETS: [[u8; 5]; 65] = [
    [1, 2, 6, 18, 19],
    [1, 3, 7, 12, 15],
    [1, 4, 5, 10, 14],
    [1, 8, 11, 13, 17],
    [1, 9, 16, 20, 21],
    [1, 57, 64, 66, 68],
    [2, 3, 5, 17, 20],
    [2, 4, 13, 15, 16],
    [2, 7, 8, 14, 21],
    [2, 9, 10, 11, 12],
    [3, 4, 8, 9, 19],
    [3, 6, 10, 13, 21],
    [3, 11, 14, 16, 18],
    [4, 6, 7, 11, 20],
    [4, 12, 17, 18, 21],
    [5, 6, 8, 12, 16],
    [5, 7, 9, 13, 18],
    [5, 11, 15, 19, 21],
    [6, 9, 14, 15, 17],
    [7, 10, 16, 17, 19],
    [8, 10, 15, 18, 20],
    [12, 13, 14, 19, 20],
    [22, 23, 26, 32, 42],
    [22, 24, 30, 39, 40],
    [22, 25, 27, 35, 36],
    [22, 28, 31, 34, 38],
    [22, 29, 33, 37, 41],
    [23, 24, 28, 29, 36],
    [23, 25, 38, 40, 41],
    [23, 27, 31, 37, 39],
    [23, 30, 33, 34, 35],
    [24, 25, 31, 33, 42],
    [24, 26, 35, 37, 38],
    [24, 27, 32, 34, 41],
    [25, 26, 29, 34, 39],
    [25, 28, 30, 32, 37],
    [26, 27, 28, 33, 40],
    [26, 30, 31, 36, 41],
    [27, 29, 30, 38, 42],
    [28, 35, 39, 41, 42],
    [29, 31, 32, 35, 40],
    [32, 33, 36, 38, 39],
    [34, 36, 37, 40, 42],
    [34, 56, 60, 68, 69],
    [43, 44, 46, 47, 50],
    [43, 45, 47, 50, 55],
    [43, 48, 49, 50, 54],
    [43, 50, 51, 52, 53],
    [44, 45, 48, 49, 51],
    [44, 52, 53, 54, 55],
    [45, 46, 52, 53, 54],
    [46, 48, 49, 51, 55],
    [47, 48, 49, 52, 53],
    [47, 51, 54, 59, 67],
    [56, 57, 59, 61, 63],
    [56, 58, 65, 66, 67],
    [56, 62, 63, 64, 70],
    [57, 58, 60, 67, 70],
    [57, 62, 63, 65, 69],
    [58, 59, 60, 64, 65],
    [58, 61, 64, 67, 69],
    [58, 62, 63, 67, 68],
    [59, 61, 65, 68, 70],
    [59, 62, 66, 69, 70],
    [60, 61, 62, 63, 66],
];
