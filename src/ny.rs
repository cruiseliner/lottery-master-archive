//! NY Open Data (Socrata SODA) fetch + normalisation into `Draw` rows.
use crate::Game;
use crate::data::{Draw, MissingRow};
use crate::eras::GameDef;
use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde_json::Value;

pub fn user_agent() -> String {
    format!(
        "lottotool/{} (+https://github.com/cruiseliner/lottery-master-archive)",
        env!("CARGO_PKG_VERSION")
    )
}

/// Fetch every row (ascending by date). One request; both datasets are < 3,000 rows.
pub fn fetch_rows(g: &GameDef) -> Result<Vec<Value>> {
    let url = format!("{}?$limit=50000&$order=draw_date%20ASC", g.ny_dataset_url);
    let resp = ureq::get(&url)
        .set("Accept", "application/json")
        .set("User-Agent", &user_agent())
        .timeout(std::time::Duration::from_secs(90))
        .call()
        .with_context(|| format!("GET {url}"))?;
    let v: Value = resp.into_json().context("decoding NY JSON")?;
    match v {
        Value::Array(a) => Ok(a),
        other => bail!(
            "unexpected NY response (not an array): {}",
            truncate(&other.to_string(), 300)
        ),
    }
}

/// Fetch dataset-level `rowsUpdatedAt` from the Socrata views API.
/// Returns `None` on any failure and emits a WARN (never fails the update; C-08 a).
/// The timestamp is an **upper bound** on when the draw was posted (dataset-level).
pub fn fetch_view_rows_updated_at(dataset_id: &str) -> Option<DateTime<Utc>> {
    match fetch_view_rows_updated_at_inner(dataset_id) {
        Ok(dt) => Some(dt),
        Err(e) => {
            let msg = format!(
                "Socrata views rowsUpdatedAt unavailable for {dataset_id}: {e:#} (diagnostic only; continuing)"
            );
            if std::env::var_os("GITHUB_ACTIONS").is_some() {
                println!("::warning::{msg}");
            } else {
                eprintln!("warning: {msg}");
            }
            None
        }
    }
}

fn fetch_view_rows_updated_at_inner(dataset_id: &str) -> Result<DateTime<Utc>> {
    let url = format!("https://data.ny.gov/api/views/{dataset_id}.json");
    let resp = ureq::get(&url)
        .set("Accept", "application/json")
        .set("User-Agent", &user_agent())
        .timeout(std::time::Duration::from_secs(30))
        .call()
        .with_context(|| format!("GET {url}"))?;
    let v: Value = resp.into_json().context("decoding views JSON")?;
    let epoch = v
        .get("rowsUpdatedAt")
        .and_then(|x| x.as_i64().or_else(|| x.as_u64().map(|u| u as i64)))
        .ok_or_else(|| anyhow!("views JSON missing integer rowsUpdatedAt"))?;
    Utc.timestamp_opt(epoch, 0)
        .single()
        .ok_or_else(|| anyhow!("rowsUpdatedAt epoch {epoch} out of range"))
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn text<'a>(row: &'a Value, k: &str) -> Option<&'a str> {
    row.get(k)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|x| !x.is_empty())
}

fn num(row: &Value, k: &str) -> Result<Option<u32>> {
    match row.get(k) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(x)) if x.trim().is_empty() => Ok(None),
        Some(Value::String(x)) => Ok(Some(
            x.trim()
                .parse::<u32>()
                .with_context(|| format!("{k}={x:?}"))?,
        )),
        Some(Value::Number(n)) => Ok(Some(u32::try_from(
            n.as_u64().ok_or_else(|| anyhow!("{k} not an integer"))?,
        )?)),
        Some(o) => bail!("{k}: unexpected {o}"),
    }
}

fn parse_nums(t: &str) -> Result<Vec<u32>> {
    t.split_whitespace()
        .map(|x| {
            x.parse::<u32>()
                .with_context(|| format!("bad number {x:?}"))
        })
        .collect()
}

