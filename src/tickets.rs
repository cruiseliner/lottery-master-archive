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

pub const VOIDS_HEADER: &str = "ticket_id,voided_at,reason";

/// Append-only void records (D-031). Named `tickets*.csv` so the R6-04 guard also matches it.
pub fn voids_path(root: &Path) -> PathBuf {
    root.join("ledger")
        .join("tickets")
        .join("tickets_voids.csv")
}

/// One void record: marks an existing ticket row as voided without touching it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoidRow {
    pub ticket_id: String,
    /// UTC, RFC 3339 with `Z`.
    pub voided_at: String,
    pub reason: String,
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
    let rows: Vec<TicketRow> = read_checked(&ledger_path(root), TICKETS_HEADER)?;
    let mut seen = std::collections::HashSet::new();
    for r in &rows {
        if !seen.insert(r.ticket_id.as_str()) {
            bail!(
                "{}: duplicate ticket_id {} (refusing to read or append)",
                ledger_path(root).display(),
                r.ticket_id
            );
        }
    }
    Ok(rows)
}

/// Read void records, failing closed on a wrong header, an unknown ticket id, or a duplicate.
pub fn read_voids(root: &Path) -> Result<Vec<VoidRow>> {
    let voids: Vec<VoidRow> = read_checked(&voids_path(root), VOIDS_HEADER)?;
    if voids.is_empty() {
        return Ok(voids);
    }
    let tickets = read_ledger(root)?;
    let mut seen = std::collections::HashSet::new();
    for v in &voids {
        if !tickets.iter().any(|t| t.ticket_id == v.ticket_id) {
            bail!("void record references unknown ticket {}", v.ticket_id);
        }
        if !seen.insert(v.ticket_id.clone()) {
            bail!("ticket {} is voided more than once", v.ticket_id);
        }
    }
    Ok(voids)
}

fn read_checked<T: for<'de> Deserialize<'de>>(p: &Path, expected: &str) -> Result<Vec<T>> {
    // Missing or 0-byte (crash after open, before the first write) = empty, like append_record.
    match std::fs::metadata(p) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("stat {}", p.display())),
        Ok(m) if m.len() == 0 => return Ok(Vec::new()),
        Ok(_) => {}
    }
    // A partial last line (crash mid-append) fails closed on every read, not just on append.
    ensure_ends_with_newline(p)?;
    let text = std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
    let header = text.lines().next().unwrap_or("").trim_end_matches('\r');
    if header != expected {
        bail!(
            "{}: unexpected header (refusing to read or append): {header:?}",
            p.display()
        );
    }
    read_csv(p)
}

/// Refuse to append to a file whose last line is incomplete (e.g. a crash mid-write), so a new
/// row can never be glued onto a partial one. Empty / missing files are fine.
pub fn ensure_ends_with_newline(path: &Path) -> Result<()> {
    match std::fs::read(path) {
        Ok(b) if !b.is_empty() && b.last() != Some(&b'\n') => bail!(
            "{} does not end in a newline (partial last line?); refusing to append. \
             Inspect and repair the private file by hand",
            path.display()
        ),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

/// Append one CSV record (header first if the file is new) with an append-only open:
/// newline check first, then the whole record in ONE `write_all`, then `sync_all` (fsync).
fn append_record<T: Serialize>(path: &Path, header: &str, row: &T) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    ensure_ends_with_newline(path)?;
    let new_file = std::fs::metadata(path)
        .map(|m| m.len() == 0)
        .unwrap_or(true);
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("opening {} for append", path.display()))?;
    let mut w = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(Vec::new());
    if new_file {
        w.write_record(header.split(','))?;
    }
    w.serialize(row)?;
    let bytes = w.into_inner().map_err(|e| anyhow!("csv: {e}"))?;
    f.write_all(&bytes)
        .with_context(|| format!("appending to {}", path.display()))?;
    f.sync_all()
        .with_context(|| format!("fsync {}", path.display()))?;
    Ok(())
}

