//! v0.2 Step 1 (D-031): private ticket ledger and `grade`.
//!
//! The ledger is `ledger/tickets/tickets.csv` under the gitignored `ledger/` (D-030, R6-04).
//! It is append-only: rows are only ever appended, never rewritten, and grading results are
//! computed on demand and never written back into ticket rows.
//!
//! Grading reports match counts and the prize tier from the encoded current prize charts
//! (`odds::PRIZE_TIERS`). Dollar amounts come only from the private payout cache
//! (`cache/payouts/...`); when no unambiguous cached amount exists the amount is
//! `TIER_LOOKUP_NEEDED`. Nothing here predicts or selects numbers.
use crate::Game;
use crate::data::{Draw, PayoutRow, read_csv};
use crate::eras::{Era, GameDef};
use crate::odds::PRIZE_TIERS;
use crate::payouts::cache_path;
use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const TICKETS_HEADER: &str =
    "ticket_id,created_at,game,draw_date,era_id,w1,w2,w3,w4,w5,special,power_play,cost_usd,note";

/// Placeholder printed when a prize amount is not available from the private cache.
pub const TIER_LOOKUP_NEEDED: &str = "TIER_LOOKUP_NEEDED";

/// Eras whose prize chart matches `odds::PRIZE_TIERS` (current charts only). Tickets for
/// other eras are refused rather than graded against a chart that may not apply.
pub const GRADABLE_ERAS: [&str; 2] = ["pb-2015", "mm-2025"];

pub fn ledger_path(root: &Path) -> PathBuf {
    root.join("ledger").join("tickets").join("tickets.csv")
}

/// One row of the private ticket ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketRow {
    pub ticket_id: String,
    /// UTC, RFC 3339 with `Z`.
    pub created_at: String,
    pub game: String,
    pub draw_date: NaiveDate,
    pub era_id: String,
    pub w1: u32,
    pub w2: u32,
    pub w3: u32,
    pub w4: u32,
    pub w5: u32,
    pub special: u32,
    pub power_play: bool,
    pub cost_usd: u32,
    pub note: String,
}

impl TicketRow {
    pub fn whites(&self) -> [u32; 5] {
        [self.w1, self.w2, self.w3, self.w4, self.w5]
    }
}

/// Unvalidated `ticket add` input.
#[derive(Debug, Clone)]
pub struct NewTicket {
    pub game: Game,
    pub draw_date: NaiveDate,
    pub whites: Vec<u32>,
    pub special: u32,
    pub power_play: bool,
    pub cost_usd: Option<u32>,
    pub note: String,
}

/// Price for one play of `era` (+ Power Play add-on when requested), from eras.toml only.
pub fn expected_cost(era: &Era, power_play: bool) -> Result<u32> {
    if era.price_status != "sourced" {
        bail!(
            "era {}: price_status is {} (not sourced); refusing to price a ticket",
            era.id,
            era.price_status
        );
    }
    let base = era
        .price_usd
        .ok_or_else(|| anyhow!("era {}: no price_usd", era.id))?;
    let addon = if power_play {
        era.power_play_price_usd.ok_or_else(|| {
            anyhow!(
                "era {}: no power_play_price_usd in eras.toml; Power Play not supported",
                era.id
            )
        })?
    } else {
        0
    };
    Ok(base + addon)
}

/// Validate a ticket against eras.toml for its draw date. Returns the era id and cost.
pub fn validate(g: &GameDef, t: &NewTicket) -> Result<(String, u32)> {
    let era = g
        .era_for(t.draw_date)
        .ok_or_else(|| anyhow!("{} has no era on {}", g.id, t.draw_date))?;
    if !GRADABLE_ERAS.contains(&era.id.as_str()) {
        bail!(
            "draw {} is in era {}; tickets are supported only for eras with an encoded prize chart ({})",
            t.draw_date,
            era.id,
            GRADABLE_ERAS.join(", ")
        );
    }
    if !g.expected_draw_day(t.draw_date) {
        bail!(
            "{} is not a scheduled {} draw day (eras.toml schedule)",
            t.draw_date,
            g.name
        );
    }
    if t.whites.len() != era.white_count as usize {
        bail!(
            "need exactly {} white balls, got {}",
            era.white_count,
            t.whites.len()
        );
    }
    for &w in &t.whites {
        if w < 1 || w > era.white_max {
            bail!(
                "white ball {w} out of range 1..={} ({})",
                era.white_max,
                era.id
            );
        }
    }
    let mut sorted = t.whites.clone();
    sorted.sort_unstable();
    if sorted.windows(2).any(|p| p[0] == p[1]) {
        bail!("duplicate white ball in {:?}", t.whites);
    }
    if t.special < 1 || t.special > era.bonus_max {
        bail!(
            "special ball {} out of range 1..={} ({})",
            t.special,
            era.bonus_max,
            era.id
        );
    }
    if t.power_play && t.game != Game::Powerball {
        bail!("--power-play applies only to Powerball");
    }
    let cost = expected_cost(era, t.power_play)?;
    if let Some(c) = t.cost_usd
        && c != cost
    {
        bail!(
            "--cost {c} does not match the eras.toml price ${cost} for {} {}{}",
            era.id,
            t.draw_date,
            if t.power_play { " with Power Play" } else { "" }
        );
    }
    if t.note.contains('\n') || t.note.contains('\r') {
        bail!("--note must be a single line");
    }
    Ok((era.id.clone(), cost))
}

