//! Per-tier winner counts / prize amounts. PRIVATE CACHE ONLY.
//!
//! Output goes to `cache/payouts/<game>/<date>_<scope>.csv` (gitignored). These tables are
//! NOT committed: powerball.com (MUSL) terms forbid reproduction without written consent and
//! megamillions.com terms are MISSING; permission from MUSL and the Florida Lottery is pending.
use crate::data::{PAYOUTS_HEADER, PayoutRow, write_csv};
use crate::ny::user_agent;
use anyhow::{Context, Result, anyhow, bail};
use chrono::NaiveDate;
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn cache_path(root: &Path, game: &str, date: NaiveDate, scope: &str) -> PathBuf {
    root.join("cache")
        .join("payouts")
        .join(game)
        .join(format!("{date}_{scope}.csv"))
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn parse_int(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit() || c == ',') {
        return None;
    }
    s.replace(',', "").parse().ok()
}

fn parse_usd(s: &str) -> Option<u64> {
    parse_int(s.trim().strip_prefix('$')?)
}

// ---------------------------------------------------------------- Powerball (powerball.com)

/// `product` = "powerball" or "double-play"; `state` = None (national) or e.g. Some("fl").
pub fn powerball_url(date: NaiveDate, double_play: bool, state: Option<&str>) -> String {
    let gc = if double_play {
        "pb-double-play"
    } else {
        "powerball"
    };
    let mut u = format!("https://www.powerball.com/draw-result?gc={gc}&date={date}");
    if let Some(s) = state {
        u.push_str(&format!("&oc={s}"));
    }
    u
}

fn http_get(url: &str) -> Result<String> {
    // powerball.com's CDN returns identity, gzip or br bodies seemingly at random (observed
    // 2026-10-23 even for the same URL), so ureq is built with "gzip" and "brotli" decoding.
    let resp = ureq::get(url)
        .set("User-Agent", &user_agent())
        .set("Accept-Encoding", "gzip, br")
        .timeout(std::time::Duration::from_secs(60))
        .call()
        .with_context(|| format!("GET {url}"))?;
    resp.into_string().context("reading body")
}

/// GET with up to 3 attempts; `ok` decides whether a body is usable.
fn http_get_retry(url: &str, ok: impl Fn(&str) -> bool) -> Result<String> {
    let mut last = anyhow::anyhow!("no attempt made");
    for attempt in 1..=3 {
        match http_get(url) {
            Ok(b) if ok(&b) => return Ok(b),
            Ok(b) => last = anyhow::anyhow!("unexpected page ({} bytes) from {url}", b.len()),
            Err(e) => last = e,
        }
        if attempt < 3 {
            std::thread::sleep(std::time::Duration::from_secs(2 * attempt));
        }
    }
    Err(last)
}

