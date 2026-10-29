//! `lottotool check`: data integrity + staleness checks. Every check reports
//! PASS / FAIL / WARN / SKIP, and SKIP always carries a reason.
use crate::Game;
use crate::data::{
    self, DRAWS_HEADER, Draw, INGEST_LOG_HEADER, IngestLogRow, MISSING_HEADER, MissingRow,
};
use crate::eras::{ErasFile, GameDef};
use crate::ingest_log;
use crate::ny;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::America::New_York;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub game: String,
    pub check: String,
    pub status: Status,
    pub detail: String,
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Skip the staleness check (used by CI before committing fresh data).
    pub no_stale: bool,
    /// Report staleness as WARN instead of FAIL (06:17 ET soft gate; R2-05).
    pub stale_as_warn: bool,
    /// Override "now" (tests / reproducibility).
    pub now: Option<DateTime<Utc>>,
    /// When set, cross-check the N most recent draws per game against official
    /// winning-number pages (local / workflow_dispatch only; never on schedule).
    pub crosscheck: Option<usize>,
}

fn r(game: &str, check: &str, status: Status, detail: impl Into<String>) -> CheckResult {
    CheckResult {
        game: game.into(),
        check: check.into(),
        status,
        detail: detail.into(),
    }
}

/// Most recent expected draw date whose data is due by `now`.
///
/// Due instant = **06:00 America/New_York on the calendar day after the draw**
/// (wall-clock; R-10). Equivalent to midnight + 30h outside DST spring-forward.
/// `stale_grace_hours` remains in eras.toml as documentation of the observed lag.
pub fn latest_due_draw(g: &GameDef, now: DateTime<Utc>) -> Option<NaiveDate> {
    let today_et = now.with_timezone(&New_York).date_naive();
    let mut d = today_et;
    for _ in 0..30 {
        if g.expected_draw_day(d) {
            let next = d.succ_opt()?;
            let due_local = New_York
                .from_local_datetime(&next.and_hms_opt(6, 0, 0)?)
                .earliest()?;
            if due_local.with_timezone(&Utc) <= now {
                return Some(d);
            }
        }
        d = d.pred_opt()?;
    }
    None
}

pub fn run(root: &Path, eras: &ErasFile, games: &[Game], opts: &Options) -> Vec<CheckResult> {
    let mut out = Vec::new();
    for &game in games {
        let gid = game.id();
        let g = match eras.game(gid) {
            Ok(g) => g,
            Err(e) => {
                out.push(r(gid, "eras", Status::Fail, e.to_string()));
                continue;
            }
        };
        out.push(r(
            gid,
            "eras.status_labels",
            Status::Pass,
            format!(
                "{} eras; {} sourced first_draw, {} inferred, {} missing",
                g.era.len(),
                g.era
                    .iter()
                    .filter(|e| e.first_draw_status == "sourced")
                    .count(),
                g.era
                    .iter()
                    .filter(|e| e.first_draw_status == "inferred")
                    .count(),
                g.era
                    .iter()
                    .filter(|e| e.first_draw_status == "missing")
                    .count()
            ),
        ));
        let dp = game.draws_path(root);
        // Schema header
        match data::header_of(&dp) {
            Ok(h) if h == DRAWS_HEADER => {
                out.push(r(gid, "draws.header", Status::Pass, "matches schema"))
            }
            Ok(h) => out.push(r(gid, "draws.header", Status::Fail, format!("got {h:?}"))),
            Err(e) => {
                out.push(r(gid, "draws.header", Status::Fail, format!("{e:#}")));
                for c in [
                    "draws.parse",
                    "draws.order_unique",
                    "draws.ranges",
                    "draws.era_ids",
                    "draws.schedule",
                    "draws.whites_ascending",
                    "missing.coverage",
                    "staleness",
                    "crosscheck",
                ] {
                    out.push(r(gid, c, Status::Skip, "draws.csv not readable"));
                }
                continue;
            }
        }
        let draws: Vec<Draw> = match data::read_csv(&dp) {
            Ok(d) => {
                out.push(r(
                    gid,
                    "draws.parse",
                    Status::Pass,
                    format!("{} rows", d.len()),
                ));
                d
            }
            Err(e) => {
                out.push(r(gid, "draws.parse", Status::Fail, format!("{e:#}")));
                continue;
            }
        };
        if draws.is_empty() {
            out.push(r(gid, "draws.nonempty", Status::Fail, "0 rows"));
            continue;
        }
        out.extend(check_order_unique(gid, &draws));
        out.extend(check_ranges(gid, g, &draws));
        out.extend(check_era_ids(gid, g, &draws));
        out.extend(check_schedule(gid, g, &draws));
        out.extend(check_whites_ascending(gid, &draws));
        out.extend(check_missing(root, game, g, &draws));
        if opts.no_stale {
            out.push(r(gid, "staleness", Status::Skip, "--no-stale"));
        } else {
            let now = opts.now.unwrap_or_else(Utc::now);
            let mut st = check_stale(gid, g, &draws, now);
            if opts.stale_as_warn && st.status == Status::Fail {
                st.status = Status::Warn;
                st.detail = format!("(soft/06:17) {}", st.detail);
            }
            out.push(st);
        }
        match opts.crosscheck {
            None => out.push(r(
                gid,
                "crosscheck",
                Status::Skip,
                "not requested (pass --crosscheck N for a local official-source check)",
            )),
            Some(n) => out.extend(crate::crosscheck::run(game, &draws, n)),
        }
    }
    out.extend(check_ingest_log(root, games));
    out
}

