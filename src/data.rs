//! CSV schemas for committed data (see docs/SCHEMAS.md).
use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const DRAWS_HEADER: &str = "draw_date,era_id,w1,w2,w3,w4,w5,bonus,multiplier,dp_w1,dp_w2,dp_w3,dp_w4,dp_w5,dp_bonus,source";
pub const MISSING_HEADER: &str = "game,field,first_draw,last_draw,rows,status,reason,source";
pub const PAYOUTS_HEADER: &str = "game,draw_date,scope,product,tier,white_matched,bonus_matched,winners,prize_usd,prize_text,source_url,fetched_at";

/// One draw row of `data/<game>/draws.csv`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Draw {
    pub draw_date: NaiveDate,
    pub era_id: String,
    pub w1: u32,
    pub w2: u32,
    pub w3: u32,
    pub w4: u32,
    pub w5: u32,
    pub bonus: u32,
    /// Blank = null; every null run is documented in missing.csv.
    pub multiplier: Option<u32>,
    pub dp_w1: Option<u32>,
    pub dp_w2: Option<u32>,
    pub dp_w3: Option<u32>,
    pub dp_w4: Option<u32>,
    pub dp_w5: Option<u32>,
    pub dp_bonus: Option<u32>,
    pub source: String,
}

impl Draw {
    pub fn whites(&self) -> [u32; 5] {
        [self.w1, self.w2, self.w3, self.w4, self.w5]
    }
    pub fn dp(&self) -> Option<([u32; 5], u32)> {
        Some((
            [
                self.dp_w1?,
                self.dp_w2?,
                self.dp_w3?,
                self.dp_w4?,
                self.dp_w5?,
            ],
            self.dp_bonus?,
        ))
    }
    pub fn has_any_dp(&self) -> bool {
        [
            self.dp_w1,
            self.dp_w2,
            self.dp_w3,
            self.dp_w4,
            self.dp_w5,
            self.dp_bonus,
        ]
        .iter()
        .any(|x| x.is_some())
    }
}

/// One documented-null range of `data/<game>/missing.csv`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissingRow {
    pub game: String,
    pub field: String,
    pub first_draw: NaiveDate,
    pub last_draw: NaiveDate,
    pub rows: usize,
    pub status: String,
    pub reason: String,
    pub source: String,
}

/// One per-tier payout row (cache/ only; see schemas/payouts.csv).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayoutRow {
    pub game: String,
    pub draw_date: NaiveDate,
    /// "national" or a lowercase state code (e.g. "fl").
    pub scope: String,
    /// "base", "power_play", "double_play", "megaplier".
    pub product: String,
    pub tier: String,
    pub white_matched: u32,
    pub bonus_matched: bool,
    pub winners: Option<u64>,
    pub prize_usd: Option<u64>,
    pub prize_text: String,
    pub source_url: String,
    pub fetched_at: String,
}

pub fn read_csv<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    let mut rdr =
        csv::Reader::from_path(path).with_context(|| format!("opening {}", path.display()))?;
    let mut out = Vec::new();
    for (i, r) in rdr.deserialize().enumerate() {
        out.push(r.with_context(|| format!("{} data row {}", path.display(), i + 1))?);
    }
    Ok(out)
}

pub fn write_csv<T: Serialize>(path: &Path, rows: &[T], header: &str) -> Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut w = csv::WriterBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    w.write_record(header.split(','))?;
    for r in rows {
        w.serialize(r)?;
    }
    w.flush()?;
    Ok(())
}

pub fn header_of(path: &Path) -> Result<String> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(text
        .lines()
        .next()
        .unwrap_or("")
        .trim_end_matches('\r')
        .to_string())
}
