//! `lottotool summary`: counts, ranges and ball frequencies (current era by default).
use crate::data::Draw;
use crate::eras::GameDef;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Summary {
    pub game: String,
    pub rows: usize,
    pub first_draw: Option<String>,
    pub last_draw: Option<String>,
    pub per_era: Vec<EraCount>,
    pub latest: Option<Draw>,
    pub frequency_era: String,
    pub frequency_rows: usize,
    /// (ball, count) sorted by ball.
    pub white_freq: Vec<(u32, usize)>,
    pub bonus_freq: Vec<(u32, usize)>,
    pub multiplier_null_rows: usize,
    pub double_play_rows: usize,
}

#[derive(Debug, Serialize)]
pub struct EraCount {
    pub era_id: String,
    pub rows: usize,
}

pub fn build(g: &GameDef, draws: &[Draw], era_id: Option<&str>) -> Summary {
    let era = era_id
        .and_then(|id| g.era.iter().find(|e| e.id == id))
        .unwrap_or_else(|| g.current_era());
    let in_era: Vec<&Draw> = draws.iter().filter(|d| d.era_id == era.id).collect();
    let mut wf = vec![0usize; era.white_max as usize + 1];
    let mut bf = vec![0usize; era.bonus_max as usize + 1];
    for d in &in_era {
        for w in d.whites() {
            if (w as usize) < wf.len() {
                wf[w as usize] += 1;
            }
        }
        if (d.bonus as usize) < bf.len() {
            bf[d.bonus as usize] += 1;
        }
    }
    Summary {
        game: g.id.clone(),
        rows: draws.len(),
        first_draw: draws.first().map(|d| d.draw_date.to_string()),
        last_draw: draws.last().map(|d| d.draw_date.to_string()),
        per_era: g
            .era
            .iter()
            .map(|e| EraCount {
                era_id: e.id.clone(),
                rows: draws.iter().filter(|d| d.era_id == e.id).count(),
            })
            .collect(),
        latest: draws.last().cloned(),
        frequency_era: era.id.clone(),
        frequency_rows: in_era.len(),
        white_freq: (1..=era.white_max).map(|b| (b, wf[b as usize])).collect(),
        bonus_freq: (1..=era.bonus_max).map(|b| (b, bf[b as usize])).collect(),
        multiplier_null_rows: draws.iter().filter(|d| d.multiplier.is_none()).count(),
        double_play_rows: draws.iter().filter(|d| d.dp().is_some()).count(),
    }
}
