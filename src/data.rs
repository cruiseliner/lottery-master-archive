//! CSV schemas for committed data (see docs/SCHEMAS.md).
use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const DRAWS_HEADER: &str = "draw_date,era_id,w1,w2,w3,w4,w5,bonus,multiplier,dp_w1,dp_w2,dp_w3,dp_w4,dp_w5,dp_bonus,source";
pub const MISSING_HEADER: &str = "game,field,first_draw,last_draw,rows,status,reason,source";
pub const PAYOUTS_HEADER: &str = "game,draw_date,scope,product,tier,white_matched,bonus_matched,winners,prize_usd,prize_text,source_url,fetched_at";
pub const INGEST_LOG_HEADER: &str =
    "game,draw_date,ny_rows_updated_at,run_started_at,schedule_slot";

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

/// One row of `data/ingest_log.csv` (D-028 / C-08).
/// `ny_rows_updated_at` is dataset-level Socrata views upper bound; empty if fetch failed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngestLogRow {
    pub game: String,
    pub draw_date: NaiveDate,
    pub ny_rows_updated_at: String,
    pub run_started_at: String,
    pub schedule_slot: String,
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

/// Same-directory temp path for atomic writes (R6-03).
/// Name starts with `.#lottotool-tmp.` and is gitignored so `git add data/` cannot sweep leftovers.
fn atomic_tmp_path(final_path: &Path) -> std::path::PathBuf {
    let file_name = final_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp_name = format!(
        ".#lottotool-tmp.{file_name}.{}.{}",
        std::process::id(),
        nanos
    );
    match final_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.join(tmp_name),
        _ => std::path::PathBuf::from(tmp_name),
    }
}

/// Write `bytes` via same-dir temp + flush + fsync + rename; remove temp on any error (R6-03).
pub fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(p) = path.parent() {
        if !p.as_os_str().is_empty() {
            std::fs::create_dir_all(p)
                .with_context(|| format!("creating parent for {}", path.display()))?;
        }
    }
    let tmp = atomic_tmp_path(path);
    let result = (|| {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)
            .with_context(|| format!("creating temp {}", tmp.display()))?;
        f.write_all(bytes)
            .with_context(|| format!("writing temp {}", tmp.display()))?;
        f.flush()
            .with_context(|| format!("flushing temp {}", tmp.display()))?;
        f.sync_all()
            .with_context(|| format!("fsync temp {}", tmp.display()))?;
        drop(f);
        std::fs::rename(&tmp, path)
            .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Atomic CSV write: same-dir temp, explicit `flush()`, fsync, rename; cleanup on error (R6-03).
pub fn write_csv<T: Serialize>(path: &Path, rows: &[T], header: &str) -> Result<()> {
    if let Some(p) = path.parent() {
        if !p.as_os_str().is_empty() {
            std::fs::create_dir_all(p)?;
        }
    }
    let tmp = atomic_tmp_path(path);
    let result = (|| {
        use std::io::Write;
        let file = std::fs::File::create(&tmp)
            .with_context(|| format!("creating temp {}", tmp.display()))?;
        let mut w = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);
        w.write_record(header.split(','))?;
        for r in rows {
            w.serialize(r)?;
        }
        // Explicit flush: csv::Writer flush-on-drop swallows errors.
        w.flush()?;
        let mut file = w
            .into_inner()
            .map_err(|e| anyhow::anyhow!("csv into_inner: {e}"))?;
        file.flush()
            .with_context(|| format!("flushing temp {}", tmp.display()))?;
        file.sync_all()
            .with_context(|| format!("fsync temp {}", tmp.display()))?;
        drop(file);
        std::fs::rename(&tmp, path)
            .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
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

#[cfg(test)]
mod atomic_write_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "lottotool-atomic-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn stray_temps(dir: &Path) -> Vec<std::path::PathBuf> {
        std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with(".#lottotool-tmp."))
                    .unwrap_or(false)
            })
            .collect()
    }

    fn sample_rows() -> Vec<IngestLogRow> {
        vec![IngestLogRow {
            game: "powerball".into(),
            draw_date: NaiveDate::from_ymd_opt(2026, 10, 26).unwrap(),
            ny_rows_updated_at: "2026-10-27T07:38:49Z".into(),
            run_started_at: "2026-10-27T15:52:45Z".into(),
            schedule_slot: "manual".into(),
        }]
    }

    #[test]
    fn write_csv_round_trip_byte_identical_on_noop_rewrite() {
        let dir = tmp_dir("noop");
        let path = dir.join("ingest_log.csv");
        let rows = sample_rows();
        write_csv(&path, &rows, INGEST_LOG_HEADER).unwrap();
        let first = std::fs::read(&path).unwrap();
        write_csv(&path, &rows, INGEST_LOG_HEADER).unwrap();
        let second = std::fs::read(&path).unwrap();
        assert_eq!(first, second, "no-op rewrite must stay byte-identical");
        assert!(
            stray_temps(&dir).is_empty(),
            "leftover temps: {:?}",
            stray_temps(&dir)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_bytes_atomic_round_trip_byte_identical() {
        let dir = tmp_dir("bytes-noop");
        let path = dir.join("meta.json");
        let body = b"{\"ok\":true}\n";
        write_bytes_atomic(&path, body).unwrap();
        let first = std::fs::read(&path).unwrap();
        write_bytes_atomic(&path, body).unwrap();
        assert_eq!(first, std::fs::read(&path).unwrap());
        assert_eq!(first, body);
        assert!(stray_temps(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_csv_failed_preserves_original_and_no_stray_temp() {
        let dir = tmp_dir("fail-csv");
        let path = dir.join("ingest_log.csv");
        let rows = sample_rows();
        write_csv(&path, &rows, INGEST_LOG_HEADER).unwrap();
        let original = std::fs::read(&path).unwrap();

        // Read-only directory: temp create fails; original must survive.
        let mut perms = std::fs::metadata(&dir).unwrap().permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(&dir, perms).unwrap();

        let err = write_csv(
            &path,
            &[IngestLogRow {
                game: "megamillions".into(),
                draw_date: NaiveDate::from_ymd_opt(2026, 10, 27).unwrap(),
                ny_rows_updated_at: "".into(),
                run_started_at: "2026-10-28T16:37:10Z".into(),
                schedule_slot: "manual".into(),
            }],
            INGEST_LOG_HEADER,
        );
        assert!(err.is_err(), "expected failure on read-only dir");

        // Restore writable for cleanup/assertions.
        let mut perms = std::fs::metadata(&dir).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dir, perms).unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert!(
            stray_temps(&dir).is_empty(),
            "stray: {:?}",
            stray_temps(&dir)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_bytes_atomic_failed_preserves_original_and_no_stray_temp() {
        let dir = tmp_dir("fail-bytes");
        let path = dir.join("meta.json");
        let original = b"{\"v\":1}\n";
        write_bytes_atomic(&path, original).unwrap();

        let mut perms = std::fs::metadata(&dir).unwrap().permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(&dir, perms).unwrap();

        assert!(write_bytes_atomic(&path, b"{\"v\":2}\n").is_err());

        let mut perms = std::fs::metadata(&dir).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dir, perms).unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert!(stray_temps(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_tmp_name_is_gitignored_pattern() {
        let p = Path::new("data/powerball/draws.csv");
        let tmp = atomic_tmp_path(p);
        let name = tmp.file_name().unwrap().to_str().unwrap();
        assert!(
            name.starts_with(".#lottotool-tmp.draws.csv."),
            "unexpected temp name {name}"
        );
        assert_eq!(tmp.parent(), p.parent());
    }
}
