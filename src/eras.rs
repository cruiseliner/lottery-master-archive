//! Loading and querying `eras.toml`.
use anyhow::{Context, Result, anyhow, bail};
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErasFile {
    pub schema_version: u32,
    pub game: Vec<GameDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameDef {
    pub id: String,
    pub name: String,
    pub ny_dataset: String,
    pub ny_dataset_url: String,
    pub draw_time_et: String,
    pub draw_time_source: String,
    pub stale_grace_hours: i64,
    pub schedule: Vec<Schedule>,
    pub era: Vec<Era>,
    #[serde(default)]
    pub null_rule: Vec<NullRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Schedule {
    pub from: NaiveDate,
    pub days: Vec<String>,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Era {
    pub id: String,
    pub first_draw: NaiveDate,
    pub first_draw_status: String,
    pub white_count: u32,
    pub white_max: u32,
    pub bonus_max: u32,
    pub matrix_status: String,
    #[serde(default)]
    pub price_usd: Option<u32>,
    pub price_status: String,
    #[serde(default)]
    pub multiplier: Option<String>,
    pub source: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NullRule {
    pub field: String,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub status: String,
    pub reason: String,
    pub source: String,
}

pub const STATUSES: [&str; 3] = ["sourced", "inferred", "missing"];

impl ErasFile {
    pub fn load(path: &Path) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse(&text)
    }
    pub fn parse(text: &str) -> Result<Self> {
        let f: ErasFile = toml::from_str(text).context("parsing eras.toml")?;
        f.validate()?;
        Ok(f)
    }
    pub fn validate(&self) -> Result<()> {
        for g in &self.game {
            if g.era.is_empty() {
                bail!("game {} has no eras", g.id);
            }
            for w in g.era.windows(2) {
                if w[1].first_draw <= w[0].first_draw {
                    bail!("game {}: eras not in ascending first_draw order", g.id);
                }
            }
            for e in &g.era {
                for (k, v) in [
                    ("first_draw_status", &e.first_draw_status),
                    ("matrix_status", &e.matrix_status),
                    ("price_status", &e.price_status),
                ] {
                    if !STATUSES.contains(&v.as_str()) {
                        bail!("era {}: {k} = {v:?} is not one of {STATUSES:?}", e.id);
                    }
                }
                if e.price_status != "missing" && e.price_usd.is_none() {
                    bail!(
                        "era {}: price_status {} but no price_usd",
                        e.id,
                        e.price_status
                    );
                }
            }
            for s in &g.schedule {
                for d in &s.days {
                    parse_weekday(d)?;
                }
            }
        }
        Ok(())
    }
    pub fn game(&self, id: &str) -> Result<&GameDef> {
        self.game
            .iter()
            .find(|g| g.id == id)
            .ok_or_else(|| anyhow!("game {id} not in eras.toml"))
    }
}

pub fn parse_weekday(s: &str) -> Result<Weekday> {
    s.parse::<Weekday>()
        .map_err(|_| anyhow!("bad weekday {s:?}"))
}

impl GameDef {
    /// Era in force for a draw date (last era whose first_draw <= date).
    pub fn era_for(&self, date: NaiveDate) -> Option<&Era> {
        self.era.iter().rev().find(|e| e.first_draw <= date)
    }
    pub fn current_era(&self) -> &Era {
        self.era.last().expect("validated non-empty")
    }
    /// Last calendar day of era `idx` (day before next era's first draw), None if current.
    pub fn era_last_day(&self, idx: usize) -> Option<NaiveDate> {
        self.era
            .get(idx + 1)
            .map(|n| n.first_draw.pred_opt().expect("valid date"))
    }
    pub fn schedule_for(&self, date: NaiveDate) -> Option<&Schedule> {
        self.schedule.iter().rev().find(|s| s.from <= date)
    }
    pub fn is_draw_day(&self, date: NaiveDate) -> bool {
        match self.schedule_for(date) {
            Some(s) => s.days.iter().any(|d| {
                parse_weekday(d)
                    .map(|w| w == date.weekday())
                    .unwrap_or(false)
            }),
            None => false,
        }
    }
    /// All scheduled draw dates in [from, to].
    pub fn scheduled_dates(&self, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
        from.iter_days()
            .take_while(|d| *d <= to)
            .filter(|d| self.is_draw_day(*d))
            .collect()
    }
    pub fn null_rule_for(&self, field: &str, date: NaiveDate) -> Option<&NullRule> {
        self.null_rule
            .iter()
            .find(|r| r.field == field && r.from <= date && date <= r.to)
    }
}