/// Read the ledger, failing closed on a wrong header or unparseable row.
/// A missing file is an empty ledger.
pub fn read_ledger(root: &Path) -> Result<Vec<TicketRow>> {
    let p = ledger_path(root);
    if !p.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
    let header = text.lines().next().unwrap_or("").trim_end_matches('\r');
    if header != TICKETS_HEADER {
        bail!(
            "{}: unexpected header (refusing to read or append): {header:?}",
            p.display()
        );
    }
    read_csv(&p)
}

/// Refuse to write the ledger if `root` is a git work tree and the ledger path is not ignored.
fn ensure_gitignored(root: &Path, path: &Path) -> Result<()> {
    if !root.join(".git").exists() {
        return Ok(());
    }
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("check-ignore")
        .arg("-q")
        .arg(path.strip_prefix(root).unwrap_or(path))
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => bail!(
            "{} is not gitignored in {}; refusing to write private tickets",
            path.display(),
            root.display()
        ),
        Err(e) => bail!("cannot run git check-ignore ({e}); refusing to write private tickets"),
    }
}

/// Validate and append one ticket. Existing bytes are never rewritten (append-only open).
pub fn append(root: &Path, g: &GameDef, t: &NewTicket, now: DateTime<Utc>) -> Result<TicketRow> {
    let (era_id, cost) = validate(g, t)?;
    let existing = read_ledger(root)?;
    let path = ledger_path(root);
    ensure_gitignored(root, &path)?;
    let mut whites = t.whites.clone();
    whites.sort_unstable();
    let row = TicketRow {
        ticket_id: format!("T{:04}", existing.len() + 1),
        created_at: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        game: t.game.id().to_string(),
        draw_date: t.draw_date,
        era_id,
        w1: whites[0],
        w2: whites[1],
        w3: whites[2],
        w4: whites[3],
        w5: whites[4],
        special: t.special,
        power_play: t.power_play,
        cost_usd: cost,
        note: t.note.clone(),
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let new_file = !path.exists();
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("opening {} for append", path.display()))?;
    let mut w = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(Vec::new());
    if new_file {
        w.write_record(TICKETS_HEADER.split(','))?;
    }
    w.serialize(&row)?;
    let bytes = w.into_inner().map_err(|e| anyhow!("csv: {e}"))?;
    f.write_all(&bytes)?;
    f.flush()?;
    f.sync_all()?;
    Ok(row)
}

