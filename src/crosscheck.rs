//! Local-only official-source winning-number cross-check (R-06 / OI-15).
//!
//! Compares committed `draws.csv` rows against:
//! - Powerball: powerball.com draw-result HTML (white-balls / powerball balls)
//! - Mega Millions: megamillions.com GetDrawDataByTickWithMatrix (`Drawing.N1..N5,MBall`)
//!
//! Never run on the scheduled update path; only via `lottotool check --crosscheck N`.
use crate::Game;
use crate::check::{CheckResult, Status};
use crate::data::Draw;
use crate::ny::user_agent;
use crate::payouts::{self, MM_API, dotnet_ticks, powerball_url};
use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use regex::Regex;
use serde_json::Value;

fn r(game: &str, check: &str, status: Status, detail: impl Into<String>) -> CheckResult {
    CheckResult {
        game: game.into(),
        check: check.into(),
        status,
        detail: detail.into(),
    }
}

/// Parse the five white balls + Powerball from a powerball.com draw-result page.
pub fn parse_powerball_winning_numbers(html: &str) -> Result<[u32; 6]> {
    let white_re = Regex::new(
        r#"(?s)class="[^"]*white-balls item-powerball[^"]*"[^>]*>\s*<div>\s*(\d+)\s*</div>"#,
    )
    .unwrap();
    let whites: Vec<u32> = white_re
        .captures_iter(html)
        .filter_map(|c| c.get(1)?.as_str().parse().ok())
        .collect();
    if whites.len() < 5 {
        bail!(
            "expected >=5 white-balls item-powerball numbers, got {}",
            whites.len()
        );
    }
    let bonus_re = Regex::new(
        r#"(?s)class="[^"]*\bpowerball item-powerball\b[^"]*"[^>]*>\s*<div>\s*(\d+)\s*</div>"#,
    )
    .unwrap();
    let bonus: u32 = bonus_re
        .captures(html)
        .and_then(|c| c.get(1)?.as_str().parse().ok())
        .ok_or_else(|| anyhow::anyhow!("powerball item-powerball number not found"))?;
    Ok([whites[0], whites[1], whites[2], whites[3], whites[4], bonus])
}

pub fn parse_megamillions_winning_numbers(v: &Value, date: NaiveDate) -> Result<[u32; 6]> {
    let inner: Value = match v.get("d") {
        Some(Value::String(s)) => serde_json::from_str(s).context("decoding d")?,
        Some(o) => o.clone(),
        None => v.clone(),
    };
    let d = &inner["Drawing"];
    let play = d["PlayDate"].as_str().unwrap_or("");
    if !play.starts_with(&date.to_string()) {
        bail!("API returned drawing {play:?}, not {date}");
    }
    let n = |k: &str| -> Result<u32> {
        d[k].as_u64()
            .map(|x| x as u32)
            .ok_or_else(|| anyhow::anyhow!("Drawing.{k} missing"))
    };
    Ok([
        n("N1")?,
        n("N2")?,
        n("N3")?,
        n("N4")?,
        n("N5")?,
        n("MBall")?,
    ])
}

fn fetch_powerball_numbers(date: NaiveDate) -> Result<[u32; 6]> {
    let url = powerball_url(date, false, None).expect("national URL");
    let html = payouts::http_get_retry_public(&url, |b| b.contains("white-balls"))?;
    parse_powerball_winning_numbers(&html)
}

fn fetch_megamillions_numbers(date: NaiveDate) -> Result<[u32; 6]> {
    let body = serde_json::json!({ "PlayDateTicks": dotnet_ticks(date).to_string() });
    let resp = ureq::post(MM_API)
        .set("User-Agent", &user_agent())
        .set("Content-Type", "application/json; charset=utf-8")
        .timeout(std::time::Duration::from_secs(60))
        .send_json(body)
        .context("POST megamillions API")?;
    let v: Value = resp.into_json()?;
    parse_megamillions_winning_numbers(&v, date)
}

fn row_nums(d: &Draw) -> [u32; 6] {
    [d.w1, d.w2, d.w3, d.w4, d.w5, d.bonus]
}

/// Cross-check the `n` most recent draws. Network errors → WARN (source lag), mismatches → FAIL.
pub fn run(game: Game, draws: &[Draw], n: usize) -> Vec<CheckResult> {
    let gid = game.id();
    if n == 0 {
        return vec![r(
            gid,
            "crosscheck",
            Status::Skip,
            "--crosscheck 0; nothing to compare",
        )];
    }
    let take = n.min(draws.len());
    let slice = &draws[draws.len() - take..];
    let mut mismatches = Vec::new();
    let mut errors = Vec::new();
    let mut ok = 0usize;
    for d in slice {
        let got = match game {
            Game::Powerball => fetch_powerball_numbers(d.draw_date),
            Game::Megamillions => fetch_megamillions_numbers(d.draw_date),
        };
        match got {
            Ok(nums) => {
                let ours = row_nums(d);
                // Official pages may list whites unsorted; compare as sets + bonus.
                let mut a = ours;
                let mut b = nums;
                a[..5].sort_unstable();
                b[..5].sort_unstable();
                if a == b {
                    ok += 1;
                } else {
                    mismatches.push(format!(
                        "{} ours={:?} official={:?}",
                        d.draw_date, ours, nums
                    ));
                }
            }
            Err(e) => errors.push(format!("{}: {e:#}", d.draw_date)),
        }
    }
    let mut out = Vec::new();
    if mismatches.is_empty() && errors.is_empty() {
        out.push(r(
            gid,
            "crosscheck",
            Status::Pass,
            format!("{ok}/{take} latest draws match official winning numbers"),
        ));
    } else if !mismatches.is_empty() {
        out.push(r(
            gid,
            "crosscheck",
            Status::Fail,
            format!(
                "{}/{} mismatched: {}",
                mismatches.len(),
                take,
                mismatches
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        ));
    } else {
        out.push(r(
            gid,
            "crosscheck",
            Status::Warn,
            format!(
                "{ok}/{take} matched; {} source errors (not treated as data FAIL): {}",
                errors.len(),
                errors
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pb_html_fixture() {
        let html = r#"
  <div class="form-control col white-balls item-powerball"><div>
1    </div></div>
  <div class="form-control col white-balls item-powerball"><div>
21    </div></div>
  <div class="form-control col white-balls item-powerball"><div>
44    </div></div>
  <div class="form-control col white-balls item-powerball"><div>
50    </div></div>
  <div class="form-control col white-balls item-powerball"><div>
57    </div></div>
  <div class="form-control col powerball item-powerball"><div>
5    </div></div>
"#;
        assert_eq!(
            parse_powerball_winning_numbers(html).unwrap(),
            [1, 21, 44, 50, 57, 5]
        );
    }

    #[test]
    fn parse_mm_api_fixture() {
        let v: Value = serde_json::json!({
            "d": "{\"Drawing\":{\"PlayDate\":\"2026-10-20T00:00:00\",\"N1\":13,\"N2\":28,\"N3\":33,\"N4\":35,\"N5\":48,\"MBall\":3}}"
        });
        assert_eq!(
            parse_megamillions_winning_numbers(&v, NaiveDate::from_ymd_opt(2026, 10, 20).unwrap())
                .unwrap(),
            [13, 28, 33, 35, 48, 3]
        );
    }
}
