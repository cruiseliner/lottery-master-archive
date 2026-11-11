//! Anti-crowd ("uniqueness") heuristics for a single combination (D-032, v0.2 Step 2).
//!
//! The score estimates how *popular* a combination is likely to be with other players, so a
//! jackpot win is less likely to be shared. It is **not** a probability: every combination has
//! exactly the same odds of being drawn. Weights are judgment, not measured data. Factor
//! directions that come from published research are marked `basis: "Wang et al. 2016"`
//! (Judgment and Decision Making 11(3):243–259, doi:10.1017/S1930297500003089); the rest are
//! marked `"judgment"`. No draw frequency, hot/cold, overdue or other prediction signal is used;
//! archived draws are read only for the exact-past-winner check.

use crate::data::Draw;
use crate::eras::Era;
use anyhow::{Result, bail};
use chrono::NaiveDate;
use serde::Serialize;

/// Printed with every result, text and JSON.
pub const DISCLAIMER: &str = "This does NOT make a ticket likelier to win: every combination has exactly the same odds. \
The score only aims to lower the chance of splitting a jackpot with other players if this combination wins. \
Heuristic weights (judgment), not measured player data.";

/// Reported score range: 0..=100. Raw factor points (max [`RAW_MAX`]) are rescaled,
/// `score = round(raw * 100 / RAW_MAX)`, so the bands keep their meaning as a share of the max.
pub const MAX_SCORE: u32 = 100;
/// Sum of factor maxima (Adam t51s258: birthdays and sequence lowered to 20 each).
pub const RAW_MAX: u32 = 90;