fn check_order_unique(gid: &str, draws: &[Draw]) -> Vec<CheckResult> {
    let mut seen = HashSet::new();
    let dups: Vec<String> = draws
        .iter()
        .filter(|d| !seen.insert(d.draw_date))
        .map(|d| d.draw_date.to_string())
        .collect();
    let sorted = draws.windows(2).all(|w| w[0].draw_date < w[1].draw_date);
    vec![
        if dups.is_empty() {
            r(
                gid,
                "draws.unique_dates",
                Status::Pass,
                "no duplicate draw_date",
            )
        } else {
            r(
                gid,
                "draws.unique_dates",
                Status::Fail,
                format!("duplicates: {}", dups.join(" ")),
            )
        },
        if sorted {
            r(gid, "draws.sorted", Status::Pass, "ascending by draw_date")
        } else {
            r(gid, "draws.sorted", Status::Fail, "not strictly ascending")
        },
    ]
}

fn check_ranges(gid: &str, g: &GameDef, draws: &[Draw]) -> Vec<CheckResult> {
    let mut bad = Vec::new();
    let mut inferred_rows = 0usize;
    for d in draws {
        let Some(e) = g.era_for(d.draw_date) else {
            bad.push(format!("{} no era", d.draw_date));
            continue;
        };
        if e.matrix_status != "sourced" {
            inferred_rows += 1;
        }
        let w = d.whites();
        let distinct: HashSet<_> = w.iter().collect();
        if distinct.len() != 5 || w.iter().any(|&x| x < 1 || x > e.white_max) {
            bad.push(format!(
                "{} whites {:?} vs 1..={}",
                d.draw_date, w, e.white_max
            ));
        }
        if d.bonus < 1 || d.bonus > e.bonus_max {
            bad.push(format!(
                "{} bonus {} vs 1..={}",
                d.draw_date, d.bonus, e.bonus_max
            ));
        }
        if let Some((dw, db)) = d.dp() {
            let dd: HashSet<_> = dw.iter().collect();
            if dd.len() != 5
                || dw.iter().any(|&x| x < 1 || x > e.white_max)
                || db < 1
                || db > e.bonus_max
            {
                bad.push(format!("{} double play out of range", d.draw_date));
            }
        } else if d.has_any_dp() {
            bad.push(format!("{} partial double play row", d.draw_date));
        }
    }
    let mut v = vec![if bad.is_empty() {
        r(
            gid,
            "draws.ranges",
            Status::Pass,
            format!(
                "all {} rows within era matrix (5 distinct whites, bonus in range)",
                draws.len()
            ),
        )
    } else {
        r(
            gid,
            "draws.ranges",
            Status::Fail,
            format!(
                "{} bad: {}",
                bad.len(),
                bad.iter().take(10).cloned().collect::<Vec<_>>().join("; ")
            ),
        )
    }];
    if inferred_rows > 0 {
        v.push(r(
            gid,
            "draws.ranges.inferred_eras",
            Status::Skip,
            format!(
                "{inferred_rows} rows checked against eras whose matrix is inferred from data, not sourced (circular for those rows; not a FAIL)"
            ),
        ));
    }
    v
}