/// Void a ticket by appending a record to `tickets_voids.csv`. The ticket row is never touched.
/// Refuses an unknown id, an already-voided id, and an empty or multi-line reason.
pub fn void(root: &Path, ticket_id: &str, reason: &str, now: DateTime<Utc>) -> Result<VoidRow> {
    let reason = reason.trim();
    if reason.is_empty() {
        bail!("--reason is required");
    }
    if reason.contains('\n') || reason.contains('\r') {
        bail!("--reason must be a single line");
    }
    let tickets = read_ledger(root)?;
    if !tickets.iter().any(|t| t.ticket_id == ticket_id) {
        bail!("no ticket {ticket_id} in the ledger");
    }
    let voids = read_voids(root)?;
    if let Some(v) = voids.iter().find(|v| v.ticket_id == ticket_id) {
        bail!("ticket {ticket_id} is already voided ({})", v.voided_at);
    }
    let path = voids_path(root);
    ensure_gitignored(root, &path)?;
    let row = VoidRow {
        ticket_id: ticket_id.to_string(),
        voided_at: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        reason: reason.to_string(),
    };
    append_record(&path, VOIDS_HEADER, &row)?;
    Ok(row)
}

/// Git guard (D-031). Finds the enclosing work tree with `git rev-parse --show-toplevel`
/// (so a `--root` inside a subdirectory is still checked) and refuses unless
/// `git check-ignore` says the ledger path is ignored. Outside any git work tree there is
/// nothing to commit to, so the write is allowed with a warning on stderr (never silently).
/// If git cannot be run at all, the write is refused.
pub fn ensure_gitignored(root: &Path, path: &Path) -> Result<()> {
    match guard_status(root, path)? {
        Guard::Ignored => Ok(()),
        Guard::NotInRepo => {
            eprintln!(
                "warning: {} is not inside a git work tree; gitignore guard not applicable",
                root.display()
            );
            Ok(())
        }
        Guard::NotIgnored(top) => bail!(
            "{} is not gitignored in work tree {}; refusing to write private tickets",
            path.display(),
            top
        ),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Guard {
    Ignored,
    NotInRepo,
    /// Inside the work tree with this top-level, but the path is not ignored (or is tracked).
    NotIgnored(String),
}

pub fn guard_status(root: &Path, path: &Path) -> Result<Guard> {
    guard_status_with("git", root, path)
}

/// `guard_status` with an explicit git program (tests use a missing or failing fake git).
/// Only git's own "not a git repository" answer means NotInRepo; any other git failure
/// (cannot spawn, crash, safe.directory refusal, ...) is an error, so the write is refused.
pub fn guard_status_with(git: &str, root: &Path, path: &Path) -> Result<Guard> {
    let top = std::process::Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-toplevel"])
        .env("LC_ALL", "C") // stable English "not a git repository" text
        .output()
        .map_err(|e| anyhow!("cannot run git ({e}); refusing to write private tickets"))?;
    if !top.status.success() {
        let err = String::from_utf8_lossy(&top.stderr);
        if err.contains("not a git repository") {
            return Ok(Guard::NotInRepo);
        }
        bail!(
            "git rev-parse failed ({}): {}; refusing to write private tickets",
            top.status,
            err.trim()
        );
    }
    let top = String::from_utf8_lossy(&top.stdout).trim().to_string();
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    // check-ignore: 0 = ignored; 1 = not ignored OR tracked (tracked files are never reported
    // as ignored), both refused; anything else (128 = fatal) is an error, also refused.
    let status = std::process::Command::new(git)
        .arg("-C")
        .arg(&top)
        .args(["check-ignore", "-q", "--"])
        .arg(&abs)
        .status()
        .map_err(|e| anyhow!("cannot run git check-ignore ({e}); refusing to write"))?;
    match status.code() {
        Some(0) => Ok(Guard::Ignored),
        Some(1) => Ok(Guard::NotIgnored(top)),
        _ => bail!("git check-ignore failed ({status}); refusing to write private tickets"),
    }
}

/// Validate and append one ticket. Existing bytes are never rewritten (append-only open).
/// Refuses a draw that is already in the archive (`draws`), so results can't be back-filled.
/// After the sales cutoff but before the draw is archived the ticket is accepted and shows
/// as LATE_ENTRY (D-030 / D-031).
pub fn append(
    root: &Path,
    g: &GameDef,
    t: &NewTicket,
    draws: &[Draw],
    now: DateTime<Utc>,
) -> Result<TicketRow> {
    let (era_id, cost) = validate(g, t)?;
    if draws.iter().any(|d| d.draw_date == t.draw_date) {
        bail!(
            "{} {} is already in the archive (data/); refusing a back-dated ticket (D-030 / D-031)",
            g.name,
            t.draw_date
        );
    }
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
    append_record(&path, TICKETS_HEADER, &row)?;
    Ok(row)
}

/// Ledger sales cutoff, 22:00 America/New_York on the draw date (D-030; the owner's rule, used
/// for both games; actual retailer cutoffs vary by state).
pub const SALES_CUTOFF_ET: (u32, u32) = (22, 0);

pub fn sales_cutoff(date: NaiveDate) -> Result<DateTime<Utc>> {
    let t = NaiveTime::from_hms_opt(SALES_CUTOFF_ET.0, SALES_CUTOFF_ET.1, 0).expect("valid");
    chrono_tz::America::New_York
        .from_local_datetime(&date.and_time(t))
        .single()
        .map(|d| d.with_timezone(&Utc))
        .ok_or_else(|| anyhow!("ambiguous cutoff {date} {t}"))
}

fn parse_utc(s: &str, what: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(s)
        .with_context(|| format!("{what} {s:?}"))?
        .with_timezone(&Utc))
}

/// LATE_ENTRY: created_at at or after the sales cutoff on the draw date (D-030).
pub fn is_late_entry(t: &TicketRow) -> Result<bool> {
    Ok(parse_utc(&t.created_at, "created_at")? >= sales_cutoff(t.draw_date)?)
}

/// LATE_VOID: voided at or after the sales cutoff on the ticket's draw date. Stricter than
/// "after the draw is archived": once sales close, a void could hide a result.
pub fn is_late_void(t: &TicketRow, v: &VoidRow) -> Result<bool> {
    Ok(parse_utc(&v.voided_at, "voided_at")? >= sales_cutoff(t.draw_date)?)
}

/// Chance baseline for average white matches per ticket: k*k/n (25/69 PB, 25/70 MM).
pub fn chance_white_baseline(era: &Era) -> f64 {
    crate::odds::Matrix::new(era.white_count, era.white_max, era.bonus_max).expected_white_matches()
}

/// Chance standard deviation of ONE ticket's white matches (hypergeometric):
/// PB 69 ≈ 0.562, MM 70 ≈ 0.559. The spread of an average over n tickets is this / sqrt(n).
pub fn chance_white_sd(era: &Era) -> f64 {
    crate::odds::Matrix::new(era.white_count, era.white_max, era.bonus_max)
        .variance_white_matches()
        .sqrt()
}

/// Below this many eligible tickets the average is labeled "too few tickets".
pub const MIN_TICKETS_FOR_MEASUREMENT: usize = 30;

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
    /// LATE_ENTRY: created_at at/after the 22:00 ET sales cutoff (D-030). Not scorecard-eligible.
    pub late_entry: bool,
    /// Void record for this ticket, if any.
    pub void: Option<VoidRow>,
    /// LATE_VOID: voided at/after the sales cutoff. Such tickets stay graded and counted.
    pub late_void: bool,
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
    let all_have_amount = tier_rows.iter().all(|r| r.prize_usd.is_some());
    match amounts.as_slice() {
        [a] if all_have_amount => (
            Some(*a),
            format!("${a} (private cache, product {})", tier_rows[0].product),
        ),
        [_, ..] if !all_have_amount => (
            None,
            format!(
                "{TIER_LOOKUP_NEEDED} (cache rows for this tier mix dollar amounts and text-only entries)"
            ),
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
    t: &TicketRow,
    draws: &[Draw],
    cache_rows: &[PayoutRow],
    void: Option<&VoidRow>,
) -> Result<Grade> {
    let game = match t.game.as_str() {
        "powerball" => Game::Powerball,
        "megamillions" => Game::Megamillions,
        other => bail!("ticket {}: unknown game {other:?}", t.ticket_id),
    };
    let late_entry = is_late_entry(t).with_context(|| format!("ticket {}", t.ticket_id))?;
    let late_void = match void {
        Some(v) => is_late_void(t, v)?,
        None => false,
    };
    let void = void.cloned();
    let Some(draw) = draws.iter().find(|d| d.draw_date == t.draw_date) else {
        return Ok(Grade {
            ticket: t.clone(),
            status: GradeStatus::Pending,
            late_entry,
            void,
            late_void,
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
        late_entry,
        void,
        late_void,
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
