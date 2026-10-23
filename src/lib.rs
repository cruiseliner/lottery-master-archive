//! lottotool: Powerball / Mega Millions draw archive, validator and odds tool.
//!
//! Public data policy: only NY Open Data winning numbers are committed under `data/`.
//! Per-tier winner/payout tables are fetched into the gitignored `cache/` only.

pub mod check;
pub mod data;
pub mod eras;
pub mod export;
pub mod ny;
pub mod odds;
pub mod payouts;
pub mod rng;
pub mod summary;

use std::path::{Path, PathBuf};

/// Supported games.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Powerball,
    Megamillions,
}

impl Game {
    pub fn id(&self) -> &'static str {
        match self {
            Game::Powerball => "powerball",
            Game::Megamillions => "megamillions",
        }
    }
    pub fn all() -> [Game; 2] {
        [Game::Powerball, Game::Megamillions]
    }
    pub fn data_dir(&self, root: &Path) -> PathBuf {
        root.join("data").join(self.id())
    }
    pub fn draws_path(&self, root: &Path) -> PathBuf {
        self.data_dir(root).join("draws.csv")
    }
    pub fn missing_path(&self, root: &Path) -> PathBuf {
        self.data_dir(root).join("missing.csv")
    }
}