/// Normalise NY rows. Leading zeros are dropped (MM mega_ball / multiplier padding is
/// inconsistent in the source, e.g. "03" and "3").
pub fn normalise(game: Game, g: &GameDef, rows: &[Value]) -> Result<Vec<Draw>> {
    let mut out = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let ctx = || format!("NY row {i}: {}", truncate(&row.to_string(), 200));
        let date_s = text(row, "draw_date")
            .ok_or_else(|| anyhow!("no draw_date"))
            .with_context(ctx)?;
        let draw_date = NaiveDate::parse_from_str(date_s.get(..10).unwrap_or(date_s), "%Y-%m-%d")
            .with_context(ctx)?;
        let nums = parse_nums(text(row, "winning_numbers").unwrap_or("")).with_context(ctx)?;
        let (whites, bonus) = match game {
            Game::Powerball => {
                if nums.len() != 6 {
                    bail!("{}: expected 6 numbers, got {}", ctx(), nums.len());
                }
                (nums[..5].to_vec(), nums[5])
            }
            Game::Megamillions => {
                if nums.len() != 5 {
                    bail!("{}: expected 5 numbers, got {}", ctx(), nums.len());
                }
                let mb = num(row, "mega_ball")
                    .with_context(ctx)?
                    .ok_or_else(|| anyhow!("no mega_ball"))
                    .with_context(ctx)?;
                (nums, mb)
            }
        };
        let multiplier = num(row, "multiplier").with_context(ctx)?;
        let dp = match text(row, "double_play_winning_numbers") {
            Some(t) => {
                let d = parse_nums(t).with_context(ctx)?;
                if d.len() != 6 {
                    bail!("{}: double play expected 6 numbers, got {}", ctx(), d.len());
                }
                Some(d)
            }
            None => None,
        };
        let era_id = g
            .era_for(draw_date)
            .map(|e| e.id.clone())
            .unwrap_or_else(|| "UNKNOWN".into());
        out.push(Draw {
            draw_date,
            era_id,
            w1: whites[0],
            w2: whites[1],
            w3: whites[2],
            w4: whites[3],
            w5: whites[4],
            bonus,
            multiplier,
            dp_w1: dp.as_ref().map(|d| d[0]),
            dp_w2: dp.as_ref().map(|d| d[1]),
            dp_w3: dp.as_ref().map(|d| d[2]),
            dp_w4: dp.as_ref().map(|d| d[3]),
            dp_w5: dp.as_ref().map(|d| d[4]),
            dp_bonus: dp.as_ref().map(|d| d[5]),
            source: format!("ny:{}", g.ny_dataset),
        });
    }
    out.sort_by_key(|d| d.draw_date);
    Ok(out)
}

/// Build missing.csv: one row per maximal run of consecutive draws with a null field,
/// split wherever the matching null_rule (status/reason) changes.
pub fn missing_rows(game: Game, g: &GameDef, draws: &[Draw]) -> Vec<MissingRow> {
    let mut out: Vec<MissingRow> = Vec::new();
    for field in ["multiplier", "double_play"] {
        let mut cur: Option<MissingRow> = None;
        for d in draws {
            let is_null = match field {
                "multiplier" => d.multiplier.is_none(),
                _ => !d.has_any_dp(),
            };
            if !is_null {
                if let Some(r) = cur.take() {
                    out.push(r);
                }
                continue;
            }
            let (status, reason, source) = match g.null_rule_for(field, d.draw_date) {
                Some(r) => (r.status.clone(), r.reason.clone(), r.source.clone()),
                None => (
                    "UNDOCUMENTED".to_string(),
                    "blank in source; no null_rule in eras.toml covers it".to_string(),
                    format!("ny:{}", g.ny_dataset),
                ),
            };
            match cur.as_mut() {
                Some(r) if r.status == status && r.reason == reason => {
                    r.last_draw = d.draw_date;
                    r.rows += 1;
                }
                _ => {
                    if let Some(r) = cur.take() {
                        out.push(r);
                    }
                    cur = Some(MissingRow {
                        game: game.id().into(),
                        field: field.into(),
                        first_draw: d.draw_date,
                        last_draw: d.draw_date,
                        rows: 1,
                        status,
                        reason,
                        source,
                    });
                }
            }
        }
        if let Some(r) = cur.take() {
            out.push(r);
        }
    }
    // R-04: historical expected-draw gaps → documented as field=draw / MISSING_DRAW.
    if let (Some(first), Some(last)) = (draws.first(), draws.last()) {
        let have: std::collections::HashSet<_> = draws.iter().map(|d| d.draw_date).collect();
        let mut gap_run: Option<(chrono::NaiveDate, chrono::NaiveDate, usize)> = None;
        let flush =
            |out: &mut Vec<MissingRow>,
             run: &mut Option<(chrono::NaiveDate, chrono::NaiveDate, usize)>| {
                if let Some((a, b, n)) = run.take() {
                    out.push(MissingRow {
                    game: game.id().into(),
                    field: "draw".into(),
                    first_draw: a,
                    last_draw: b,
                    rows: n,
                    status: "MISSING_DRAW".into(),
                    reason: "expected draw date absent from source; add [[game.schedule_exception]] if cancelled/moved".into(),
                    source: format!("ny:{}", g.ny_dataset),
                });
                }
            };
        for d in g.scheduled_dates(first.draw_date, last.draw_date) {
            if have.contains(&d) {
                flush(&mut out, &mut gap_run);
                continue;
            }
            match gap_run.as_mut() {
                Some((_a, b, n)) => {
                    *b = d;
                    *n += 1;
                }
                None => gap_run = Some((d, d, 1)),
            }
        }
        flush(&mut out, &mut gap_run);
    }
    out
}