/// Official draw instant (draw_time_et on draw_date, America/New_York) in UTC.
pub fn draw_instant(g: &GameDef, date: NaiveDate) -> Result<DateTime<Utc>> {
    let t = NaiveTime::parse_from_str(&g.draw_time_et, "%H:%M")
        .with_context(|| format!("draw_time_et {:?}", g.draw_time_et))?;
    chrono_tz::America::New_York
        .from_local_datetime(&date.and_time(t))
        .single()
        .map(|d| d.with_timezone(&Utc))
        .ok_or_else(|| anyhow!("ambiguous draw time {date} {t}"))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GradeStatus {
    /// Draw date not yet in data/<game>/draws.csv.
    Pending,
    Graded,
}

#[derive(Debug, Clone, Serialize)]
pub struct Grade {
    pub ticket: TicketRow,
    pub status: GradeStatus,
    /// created_at is at or after the official draw time (eras.toml draw_time_et).
    pub entered_after_draw: bool,
    pub drawn_whites: Option<[u32; 5]>,
    pub drawn_special: Option<u32>,
    /// Multiplier value from the archive row, if any.
    pub drawn_multiplier: Option<u32>,
    pub multiplier_note: Option<String>,
    pub white_matches: Vec<u32>,
    pub special_match: bool,
    /// e.g. "3+1"; None when pending.
    pub tier: Option<String>,
    pub prize_tier: bool,
    /// Whole dollars from the private cache, or None.
    pub prize_usd: Option<u64>,
    /// "$N (cache ...)", "TIER_LOOKUP_NEEDED (...)", "NO_PRIZE" or "PENDING".
    pub prize: String,
}

/// White matches only count against drawn whites, and the special ball only against the
/// drawn special ball (a ticket special equal to a drawn white is NOT a match).
pub fn match_counts(
    ticket_whites: &[u32; 5],
    ticket_special: u32,
    draw: &Draw,
) -> (Vec<u32>, bool) {
    let drawn = draw.whites();
    let mut m: Vec<u32> = ticket_whites
        .iter()
        .copied()
        .filter(|w| drawn.contains(w))
        .collect();
    m.sort_unstable();
    (m, ticket_special == draw.bonus)
}

pub fn is_prize_tier(white: u32, special: bool) -> bool {
    PRIZE_TIERS.contains(&(white, special))
}

/// Read the national-scope cache file for one draw, if present.
pub fn read_cache(root: &Path, game: Game, date: NaiveDate) -> Result<Vec<PayoutRow>> {
    let p = cache_path(root, game.id(), date, "national");
    if !p.exists() {
        return Ok(Vec::new());
    }
    read_csv(&p)
}

/// Pick an amount from cached rows: exactly one distinct `prize_usd` among the rows for this
/// tier and product, else TIER_LOOKUP_NEEDED with the reason. Never computes an amount.
pub fn lookup_prize(
    rows: &[PayoutRow],
    game: Game,
    white: u32,
    special: bool,
    power_play: bool,
) -> (Option<u64>, String) {
    let product_ok = |p: &str| match game {
        Game::Powerball => p == if power_play { "power_play" } else { "base" },
        // mm-2025: built-in multiplier rows ("builtin_*") or base; never megaplier.
        Game::Megamillions => p == "base" || p.starts_with("builtin_"),
    };
    let tier_rows: Vec<&PayoutRow> = rows
        .iter()
        .filter(|r| {
            r.white_matched == white && r.bonus_matched == special && product_ok(&r.product)
        })
        .collect();
    if tier_rows.is_empty() {
        let why = if rows.is_empty() {
            "no private payout cache for this draw"
        } else {
            "tier/product not in private payout cache"
        };
        return (None, format!("{TIER_LOOKUP_NEEDED} ({why})"));
    }
    let mut amounts: Vec<u64> = tier_rows.iter().filter_map(|r| r.prize_usd).collect();
    amounts.sort_unstable();
    amounts.dedup();
    match amounts.as_slice() {
        [a] if tier_rows.iter().all(|r| r.prize_usd.is_some()) => (
            Some(*a),
            format!("${a} (private cache, product {})", tier_rows[0].product),
        ),
        [] => (
            None,
            format!(
                "{TIER_LOOKUP_NEEDED} (cache text: {:?})",
                tier_rows[0].prize_text
            ),
        ),
        _ => (
            None,
            format!("{TIER_LOOKUP_NEEDED} (cache has more than one amount for this tier)"),
        ),
    }
}

/// Grade one ticket against the archive (and optional cached payout rows for its draw).
pub fn grade(
    g: &GameDef,
    t: &TicketRow,
    draws: &[Draw],
    cache_rows: &[PayoutRow],
) -> Result<Grade> {
    let game = match t.game.as_str() {
        "powerball" => Game::Powerball,
        "megamillions" => Game::Megamillions,
        other => bail!("ticket {}: unknown game {other:?}", t.ticket_id),
    };
    let created: DateTime<Utc> = DateTime::parse_from_rfc3339(&t.created_at)
        .with_context(|| format!("ticket {} created_at", t.ticket_id))?
        .with_timezone(&Utc);
    let entered_after_draw = created >= draw_instant(g, t.draw_date)?;
    let Some(draw) = draws.iter().find(|d| d.draw_date == t.draw_date) else {
        return Ok(Grade {
            ticket: t.clone(),
            status: GradeStatus::Pending,
            entered_after_draw,
            drawn_whites: None,
            drawn_special: None,
            drawn_multiplier: None,
            multiplier_note: None,
            white_matches: Vec::new(),
            special_match: false,
            tier: None,
            prize_tier: false,
            prize_usd: None,
            prize: "PENDING (draw not in archive yet)".into(),
        });
    };
    let (white_matches, special_match) = match_counts(&t.whites(), t.special, draw);
    let w = white_matches.len() as u32;
    let prize_tier = is_prize_tier(w, special_match);
    let multiplier_note = match (game, t.power_play, draw.multiplier) {
        (Game::Powerball, true, Some(m)) => Some(format!("Power Play {m}x (archive)")),
        (Game::Powerball, true, None) => {
            Some("Power Play multiplier MISSING in archive for this draw".into())
        }
        (Game::Powerball, false, _) => None,
        (Game::Megamillions, _, Some(m)) => Some(format!("multiplier {m}x (archive)")),
        (Game::Megamillions, _, None) => {
            Some("built-in multiplier: MISSING in archive (NY data has no value)".into())
        }
    };
    let (prize_usd, prize) = if !prize_tier {
        (None, "NO_PRIZE".to_string())
    } else {
        lookup_prize(cache_rows, game, w, special_match, t.power_play)
    };
    Ok(Grade {
        ticket: t.clone(),
        status: GradeStatus::Graded,
        entered_after_draw,
        drawn_whites: Some(draw.whites()),
        drawn_special: Some(draw.bonus),
        drawn_multiplier: draw.multiplier,
        multiplier_note,
        white_matches,
        special_match,
        tier: Some(format!("{w}+{}", u8::from(special_match))),
        prize_tier,
        prize_usd,
        prize,
    })
}
