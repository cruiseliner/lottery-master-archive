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
    /// Scheduled weekdays that produced no draw (cancelled / moved). Excluded from expected dates.
    #[serde(default)]
    pub schedule_exception: Vec<ScheduleException>,
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
pub struct ScheduleException {
    pub date: NaiveDate,
    pub reason: String,
    #[serde(default)]
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
    /// Independent pool key for white balls (R2-04). Prefer `{game}-white-{max}`.
    #[serde(default = "default_white_group")]
    pub white_group: String,
    /// Independent pool key for the bonus ball (R2-04). Prefer `{game}-bonus-{max}`.
    #[serde(default = "default_bonus_group")]
    pub bonus_group: String,
    pub source: String,
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_white_group() -> String {
    "white".into()
}
fn default_bonus_group() -> String {
    "bonus".into()
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
            // R2-04: a pool group must never mix differing maxima.
            for e in &g.era {
                eras_in_pool(g, &e.white_group, true).map_err(|m| anyhow!("game {}: {m}", g.id))?;
                eras_in_pool(g, &e.bonus_group, false)
                    .map_err(|m| anyhow!("game {}: {m}", g.id))?;
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
    pub fn is_schedule_exception(&self, date: NaiveDate) -> bool {
        self.schedule_exception.iter().any(|e| e.date == date)
    }
    /// Draw weekday that is not listed under `[[game.schedule_exception]]`.
    pub fn expected_draw_day(&self, date: NaiveDate) -> bool {
        self.is_draw_day(date) && !self.is_schedule_exception(date)
    }
    /// All expected draw dates in [from, to] (schedule exceptions excluded).
    pub fn scheduled_dates(&self, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
        from.iter_days()
            .take_while(|d| *d <= to)
            .filter(|d| self.expected_draw_day(*d))
            .collect()
    }
    pub fn null_rule_for(&self, field: &str, date: NaiveDate) -> Option<&NullRule> {
        self.null_rule
            .iter()
            .find(|r| r.field == field && r.from <= date && date <= r.to)
    }
}

/// Derive a pool group id from game, ball kind, and max (R2-04).
pub fn pool_group_id(game: &str, ball: &str, max: u32) -> String {
    let short = match game {
        "powerball" => "pb",
        "megamillions" => "mm",
        other => other,
    };
    format!("{short}-{ball}-{max}")
}

/// Eras that share `group` for white or bonus balls. Refuses if maxima differ (R2-04).
pub fn eras_in_pool<'a>(
    game: &'a GameDef,
    group: &str,
    white: bool,
) -> Result<Vec<&'a Era>, String> {
    let mut out = Vec::new();
    let mut max_seen: Option<u32> = None;
    for e in &game.era {
        let (g, mx) = if white {
            (&e.white_group, e.white_max)
        } else {
            (&e.bonus_group, e.bonus_max)
        };
        if g != group {
            continue;
        }
        if let Some(m) = max_seen {
            if m != mx {
                return Err(format!(
                    "refuse to pool group {group:?}: era {} has max {mx} but another era in the group has max {m}",
                    e.id
                ));
            }
        } else {
            max_seen = Some(mx);
        }
        out.push(e);
    }
    Ok(out)
}
