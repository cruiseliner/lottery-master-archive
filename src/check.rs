//! `lottotool check`: data integrity + staleness checks. Every check reports
//! PASS / FAIL / WARN / SKIP, and SKIP always carries a reason.
use crate::Game;
use crate::data::{self, DRAWS_HEADER, Draw, MISSING_HEADER, MissingRow};
use crate::eras::{ErasFile, GameDef};
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