fn check_era_ids(gid: &str, g: &GameDef, draws: &[Draw]) -> Vec<CheckResult> {
    let bad: Vec<String> = draws
        .iter()
        .filter(|d| g.era_for(d.draw_date).map(|e| e.id.as_str()) != Some(d.era_id.as_str()))
        .map(|d| format!("{} has {}", d.draw_date, d.era_id))
        .collect();
    vec![if bad.is_empty() {
        r(
            gid,
            "draws.era_ids",
            Status::Pass,
            "every era_id matches eras.toml",
        )
    } else {
        r(
            gid,
            "draws.era_ids",
            Status::Fail,
            format!(
                "{} mismatched (run update): {}",
                bad.len(),
                bad.iter().take(5).cloned().collect::<Vec<_>>().join("; ")
            ),
        )
    }]
}

fn check_schedule(gid: &str, g: &GameDef, draws: &[Draw]) -> Vec<CheckResult> {
    let first = draws.first().unwrap().draw_date;
    let last = draws.last().unwrap().draw_date;
    let have: HashSet<NaiveDate> = draws.iter().map(|d| d.draw_date).collect();
    let expected = g.scheduled_dates(first, last);
    let exp_set: HashSet<NaiveDate> = expected.iter().copied().collect();
    let gaps: Vec<String> = expected
        .iter()
        .filter(|d| !have.contains(d))
        .map(|d| d.to_string())
        .collect();
    let extra: Vec<String> = draws
        .iter()
        .filter(|d| !exp_set.contains(&d.draw_date))
        .map(|d| d.draw_date.to_string())
        .collect();
    vec![
        if gaps.is_empty() {
            r(
                gid,
                "draws.schedule_gaps",
                Status::Pass,
                format!(
                    "{} expected dates {first}..{last}, none missing",
                    expected.len()
                ),
            )
        } else {
            // Historical gaps: WARN + MISSING_DRAW (R-04). Due gaps still FAIL via staleness.
            r(
                gid,
                "draws.schedule_gaps",
                Status::Warn,
                format!(
                    "MISSING_DRAW: {} expected dates absent (document with [[game.schedule_exception]] or accept as WARN): {}",
                    gaps.len(),
                    gaps.iter().take(10).cloned().collect::<Vec<_>>().join(" ")
                ),
            )
        },
        if extra.is_empty() {
            r(
                gid,
                "draws.off_schedule",
                Status::Pass,
                "no draws on non-draw days",
            )
        } else {
            r(
                gid,
                "draws.off_schedule",
                Status::Warn,
                format!(
                    "{} draws on non-schedule days (add a schedule exception if real): {}",
                    extra.len(),
                    extra.join(" ")
                ),
            )
        },
    ]
}

