//! `lottotool export`: draws as CSV or JSON (optionally filtered by era / date range).
use crate::data::{DRAWS_HEADER, Draw};
use anyhow::Result;
use chrono::NaiveDate;
use std::io::Write;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Format {
    Csv,
    Json,
}

pub fn filter<'a>(
    draws: &'a [Draw],
    era: Option<&str>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Vec<&'a Draw> {
    draws
        .iter()
        .filter(|d| era.is_none_or(|e| d.era_id == e))
        .filter(|d| from.is_none_or(|f| d.draw_date >= f))
        .filter(|d| to.is_none_or(|t| d.draw_date <= t))
        .collect()
}

pub fn write(rows: &[&Draw], fmt: Format, out: &mut dyn Write) -> Result<()> {
    match fmt {
        Format::Json => {
            serde_json::to_writer_pretty(&mut *out, rows)?;
            writeln!(out)?;
        }
        Format::Csv => {
            let mut w = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(out);
            w.write_record(DRAWS_HEADER.split(','))?;
            for r in rows {
                w.serialize(r)?;
            }
            w.flush()?;
        }
    }
    Ok(())
}