/// Parse the `winners-table` of a powerball.com draw-result page.
pub fn parse_powerball_html(
    html: &str,
    date: NaiveDate,
    scope: &str,
    double_play: bool,
    url: &str,
    fetched_at: &str,
) -> Result<Vec<PayoutRow>> {
    let start = html
        .find("winners-table")
        .ok_or_else(|| anyhow!("no winners-table in page"))?;
    let body_start = html[start..]
        .find("<tbody")
        .map(|i| start + i)
        .ok_or_else(|| anyhow!("no tbody"))?;
    let body_end = html[body_start..]
        .find("</tbody>")
        .map(|i| body_start + i)
        .ok_or_else(|| anyhow!("no /tbody"))?;
    let body = &html[body_start..body_end];
    let tier_re = Regex::new(r"game-balls[^>]*\bm(\d)(-d?pb)?\b")?;
    let cell_re = Regex::new(r#"(?s)data-label="([^"]+)"\s*>(.*?)</td>"#)?;
    let tag_re = Regex::new(r"<[^>]+>")?;
    let mut out = Vec::new();
    for tr in body.split("<tr").skip(1) {
        let Some(c) = tier_re.captures(tr) else {
            continue;
        };
        let white: u32 = c[1].parse()?;
        let bonus = c.get(2).is_some();
        let tier = format!("{white}+{}", if bonus { 1 } else { 0 });
        let mut cells = Vec::new();
        for cc in cell_re.captures_iter(tr) {
            let label = cc[1].to_string();
            let val = tag_re.replace_all(&cc[2], "").trim().to_string();
            cells.push((label, val));
        }
        // Pair "<X> Winners" with "<X> Prize".
        let products: Vec<String> = cells
            .iter()
            .filter_map(|(l, _)| l.strip_suffix(" Winners").map(str::to_string))
            .collect();
        for p in products {
            let winners = cells
                .iter()
                .find(|(l, _)| *l == format!("{p} Winners"))
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let prize = cells
                .iter()
                .find(|(l, _)| *l == format!("{p} Prize"))
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            if winners.is_empty() && prize.is_empty() {
                continue; // e.g. no Power Play on the jackpot tier
            }
            let product = match p.as_str() {
                "Powerball" => "base",
                "Power Play" => "power_play",
                "Double Play" => "double_play",
                _ => "unknown",
            };
            out.push(PayoutRow {
                game: "powerball".into(),
                draw_date: date,
                scope: scope.into(),
                product: product.into(),
                tier: tier.clone(),
                white_matched: white,
                bonus_matched: bonus,
                winners: parse_int(&winners),
                prize_usd: parse_usd(&prize),
                prize_text: prize,
                source_url: url.into(),
                fetched_at: fetched_at.into(),
            });
        }
    }
    if out.is_empty() {
        bail!("winners-table parsed but no tier rows found (page layout changed?)");
    }
    let _ = double_play;
    Ok(out)
}

pub fn fetch_powerball(date: NaiveDate, state: Option<&str>) -> Result<Vec<PayoutRow>> {
    let scope = state.unwrap_or("national");
    let mut rows = Vec::new();
    for dp in [false, true] {
        if dp && date < NaiveDate::from_ymd_opt(2021, 8, 23).unwrap() {
            continue;
        }
        let url = powerball_url(date, dp, state);
        let html = http_get_retry(&url, |b| b.contains("winners-table"))?;
        match parse_powerball_html(&html, date, scope, dp, &url, &now_iso()) {
            Ok(mut r) => rows.append(&mut r),
            Err(e) if dp => eprintln!("note: no Double Play table for {date} ({e})"),
            Err(e) => return Err(e),
        }
    }
    Ok(rows)
}

// ---------------------------------------------------------------- Mega Millions (megamillions.com)

pub const MM_API: &str =
    "https://www.megamillions.com/cmspages/utilservice.asmx/GetDrawDataByTickWithMatrix";

/// .NET ticks (100 ns since 0001-01-01) for midnight of `date`.
pub fn dotnet_ticks(date: NaiveDate) -> i64 {
    let epoch = NaiveDate::from_ymd_opt(1, 1, 1).unwrap();
    (date - epoch).num_days() * 86_400 * 10_000_000
}

pub fn parse_megamillions(
    v: &Value,
    date: NaiveDate,
    url: &str,
    fetched_at: &str,
) -> Result<Vec<PayoutRow>> {
    let inner: Value = match v.get("d") {
        Some(Value::String(s)) => serde_json::from_str(s).context("decoding d")?,
        Some(o) => o.clone(),
        None => v.clone(),
    };
    let play = inner["Drawing"]["PlayDate"].as_str().unwrap_or("");
    if !play.starts_with(&date.to_string()) {
        bail!("API returned drawing {play:?}, not {date} (no draw that day?)");
    }
    let matrix = inner["PrizeMatrix"]["PrizeTiers"]
        .as_array()
        .ok_or_else(|| anyhow!("no PrizeMatrix.PrizeTiers"))?;
    let megaplier = inner["Drawing"]["Megaplier"].as_i64().filter(|m| *m > 0);
    let mut out = Vec::new();
    for t in inner["PrizeTiers"]
        .as_array()
        .ok_or_else(|| anyhow!("no PrizeTiers"))?
    {
        let idx = t["Tier"].as_i64().ok_or_else(|| anyhow!("Tier"))?;
        let m = matrix
            .iter()
            .find(|m| m["PrizeTier"].as_i64() == Some(idx))
            .ok_or_else(|| anyhow!("tier {idx} not in matrix"))?;
        let white = m["TierWhiteBall"].as_u64().unwrap_or(0) as u32;
        let bonus = m["TierMegaBall"].as_bool().unwrap_or(false);
        let base = m["PrizeAmount"].as_f64().unwrap_or(0.0);
        let is_jp = m["IsJackpot"].as_bool().unwrap_or(false);
        let is_mp = t["IsMegaplier"].as_bool().unwrap_or(false);
        let mult_s = t["Multiplier"].as_str().unwrap_or("").trim().to_string();
        let mult: Option<i64> = if !mult_s.is_empty() {
            mult_s.trim_end_matches(['x', 'X']).parse().ok()
        } else if is_mp {
            megaplier
        } else {
            None
        };
        let product = if is_mp {
            "megaplier".to_string()
        } else if !mult_s.is_empty() {
            format!("builtin_{mult_s}")
        } else {
            "base".to_string()
        };
        let (prize_usd, prize_text) = if is_jp {
            (None, "Jackpot".to_string())
        } else {
            match mult {
                Some(k) => {
                    let col = m[format!("Mega{k}")].as_f64().filter(|x| *x > 0.0);
                    match col {
                        Some(x) => (Some(x as u64), format!("${x:.0} (Mega{k})")),
                        None => (
                            None,
                            format!(
                                "base ${base:.0}; multiplier {k}x; amount column Mega{k} absent"
                            ),
                        ),
                    }
                }
                None => (Some(base as u64), format!("${base:.0}")),
            }
        };
        out.push(PayoutRow {
            game: "megamillions".into(),
            draw_date: date,
            scope: "national".into(),
            product,
            tier: format!("{white}+{}", if bonus { 1 } else { 0 }),
            white_matched: white,
            bonus_matched: bonus,
            winners: t["Winners"].as_u64(),
            prize_usd,
            prize_text,
            source_url: url.into(),
            fetched_at: fetched_at.into(),
        });
    }
    Ok(out)
}

pub fn fetch_megamillions(date: NaiveDate) -> Result<Vec<PayoutRow>> {
    let body = serde_json::json!({ "PlayDateTicks": dotnet_ticks(date).to_string() });
    let resp = ureq::post(MM_API)
        .set("User-Agent", &user_agent())
        .set("Content-Type", "application/json; charset=utf-8")
        .timeout(std::time::Duration::from_secs(60))
        .send_json(body)
        .context("POST megamillions API")?;
    let v: Value = resp.into_json()?;
    parse_megamillions(&v, date, MM_API, &now_iso())
}

pub fn write_cache(
    root: &Path,
    game: &str,
    date: NaiveDate,
    scope: &str,
    rows: &[PayoutRow],
) -> Result<PathBuf> {
    let p = cache_path(root, game, date, scope);
    write_csv(&p, rows, PAYOUTS_HEADER)?;
    Ok(p)
}