fn check_missing(root: &Path, game: Game, g: &GameDef, draws: &[Draw]) -> Vec<CheckResult> {
    let gid = game.id();
    let mp = game.missing_path(root);
    match data::header_of(&mp) {
        Ok(h) if h == MISSING_HEADER => {}
        Ok(h) => return vec![r(gid, "missing.header", Status::Fail, format!("got {h:?}"))],
        Err(e) => return vec![r(gid, "missing.header", Status::Fail, format!("{e:#}"))],
    }
    let have: Vec<MissingRow> = match data::read_csv(&mp) {
        Ok(v) => v,
        Err(e) => return vec![r(gid, "missing.parse", Status::Fail, format!("{e:#}"))],
    };
    let expect = ny::missing_rows(game, g, draws);
    let mut v = Vec::new();
    if have == expect {
        let total: usize = have.iter().map(|m| m.rows).sum();
        v.push(r(
            gid,
            "missing.coverage",
            Status::Pass,
            format!(
                "{} documented null ranges covering {} field-rows; every blank is documented",
                have.len(),
                total
            ),
        ));
    } else {
        v.push(r(gid, "missing.coverage", Status::Fail, format!("missing.csv ({} ranges) does not match blanks in draws.csv ({} ranges); run `lottotool update {gid}`", have.len(), expect.len())));
    }
    let undoc: Vec<&MissingRow> = expect
        .iter()
        .filter(|m| m.status == "UNDOCUMENTED")
        .collect();
    if undoc.is_empty() {
        v.push(r(
            gid,
            "missing.rules",
            Status::Pass,
            "every null run matches a null_rule in eras.toml",
        ));
    } else {
        v.push(r(
            gid,
            "missing.rules",
            Status::Fail,
            format!(
                "{} null runs have no null_rule: {}",
                undoc.len(),
                undoc
                    .iter()
                    .map(|m| format!(
                        "{} {}..{} ({} rows)",
                        m.field, m.first_draw, m.last_draw, m.rows
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        ));
    }
    v
}

fn check_whites_ascending(gid: &str, draws: &[Draw]) -> Vec<CheckResult> {
    let bad: Vec<String> = draws
        .iter()
        .filter(|d| {
            let w = d.whites();
            w.windows(2).any(|p| p[0] > p[1])
        })
        .map(|d| format!("{} {:?}", d.draw_date, d.whites()))
        .collect();
    vec![if bad.is_empty() {
        r(
            gid,
            "draws.whites_ascending",
            Status::Pass,
            "all white-ball tuples strictly ascending",
        )
    } else {
        r(
            gid,
            "draws.whites_ascending",
            Status::Fail,
            format!(
                "{} non-ascending: {}",
                bad.len(),
                bad.iter().take(10).cloned().collect::<Vec<_>>().join("; ")
            ),
        )
    }]
}

/// Feature-live cutoffs for ingest_log coverage (D-029 / R4-01 / C-08).
/// Draws with `draw_date` **strictly after** these dates must have a log row.
/// PB last pre-feature draw: 2026-10-24; MM: 2026-10-23.
fn ingest_log_cutoff(game_id: &str) -> Option<NaiveDate> {
    match game_id {
        "powerball" => NaiveDate::from_ymd_opt(2026, 10, 24),
        "megamillions" => NaiveDate::from_ymd_opt(2026, 10, 23),
        _ => None,
    }
}

fn game_from_id(id: &str) -> Option<Game> {
    match id {
        "powerball" => Some(Game::Powerball),
        "megamillions" => Some(Game::Megamillions),
        _ => None,
    }
}

/// Archive-level ingest_log integrity (D-029 / R4-01 / R5-02).
///
/// - File may be absent when no post-cutoff draws exist yet (PASS).
/// - When present: header, parse, field formats (R5-02), timing WARNs (D-028),
///   unique `(game, draw_date)`, every row refs an existing draw, and every
///   post-cutoff draw among `games` has a log row.
fn check_ingest_log(root: &Path, games: &[Game]) -> Vec<CheckResult> {
    let label = "*";
    let p = ingest_log::path(root);

    // Collect post-cutoff draws for the games under check.
    let mut required: Vec<(String, NaiveDate)> = Vec::new();
    for &game in games {
        let cutoff = match ingest_log_cutoff(game.id()) {
            Some(c) => c,
            None => continue,
        };
        let dp = game.draws_path(root);
        let draws: Vec<Draw> = match data::read_csv(&dp) {
            Ok(d) => d,
            Err(_) => continue, // draws.parse already FAILed for this game
        };
        for d in draws {
            if d.draw_date > cutoff {
                required.push((game.id().to_string(), d.draw_date));
            }
        }
    }

    if !p.exists() {
        if required.is_empty() {
            return vec![r(
                label,
                "ingest_log.present",
                Status::Pass,
                "absent; no post-cutoff draws require log rows yet",
            )];
        }
        return vec![r(
            label,
            "ingest_log.present",
            Status::Fail,
            format!(
                "missing ingest_log.csv but {} post-cutoff draw(s) need log rows (D-029)",
                required.len()
            ),
        )];
    }

    let mut out = Vec::new();
    match data::header_of(&p) {
        Ok(h) if h == INGEST_LOG_HEADER => out.push(r(
            label,
            "ingest_log.header",
            Status::Pass,
            "matches schema",
        )),
        Ok(h) => {
            out.push(r(
                label,
                "ingest_log.header",
                Status::Fail,
                format!("got {h:?}"),
            ));
            for c in [
                "ingest_log.parse",
                "ingest_log.fields",
                "ingest_log.timing",
                "ingest_log.unique",
                "ingest_log.refs",
                "ingest_log.coverage",
            ] {
                out.push(r(label, c, Status::Skip, "header mismatch"));
            }
            return out;
        }
        Err(e) => {
            out.push(r(
                label,
                "ingest_log.header",
                Status::Fail,
                format!("{e:#}"),
            ));
            for c in [
                "ingest_log.parse",
                "ingest_log.fields",
                "ingest_log.timing",
                "ingest_log.unique",
                "ingest_log.refs",
                "ingest_log.coverage",
            ] {
                out.push(r(label, c, Status::Skip, "header unreadable"));
            }
            return out;
        }
    }

    let rows: Vec<IngestLogRow> = match data::read_csv(&p) {
        Ok(v) => {
            out.push(r(
                label,
                "ingest_log.parse",
                Status::Pass,
                format!("{} rows", v.len()),
            ));
            v
        }
        Err(e) => {
            out.push(r(label, "ingest_log.parse", Status::Fail, format!("{e:#}")));
            for c in [
                "ingest_log.fields",
                "ingest_log.timing",
                "ingest_log.unique",
                "ingest_log.refs",
                "ingest_log.coverage",
            ] {
                out.push(r(label, c, Status::Skip, "parse failed"));
            }
            return out;
        }
    };

    // R5-02: raw field-format checks (strict timestamps, draw_date round-trip,
    // schedule_slot = manual | well-formed 5-field cron; reject empty/local).
    out.extend(check_ingest_log_fields(&p, &rows));

    let mut seen = HashSet::new();
    let dups: Vec<String> = rows
        .iter()
        .filter(|r| !seen.insert((r.game.clone(), r.draw_date)))
        .map(|r| format!("{}:{}", r.game, r.draw_date))
        .collect();
    if dups.is_empty() {
        out.push(r(
            label,
            "ingest_log.unique",
            Status::Pass,
            "no duplicate (game, draw_date)",
        ));
    } else {
        out.push(r(
            label,
            "ingest_log.unique",
            Status::Fail,
            format!(
                "duplicates: {}",
                dups.iter().take(10).cloned().collect::<Vec<_>>().join(" ")
            ),
        ));
    }

    let mut orphan = Vec::new();
    for row in &rows {
        let Some(game) = game_from_id(&row.game) else {
            orphan.push(format!("{}:{} unknown game", row.game, row.draw_date));
            continue;
        };
        let draws: Vec<Draw> = match data::read_csv(&game.draws_path(root)) {
            Ok(d) => d,
            Err(e) => {
                orphan.push(format!(
                    "{}:{} draws unreadable ({e:#})",
                    row.game, row.draw_date
                ));
                continue;
            }
        };
        if !draws.iter().any(|d| d.draw_date == row.draw_date) {
            orphan.push(format!("{}:{}", row.game, row.draw_date));
        }
    }
    if orphan.is_empty() {
        out.push(r(
            label,
            "ingest_log.refs",
            Status::Pass,
            "every log row references an existing draw",
        ));
    } else {
        out.push(r(
            label,
            "ingest_log.refs",
            Status::Fail,
            format!(
                "{} log row(s) with no matching draw: {}",
                orphan.len(),
                orphan
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ));
    }

    let have: HashSet<(String, NaiveDate)> =
        rows.iter().map(|r| (r.game.clone(), r.draw_date)).collect();
    let missing: Vec<String> = required
        .iter()
        .filter(|k| !have.contains(&(k.0.clone(), k.1)))
        .map(|k| format!("{}:{}", k.0, k.1))
        .collect();
    if missing.is_empty() {
        out.push(r(
            label,
            "ingest_log.coverage",
            Status::Pass,
            if required.is_empty() {
                "no post-cutoff draws yet (PB > 2026-10-24, MM > 2026-10-23)".into()
            } else {
                format!("all {} post-cutoff draw(s) have log rows", required.len())
            },
        ));
    } else {
        out.push(r(
            label,
            "ingest_log.coverage",
            Status::Fail,
            format!(
                "{} post-cutoff draw(s) lack ingest_log rows (half-written update?): {}",
                missing.len(),
                missing
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ));
    }
    out
}

/// Strict `YYYY-MM-DDTHH:MM:SSZ` only — no offsets, no fractional seconds (R5-02).
fn is_strict_utc_second(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 20 {
        return false;
    }
    if b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return false;
    }
    for &i in &[0usize, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !b[i].is_ascii_digit() {
            return false;
        }
    }
    chrono::NaiveDateTime::parse_from_str(&s[..19], "%Y-%m-%dT%H:%M:%S").is_ok()
}

/// `manual`, or any well-formed 5-field cron; reject empty and `local` (R5-02 / D-027).
fn is_allowed_schedule_slot(s: &str) -> bool {
    if s == "manual" {
        return true;
    }
    if s.is_empty() || s == "local" {
        return false;
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 5 {
        return false;
    }
    parts.iter().all(|p| {
        !p.is_empty()
            && p.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '*' | '/' | '-' | ','))
    })
}

/// Draw date raw string must round-trip through `%Y-%m-%d` (R5-02).
fn draw_date_raw_ok(raw: &str) -> bool {
    match NaiveDate::parse_from_str(raw, "%Y-%m-%d") {
        Ok(d) => d.format("%Y-%m-%d").to_string() == raw,
        Err(_) => false,
    }
}

/// Allowed clock skew for `ny_rows_updated_at` ≤ `run_started_at` (D-028 WARN).
const INGEST_NY_SKEW_SECS: i64 = 300;

fn check_ingest_log_fields(path: &Path, typed: &[IngestLogRow]) -> Vec<CheckResult> {
    let label = "*";
    let mut field_problems: Vec<String> = Vec::new();
    let mut timing_warns: Vec<String> = Vec::new();

    let mut rdr = match csv::ReaderBuilder::new().has_headers(true).from_path(path) {
        Ok(r) => r,
        Err(e) => {
            return vec![
                r(
                    label,
                    "ingest_log.fields",
                    Status::Fail,
                    format!("raw re-read failed: {e}"),
                ),
                r(
                    label,
                    "ingest_log.timing",
                    Status::Skip,
                    "fields re-read failed",
                ),
            ];
        }
    };

    let mut raw_rows: Vec<csv::StringRecord> = Vec::new();
    for (i, rec) in rdr.records().enumerate() {
        match rec {
            Ok(rec) => raw_rows.push(rec),
            Err(e) => {
                return vec![
                    r(
                        label,
                        "ingest_log.fields",
                        Status::Fail,
                        format!("raw row {}: {e}", i + 1),
                    ),
                    r(
                        label,
                        "ingest_log.timing",
                        Status::Skip,
                        "fields re-read failed",
                    ),
                ];
            }
        }
    }

    if raw_rows.len() != typed.len() {
        return vec![
            r(
                label,
                "ingest_log.fields",
                Status::Fail,
                format!("raw row count {} != typed {}", raw_rows.len(), typed.len()),
            ),
            r(
                label,
                "ingest_log.timing",
                Status::Skip,
                "fields count mismatch",
            ),
        ];
    }

    for (i, (raw, row)) in raw_rows.iter().zip(typed.iter()).enumerate() {
        let n = i + 1;
        if raw.len() < 5 {
            field_problems.push(format!("row {n}: expected ≥5 fields, got {}", raw.len()));
            continue;
        }
        let game = raw.get(0).unwrap_or("");
        let draw_raw = raw.get(1).unwrap_or("");
        let ny_raw = raw.get(2).unwrap_or("");
        let run_raw = raw.get(3).unwrap_or("");
        let slot_raw = raw.get(4).unwrap_or("");

        if game != "powerball" && game != "megamillions" {
            field_problems.push(format!("row {n}: bad game {game:?}"));
        }
        if !draw_date_raw_ok(draw_raw) {
            field_problems.push(format!(
                "row {n}: draw_date {draw_raw:?} fails YYYY-MM-DD round-trip"
            ));
        }
        if !ny_raw.is_empty() && !is_strict_utc_second(ny_raw) {
            field_problems.push(format!(
                "row {n}: ny_rows_updated_at {ny_raw:?} not strict YYYY-MM-DDTHH:MM:SSZ"
            ));
        }
        if !is_strict_utc_second(run_raw) {
            field_problems.push(format!(
                "row {n}: run_started_at {run_raw:?} not strict YYYY-MM-DDTHH:MM:SSZ"
            ));
        }
        if !is_allowed_schedule_slot(slot_raw) {
            field_problems.push(format!(
                "row {n}: schedule_slot {slot_raw:?} (want manual or 5-field cron; not empty/local)"
            ));
        }

        // Timing WARNs (D-028): never FAIL ingest on views-timing quirks.
        if is_strict_utc_second(run_raw) {
            if let Ok(run_ndt) =
                chrono::NaiveDateTime::parse_from_str(&run_raw[..19], "%Y-%m-%dT%H:%M:%S")
            {
                let run_date = run_ndt.date();
                if row.draw_date > run_date {
                    timing_warns.push(format!(
                        "row {n}: draw_date {} > run_started_at date {run_date}",
                        row.draw_date
                    ));
                }
                if !ny_raw.is_empty() && is_strict_utc_second(ny_raw) {
                    if let Ok(ny_ndt) =
                        chrono::NaiveDateTime::parse_from_str(&ny_raw[..19], "%Y-%m-%dT%H:%M:%S")
                    {
                        let skew = chrono::Duration::seconds(INGEST_NY_SKEW_SECS);
                        if ny_ndt > run_ndt + skew {
                            timing_warns.push(format!(
                                "row {n}: ny_rows_updated_at {ny_raw} > run_started_at {run_raw} (+{INGEST_NY_SKEW_SECS}s skew)"
                            ));
                        }
                    }
                }
            }
        }
    }

    let mut out = Vec::new();
    if field_problems.is_empty() {
        out.push(r(
            label,
            "ingest_log.fields",
            Status::Pass,
            format!("{} row(s) field formats OK (R5-02)", typed.len()),
        ));
    } else {
        out.push(r(
            label,
            "ingest_log.fields",
            Status::Fail,
            format!(
                "{} field problem(s): {}",
                field_problems.len(),
                field_problems
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        ));
    }
    if timing_warns.is_empty() {
        out.push(r(
            label,
            "ingest_log.timing",
            Status::Pass,
            "draw_date/ny_rows_updated_at ordering OK (R5-02)",
        ));
    } else {
        out.push(r(
            label,
            "ingest_log.timing",
            Status::Warn,
            format!(
                "{} timing quirk(s) (WARN per D-028): {}",
                timing_warns.len(),
                timing_warns
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        ));
    }
    out
}

pub fn check_stale(gid: &str, g: &GameDef, draws: &[Draw], now: DateTime<Utc>) -> CheckResult {
    let latest = draws.last().map(|d| d.draw_date);
    match (latest, latest_due_draw(g, now)) {
        (_, None) => r(
            gid,
            "staleness",
            Status::Skip,
            "no expected draw found in the last 30 days of the schedule",
        ),
        (Some(l), Some(due)) if l >= due => r(
            gid,
            "staleness",
            Status::Pass,
            format!(
                "latest row {l}; latest draw due by now {due} (due = 06:00 ET next calendar day)"
            ),
        ),
        (l, Some(due)) => r(
            gid,
            "staleness",
            Status::Fail,
            format!(
                "STALE: latest row {}, but draw {due} was due by 06:00 ET on the next calendar day; source lag or failed update",
                l.map(|x| x.to_string()).unwrap_or_else(|| "none".into()),
            ),
        ),
    }
}

pub fn any_fail(results: &[CheckResult]) -> bool {
    results.iter().any(|c| c.status == Status::Fail)
}
