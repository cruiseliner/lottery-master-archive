//! In-repo delay/ingest log (`data/ingest_log.csv`).
//!
//! Appended only when a new draw lands (D-028 / C-08). Idempotent on `(game, draw_date)`.
//! `ny_rows_updated_at` is a dataset-level Socrata views upper bound on posting time;
//! blank when the views call failed (WARN, never fail the update).
//!
//! Corrupt or unparseable existing files fail the update closed (D-029 / R4-01): never
//! treat a bad file as empty and rewrite, which would silently destroy history.

use crate::data::{self, INGEST_LOG_HEADER, IngestLogRow};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::collections::HashSet;
use std::path::Path;

pub fn path(root: &Path) -> std::path::PathBuf {
    root.join("data").join("ingest_log.csv")
}

/// Append one row per newly added draw date. Skips dates already present for `game`.
/// Creates the file with header when needed. Returns the number of rows newly written.
/// Does nothing (and does not create the file) when `new_dates` is empty after filtering.
///
/// If `ingest_log.csv` already exists but cannot be read/parsed, returns `Err` and does
/// not rewrite the file (D-029).
pub fn append_for_new_draws(
    root: &Path,
    game: &str,
    new_dates: &[NaiveDate],
    ny_rows_updated_at: Option<&str>,
    run_started_at: &str,
    schedule_slot: &str,
) -> Result<usize> {
    if new_dates.is_empty() {
        return Ok(0);
    }
    let p = path(root);
    let mut existing: Vec<IngestLogRow> = if p.exists() {
        data::read_csv(&p).with_context(|| {
            format!(
                "existing ingest_log.csv is corrupt or unparseable ({}); refusing to rewrite (D-029)",
                p.display()
            )
        })?
    } else {
        Vec::new()
    };
    let have: HashSet<(String, NaiveDate)> = existing
        .iter()
        .map(|r| (r.game.clone(), r.draw_date))
        .collect();
    let ny = ny_rows_updated_at.unwrap_or("").to_string();
    let mut added = 0usize;
    for &draw_date in new_dates {
        let key = (game.to_string(), draw_date);
        if have.contains(&key) {
            continue;
        }
        existing.push(IngestLogRow {
            game: game.to_string(),
            draw_date,
            ny_rows_updated_at: ny.clone(),
            run_started_at: run_started_at.to_string(),
            schedule_slot: schedule_slot.to_string(),
        });
        added += 1;
    }
    if added == 0 {
        return Ok(0);
    }
    // Stable order: by draw_date then game (matches typical append-chronological use).
    existing.sort_by(|a, b| {
        a.draw_date
            .cmp(&b.draw_date)
            .then_with(|| a.game.cmp(&b.game))
    });
    data::write_csv(&p, &existing, INGEST_LOG_HEADER)?;
    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn tmp() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "lottotool-ingest-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(p.join("data")).unwrap();
        p
    }

    #[test]
    fn idempotent_same_draw_twice() {
        let root = tmp();
        let dates = [d("2026-10-24")];
        let n1 = append_for_new_draws(
            &root,
            "powerball",
            &dates,
            Some("2026-10-25T07:40:00Z"),
            "2026-10-25T14:22:00Z",
            "17 6 * * *",
        )
        .unwrap();
        let n2 = append_for_new_draws(
            &root,
            "powerball",
            &dates,
            Some("2026-10-25T07:40:00Z"),
            "2026-10-25T18:00:00Z",
            "17 10 * * *",
        )
        .unwrap();
        assert_eq!(n1, 1);
        assert_eq!(n2, 0);
        let rows: Vec<IngestLogRow> = data::read_csv(&path(&root)).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].game, "powerball");
        assert_eq!(rows[0].draw_date, d("2026-10-24"));
        assert_eq!(rows[0].schedule_slot, "17 6 * * *");
        assert_eq!(rows[0].run_started_at, "2026-10-25T14:22:00Z");
    }

    #[test]
    fn empty_new_dates_does_not_create_file() {
        let root = tmp();
        let n = append_for_new_draws(
            &root,
            "powerball",
            &[],
            None,
            "2026-10-25T14:22:00Z",
            "manual",
        )
        .unwrap();
        assert_eq!(n, 0);
        assert!(!path(&root).exists());
    }

    #[test]
    fn multiple_dates_same_run_metadata() {
        let root = tmp();
        let dates = [d("2026-10-22"), d("2026-10-24")];
        let n = append_for_new_draws(
            &root,
            "powerball",
            &dates,
            Some("2026-10-25T07:40:00Z"),
            "2026-10-25T14:22:00Z",
            "manual",
        )
        .unwrap();
        assert_eq!(n, 2);
        let rows: Vec<IngestLogRow> = data::read_csv(&path(&root)).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter()
                .all(|r| r.run_started_at == "2026-10-25T14:22:00Z")
        );
        assert!(rows.iter().all(|r| r.schedule_slot == "manual"));
    }

    #[test]
    fn blank_ny_rows_updated_at_when_none() {
        let root = tmp();
        append_for_new_draws(
            &root,
            "megamillions",
            &[d("2026-10-23")],
            None,
            "2026-10-24T14:00:00Z",
            "local",
        )
        .unwrap();
        let rows: Vec<IngestLogRow> = data::read_csv(&path(&root)).unwrap();
        assert_eq!(rows[0].ny_rows_updated_at, "");
    }

    /// R4-01 / D-029: corrupt existing file must fail the update and leave bytes unchanged.
    #[test]
    fn corrupt_existing_file_fails_closed() {
        let root = tmp();
        let p = path(&root);
        let garbage = "game,draw_date,ny_rows_updated_at,run_started_at,schedule_slot\n\
not-a-date,powerball,x,y,z\n";
        std::fs::write(&p, garbage).unwrap();
        let before = std::fs::read(&p).unwrap();
        let err = append_for_new_draws(
            &root,
            "powerball",
            &[d("2026-10-26")],
            Some("2026-10-27T07:30:00Z"),
            "2026-10-27T10:17:00Z",
            "17 6 * * *",
        )
        .unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("corrupt") || msg.contains("unparseable") || msg.contains("D-029"),
            "error should mention corrupt/unparseable/D-029, got: {msg}"
        );
        let after = std::fs::read(&p).unwrap();
        assert_eq!(before, after, "corrupt file must not be rewritten");
    }

    /// Wrong header / truncated body is also fail-closed.
    #[test]
    fn wrong_header_existing_file_fails_closed() {
        let root = tmp();
        let p = path(&root);
        let garbage = "wrong,header,here\n1,2,3\n";
        std::fs::write(&p, garbage).unwrap();
        let before = std::fs::read(&p).unwrap();
        let err = append_for_new_draws(
            &root,
            "megamillions",
            &[d("2026-10-27")],
            None,
            "2026-10-28T14:00:00Z",
            "manual",
        )
        .unwrap_err();
        assert!(err.to_string().contains("ingest_log") || format!("{err:#}").contains("D-029"));
        assert_eq!(before, std::fs::read(&p).unwrap());
    }
}