/// Rescale raw points to 0..=100, rounding half up.
pub fn rescale(raw: u32) -> u32 {
    (raw.min(RAW_MAX) * MAX_SCORE + RAW_MAX / 2) / RAW_MAX
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Factor {
    pub id: &'static str,
    pub points: u32,
    pub max: u32,
    pub reason: String,
    /// "Wang et al. 2016" (direction from published research) or "judgment".
    pub basis: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrowdReport {
    pub game: String,
    pub era_id: String,
    pub whites: [u32; 5],
    pub special: Option<u32>,
    /// Rescaled 0..=100 (see [`rescale`]).
    pub score: u32,
    pub max_score: u32,
    /// Sum of factor points before rescaling, out of `raw_max`.
    pub raw_points: u32,
    pub raw_max: u32,
    pub band: &'static str,
    pub factors: Vec<Factor>,
    /// Archived draws (main or Double Play) with these exact five whites (popularity only).
    pub past_winners: Vec<PastWinner>,
    pub disclaimer: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PastWinner {
    pub draw_date: NaiveDate,
    /// "main" or "double_play" (Powerball Double Play; in `data/` only from 2026-06-03).
    pub draw: &'static str,
}

const WANG: &str = "Wang et al. 2016";
const JUDGMENT: &str = "judgment";

/// Validate and sort a combination against an era's matrix.
pub fn validate(era: &Era, whites: &[u32], special: Option<u32>) -> Result<[u32; 5]> {
    if whites.len() != 5 {
        bail!("need exactly 5 white balls, got {}", whites.len());
    }
    let mut w: [u32; 5] = [whites[0], whites[1], whites[2], whites[3], whites[4]];
    w.sort_unstable();
    if w.windows(2).any(|p| p[0] == p[1]) {
        bail!("white balls must be distinct");
    }
    if w[0] < 1 || w[4] > era.white_max {
        bail!(
            "white balls must be 1..={} for era {}",
            era.white_max,
            era.id
        );
    }
    if let Some(s) = special
        && (s < 1 || s > era.bonus_max)
    {
        bail!(
            "special ball must be 1..={} for era {}",
            era.bonus_max,
            era.id
        );
    }
    Ok(w)
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Dates, months, birthdays: whites of 31 or below (and all 12 or below).
fn birthdays(w: &[u32; 5]) -> Factor {
    let k = w.iter().filter(|&&x| x <= 31).count();
    let base = [0, 0, 0, 4, 10, 16][k];
    let months = w.iter().all(|&x| x <= 12);
    let points = base + if months { 4 } else { 0 };
    let mut reason = format!("{} of 5 whites are 31 or below (dates/birthdays)", k);
    if months {
        reason.push_str("; all 12 or below (months)");
    }
    Factor {
        id: "birthdays",
        points,
        max: 20,
        reason,
        basis: WANG,
    }
}

/// Numeric sequences: consecutive runs and equal spacing.
fn sequence(w: &[u32; 5]) -> Factor {
    let (mut run, mut best) = (1u32, 1u32);
    for p in w.windows(2) {
        run = if p[1] == p[0] + 1 { run + 1 } else { 1 };
        best = best.max(run);
    }
    let step = w[1] - w[0];
    let arithmetic = w.windows(2).all(|p| p[1] - p[0] == step);
    let (points, reason) = if arithmetic {
        (
            20,
            format!("all five equally spaced (step {step}): a numeric sequence"),
        )
    } else {
        match best {
            4 => (14, "4 consecutive numbers".to_string()),
            3 => (6, "3 consecutive numbers".to_string()),
            _ => (
                0,
                "no run of 3+ consecutive numbers, not equally spaced".to_string(),
            ),
        }
    };
    Factor {
        id: "sequence",
        points,
        max: 20,
        reason,
        basis: WANG,
    }
}

/// All in one decade (1-9, 10-19, ...).
fn same_decade(w: &[u32; 5]) -> Factor {
    let same = w.iter().all(|&x| x / 10 == w[0] / 10);
    let lo = (w[0] / 10) * 10;
    Factor {
        id: "same_decade",
        points: if same { 10 } else { 0 },
        max: 10,
        reason: if same {
            format!("all five in {}-{}", lo.max(1), lo + 9)
        } else {
            "spans more than one decade".to_string()
        },
        basis: JUDGMENT,
    }
}

/// Same last digit (a column pattern, e.g. 3 13 23 33 43).
fn last_digit(w: &[u32; 5]) -> Factor {
    let (best, digit) = (0..10)
        .map(|d| (w.iter().filter(|&&x| x % 10 == d).count(), d))
        .max()
        .unwrap_or((0, 0));
    let points = match best {
        5 => 10,
        4 => 5,
        _ => 0,
    };
    Factor {
        id: "last_digit",
        points,
        max: 10,
        reason: if best >= 2 {
            format!("{best} of 5 whites end in {digit}")
        } else {
            "all last digits differ".to_string()
        },
        basis: JUDGMENT,
    }
}

/// "Lucky" 7 and 11 (whites and special), and round-number patterns (4+ multiples of 5).
fn lucky(w: &[u32; 5], special: Option<u32>) -> Factor {
    let lucky_whites = w.iter().filter(|&&x| x == 7 || x == 11).count();
    let lucky_special = special.is_some_and(|s| s == 7 || s == 11);
    let n = lucky_whites + usize::from(lucky_special);
    let fives = w.iter().filter(|&&x| x % 5 == 0).count();
    let round = fives >= 4;
    let points = (3 * n as u32).min(6) + if round { 4 } else { 0 };
    let mut reason = format!("{} of 7/11", plural(n, "use", "uses"));
    if round {
        reason.push_str(&format!(
            "; {fives} of 5 whites are multiples of 5 (round-number pattern)"
        ));
    }
    Factor {
        id: "lucky_numbers",
        points,
        max: 10,
        reason,
        basis: "Wang et al. 2016 (7); judgment (11, multiples of 5)",
    }
}

/// Very low sum of whites (clustered low numbers).
fn low_sum(era: &Era, w: &[u32; 5]) -> Factor {
    let sum: u32 = w.iter().sum();
    // Mean sum under uniform random choice: k * (n + 1) / 2.
    let mean = era.white_count as f64 * (era.white_max as f64 + 1.0) / 2.0;
    let ratio = sum as f64 / mean;
    let points = if ratio < 0.5 {
        10
    } else if ratio < 0.7 {
        5
    } else {
        0
    };
    Factor {
        id: "low_sum",
        points,
        max: 10,
        reason: format!(
            "sum {sum} vs {mean:.1} for a uniformly random pick ({:.0}%)",
            ratio * 100.0
        ),
        basis: "Wang et al. 2016 (low numbers more popular); judgment (thresholds)",
    }
}

/// Exact repeat of a past drawn combination (five whites), in the main draw or, for Powerball,
/// the Double Play draw. Popularity concern only: some players replay past winners; a past winner
/// has the same odds as any other combination. Double Play numbers exist in `data/` only from
/// 2026-06-03 (2021-08-23..2026-06-01 is MISSING), so older Double Play repeats are not found.
fn past_winner(w: &[u32; 5], draws: &[Draw]) -> (Factor, Vec<PastWinner>) {
    let same = |mut x: [u32; 5]| {
        x.sort_unstable();
        x == *w
    };
    let mut hits = Vec::new();
    for d in draws {
        if same(d.whites()) {
            hits.push(PastWinner {
                draw_date: d.draw_date,
                draw: "main",
            });
        }
        if let Some((dp, _)) = d.dp()
            && same(dp)
        {
            hits.push(PastWinner {
                draw_date: d.draw_date,
                draw: "double_play",
            });
        }
    }
    let dp_checked = draws.iter().filter(|d| d.dp().is_some()).count();
    let f = Factor {
        id: "past_winner",
        points: if hits.is_empty() { 0 } else { 10 },
        max: 10,
        reason: if hits.is_empty() {
            format!(
                "not a past drawn combination ({} main draws, all eras, and {} Double Play draws checked)",
                draws.len(),
                dp_checked
            )
        } else {
            format!(
                "same five whites as {} (popularity concern only; odds unchanged)",
                hits.iter()
                    .map(|h| match h.draw {
                        "double_play" => format!("the Double Play draw on {}", h.draw_date),
                        _ => format!("the draw on {}", h.draw_date),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
        basis: JUDGMENT,
    };
    (f, hits)
}

pub fn band(score: u32) -> &'static str {
    match score {
        0..20 => "low",
        20..50 => "medium",
        _ => "high",
    }
}

/// Score one combination. `draws` are the archived draws for the same game, used only for the
/// exact-past-winner check (main and Double Play numbers).
pub fn score(
    game: &str,
    era: &Era,
    whites: &[u32],
    special: Option<u32>,
    draws: &[Draw],
) -> Result<CrowdReport> {
    let w = validate(era, whites, special)?;
    let (pw, past_winners) = past_winner(&w, draws);
    let factors = vec![
        birthdays(&w),
        sequence(&w),
        same_decade(&w),
        last_digit(&w),
        lucky(&w, special),
        low_sum(era, &w),
        pw,
    ];
    debug_assert_eq!(factors.iter().map(|f| f.max).sum::<u32>(), RAW_MAX);
    let raw_points = factors.iter().map(|f| f.points).sum::<u32>();
    let score = rescale(raw_points);
    Ok(CrowdReport {
        game: game.to_string(),
        era_id: era.id.clone(),
        whites: w,
        special,
        score,
        max_score: MAX_SCORE,
        raw_points,
        raw_max: RAW_MAX,
        band: band(score),
        factors,
        past_winners,
        disclaimer: DISCLAIMER,
    })
}
