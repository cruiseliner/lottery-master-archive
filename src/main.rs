use anyhow::{Context, Result, bail};
use chrono::{DateTime, NaiveDate, Utc};
use clap::{Parser, Subcommand};
use lottotool::check::{self, Status};
use lottotool::data::{self, DRAWS_HEADER, Draw, MISSING_HEADER};
use lottotool::eras::ErasFile;
use lottotool::odds::{Matrix, PRIZE_TIERS};
use lottotool::{Game, export, ny, payouts, rng, summary};
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "lottotool",
    version,
    about = "Powerball / Mega Millions draw archive, validator and odds tool"
)]
struct Cli {
    /// Repository root (contains eras.toml and data/).
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,
    /// Machine-readable JSON output.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Fetch winning numbers from NY Open Data and rewrite data/<game>/{draws,missing}.csv.
    /// With --payouts, fetch per-tier winners/prizes for one date into the gitignored cache/ instead.
    Update {
        game: Game,
        /// Fetch per-tier payout tables (private cache only, never committed).
        #[arg(long)]
        payouts: bool,
        /// Draw date for --payouts (YYYY-MM-DD).
        #[arg(long, requires = "payouts")]
        date: Option<NaiveDate>,
        /// State scope for Powerball payouts, e.g. "fl" (powerball.com &oc=). Default national.
        #[arg(long, requires = "payouts")]
        state: Option<String>,
        /// Fetch and validate but do not write files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate committed data; non-zero exit with --fail-on-error if any check FAILs.
    Check {
        #[arg(long)]
        game: Option<Game>,
        #[arg(long)]
        fail_on_error: bool,
        /// Skip the staleness check (integrity-only).
        #[arg(long)]
        no_stale: bool,
        /// Evaluate staleness as of this instant (RFC 3339), default now.
        #[arg(long)]
        now: Option<DateTime<Utc>>,
        /// Cross-check the N most recent draws against official winning-number
        /// sources (local / workflow_dispatch only; do not use on schedule).
        #[arg(long)]
        crosscheck: Option<usize>,
    },
    /// List eras with sourced/inferred/missing status and exact jackpot odds.
    Eras {
        #[arg(long)]
        game: Option<Game>,
        /// Monte Carlo check of the current-era prize tiers with N simulated plays.
        #[arg(long)]
        simulate: Option<u64>,
        /// Seed for the ChaCha20 RNG used by --simulate.
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },
    /// Row counts, date range, per-era counts and ball frequencies.
    Summary {
        game: Game,
        /// Era for the frequency table (default: current era).
        #[arg(long)]
        era: Option<String>,
    },
    /// Export draws as CSV or JSON.
    Export {
        game: Game,
        #[arg(long, value_enum, default_value = "csv")]
        format: export::Format,
        #[arg(long)]
        era: Option<String>,
        #[arg(long)]
        from: Option<NaiveDate>,
        #[arg(long)]
        to: Option<NaiveDate>,
        /// Output file (default stdout).
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(code) => code,
        Err(e) => {
            if cli.json {
                println!("{}", json!({"ok": false, "error": format!("{e:#}")}));
            } else {
                eprintln!("error: {e:#}");
            }
            ExitCode::from(2)
        }
    }
}

fn load_draws(cli: &Cli, game: Game) -> Result<Vec<Draw>> {
    data::read_csv(&game.draws_path(&cli.root))
}

fn run(cli: &Cli) -> Result<ExitCode> {
    let eras = ErasFile::load(&cli.root.join("eras.toml"))?;
    match &cli.cmd {
        Cmd::Update {
            game,
            payouts: true,
            date,
            state,
            dry_run,
        } => {
            let date = date.context("--payouts needs --date YYYY-MM-DD")?;
            let rows = match game {
                Game::Powerball => payouts::fetch_powerball(date, state.as_deref())?,
                Game::Megamillions => {
                    if state.is_some() {
                        bail!(
                            "--state is only supported for powerball (megamillions.com per-state counts: MISSING)"
                        );
                    }
                    payouts::fetch_megamillions(date)?
                }
            };
            let scope = state.as_deref().unwrap_or("national");
            let path = if *dry_run {
                None
            } else {
                Some(payouts::write_cache(
                    &cli.root,
                    game.id(),
                    date,
                    scope,
                    &rows,
                )?)
            };
            if cli.json {
                println!(
                    "{}",
                    json!({"ok": true, "game": game, "date": date, "scope": scope, "rows": rows.len(),
                    "written": path.as_ref().map(|p| p.display().to_string()), "note": "private cache only; not for commit"})
                );
            } else {
                println!(
                    "{} payout rows for {} {date} ({scope})",
                    rows.len(),
                    game.id()
                );
                if let Some(p) = path {
                    println!("wrote {} (gitignored cache; do not commit)", p.display());
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Update { game, dry_run, .. } => {
            let g = eras.game(game.id())?;
            let raw = ny::fetch_rows(g)?;
            let draws = ny::normalise(*game, g, &raw)?;
            let missing = ny::missing_rows(*game, g, &draws);
            let old = data::read_csv::<Draw>(&game.draws_path(&cli.root)).unwrap_or_default();
            let added = draws
                .iter()
                .filter(|d| !old.iter().any(|o| o.draw_date == d.draw_date))
                .count();
            let changed = draws
                .iter()
                .filter(|d| old.iter().any(|o| o.draw_date == d.draw_date && o != *d))
                .count();
            let change_list: Vec<(Draw, Draw)> = draws
                .iter()
                .filter_map(|d| {
                    old.iter()
                        .find(|o| o.draw_date == d.draw_date && *o != d)
                        .map(|o| (o.clone(), d.clone()))
                })
                .collect();
            if !dry_run {
                data::write_csv(&game.draws_path(&cli.root), &draws, DRAWS_HEADER)?;
                data::write_csv(&game.missing_path(&cli.root), &missing, MISSING_HEADER)?;
                if !change_list.is_empty() {
                    write_changes_md(&cli.root, *game, &change_list)?;
                }
                write_meta_json(&cli.root, &eras)?;
            }
            let first = draws.first().map(|d| d.draw_date.to_string());
            let last = draws.last().map(|d| d.draw_date.to_string());
            if cli.json {
                println!(
                    "{}",
                    json!({"ok": true, "game": game, "source": g.ny_dataset_url, "fetched_rows": raw.len(),
                    "rows": draws.len(), "first_draw": first, "last_draw": last, "added": added, "changed": changed,
                    "missing_ranges": missing.len(), "dry_run": dry_run,
                    "changes_md": if changed > 0 { format!("data/{}/CHANGES.md", game.id()) } else { String::new() }})
                );
            } else {
                println!(
                    "{}: fetched {} NY rows from {}",
                    game.id(),
                    raw.len(),
                    g.ny_dataset_url
                );
                println!(
                    "  rows {} ({} .. {}), added {added}, changed {changed}",
                    draws.len(),
                    first.unwrap_or_default(),
                    last.unwrap_or_default()
                );
                if changed > 0 {
                    println!(
                        "  historical changes documented in data/{}/CHANGES.md",
                        game.id()
                    );
                }
                for m in &missing {
                    println!(
                        "  null {} {}..{} rows={} {}",
                        m.field, m.first_draw, m.last_draw, m.rows, m.status
                    );
                }
                if *dry_run {
                    println!("  (dry run: nothing written)");
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Check {
            game,
            fail_on_error,
            no_stale,
            now,
            crosscheck,
        } => {
            let games: Vec<Game> = game
                .map(|g| vec![g])
                .unwrap_or_else(|| Game::all().to_vec());
            let res = check::run(
                &cli.root,
                &eras,
                &games,
                &check::Options {
                    no_stale: *no_stale,
                    now: *now,
                    crosscheck: *crosscheck,
                },
            );
            let fail = check::any_fail(&res);
            let count = |s: Status| res.iter().filter(|c| c.status == s).count();
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &json!({"ok": !fail, "pass": count(Status::Pass), "warn": count(Status::Warn),
                    "fail": count(Status::Fail), "skip": count(Status::Skip), "results": res})
                    )?
                );
            } else {
                for c in &res {
                    let s = match c.status {
                        Status::Pass => "PASS",
                        Status::Warn => "WARN",
                        Status::Fail => "FAIL",
                        Status::Skip => "SKIP",
                    };
                    println!("{s:4}  {:12} {:28} {}", c.game, c.check, c.detail);
                }
                println!(
                    "summary: {} pass, {} warn, {} fail, {} skip",
                    count(Status::Pass),
                    count(Status::Warn),
                    count(Status::Fail),
                    count(Status::Skip)
                );
            }
            Ok(if fail && *fail_on_error {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        Cmd::Eras {
            game,
            simulate,
            seed,
        } => {
            let mut out = Vec::new();
            for g in eras
                .game
                .iter()
                .filter(|g| game.is_none_or(|x| x.id() == g.id))
            {
                for (i, e) in g.era.iter().enumerate() {
                    let m = Matrix::new(e.white_count, e.white_max, e.bonus_max);
                    out.push(json!({"game": g.id, "era": e.id, "first_draw": e.first_draw, "last_day": g.era_last_day(i),
                        "first_draw_status": e.first_draw_status, "matrix": format!("{}/{} + 1/{}", e.white_count, e.white_max, e.bonus_max),
                        "matrix_status": e.matrix_status, "price_usd": e.price_usd, "price_status": e.price_status,
                        "white_group": e.white_group, "bonus_group": e.bonus_group,
                        "jackpot_one_in": m.total().to_string(), "multiplier": e.multiplier, "source": e.source}));
                }
            }
            let mut sims = Vec::new();
            if let Some(n) = simulate {
                for g in eras
                    .game
                    .iter()
                    .filter(|g| game.is_none_or(|x| x.id() == g.id))
                {
                    let e = g.current_era();
                    let m = Matrix::new(e.white_count, e.white_max, e.bonus_max);
                    let counts = rng::simulate(&m, *n, *seed);
                    let tiers: Vec<_> = PRIZE_TIERS
                        .iter()
                        .enumerate()
                        .map(|(i, &(w, b))| {
                            json!({
                        "tier": format!("{w}+{}", b as u8), "observed": counts[i],
                        "expected": *n as f64 / m.one_in(w, b)})
                        })
                        .collect();
                    sims.push(json!({"game": g.id, "era": e.id, "plays": n, "seed": seed, "rng": "ChaCha20", "tiers": tiers,
                        "no_prize": counts[PRIZE_TIERS.len()]}));
                }
            }
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &json!({"ok": true, "eras": out, "simulations": sims})
                    )?
                );
            } else {
                for v in &out {
                    println!(
                        "{:12} {:8} first_draw {} [{}]  {} [{}]  price {} [{}]  jackpot 1 in {}",
                        v["game"].as_str().unwrap(),
                        v["era"].as_str().unwrap(),
                        v["first_draw"].as_str().unwrap(),
                        v["first_draw_status"].as_str().unwrap(),
                        v["matrix"].as_str().unwrap(),
                        v["matrix_status"].as_str().unwrap(),
                        v["price_usd"]
                            .as_u64()
                            .map(|p| format!("${p}"))
                            .unwrap_or("MISSING".into()),
                        v["price_status"].as_str().unwrap(),
                        v["jackpot_one_in"].as_str().unwrap()
                    );
                }
                for s in &sims {
                    println!(
                        "simulation {} {} plays={} seed={} (ChaCha20)",
                        s["game"].as_str().unwrap(),
                        s["era"].as_str().unwrap(),
                        s["plays"],
                        s["seed"]
                    );
                    for t in s["tiers"].as_array().unwrap() {
                        println!(
                            "  {:4} observed {:>8} expected {:>12.1}",
                            t["tier"].as_str().unwrap(),
                            t["observed"],
                            t["expected"].as_f64().unwrap()
                        );
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Summary { game, era } => {
            let g = eras.game(game.id())?;
            let draws = load_draws(cli, *game)?;
            let s = summary::build(g, &draws, era.as_deref());
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({"ok": true, "summary": s}))?
                );
            } else {
                println!(
                    "{}: {} rows, {} .. {}",
                    s.game,
                    s.rows,
                    s.first_draw.clone().unwrap_or_default(),
                    s.last_draw.clone().unwrap_or_default()
                );
                for e in &s.per_era {
                    println!("  era {:8} rows {}", e.era_id, e.rows);
                }
                if let Some(d) = &s.latest {
                    println!(
                        "  latest {} whites {:?} bonus {} multiplier {}",
                        d.draw_date,
                        d.whites(),
                        d.bonus,
                        d.multiplier.map(|m| m.to_string()).unwrap_or("null".into())
                    );
                }
                println!(
                    "  multiplier null rows {}; double play rows {}",
                    s.multiplier_null_rows, s.double_play_rows
                );
                let mut top = s.white_freq.clone();
                top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                println!(
                    "  era {} ({} draws) most frequent whites: {}",
                    s.frequency_era,
                    s.frequency_rows,
                    top.iter()
                        .take(5)
                        .map(|(b, c)| format!("{b}x{c}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                println!("  (descriptive only: past frequencies do not change future odds)");
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Export {
            game,
            format,
            era,
            from,
            to,
            out,
        } => {
            let draws = load_draws(cli, *game)?;
            let rows = export::filter(&draws, era.as_deref(), *from, *to);
            match out {
                Some(p) => {
                    let mut f = std::fs::File::create(p)?;
                    export::write(&rows, *format, &mut f)?;
                    if cli.json {
                        println!(
                            "{}",
                            json!({"ok": true, "rows": rows.len(), "out": p.display().to_string()})
                        );
                    } else {
                        println!("wrote {} rows to {}", rows.len(), p.display());
                    }
                }
                None => export::write(
                    &rows,
                    if cli.json {
                        export::Format::Json
                    } else {
                        *format
                    },
                    &mut std::io::stdout(),
                )?,
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn write_changes_md(
    root: &std::path::Path,
    game: Game,
    changes: &[(data::Draw, data::Draw)],
) -> Result<()> {
    use std::fmt::Write as _;
    let path = game.data_dir(root).join("CHANGES.md");
    let mut body = String::new();
    let ts = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    writeln!(
        &mut body,
        "# Historical row changes — {}\n\nUpdated: {ts} UTC\n",
        game.id()
    )?;
    writeln!(
        &mut body,
        "Source republished different values for already-committed draw dates. Review before relying on downstream analytics.\n"
    )?;
    writeln!(
        &mut body,
        "| draw_date | field | old | new |\n|---|---|---|---|"
    )?;
    for (old, new) in changes {
        let pairs = [
            ("w1", old.w1.to_string(), new.w1.to_string()),
            ("w2", old.w2.to_string(), new.w2.to_string()),
            ("w3", old.w3.to_string(), new.w3.to_string()),
            ("w4", old.w4.to_string(), new.w4.to_string()),
            ("w5", old.w5.to_string(), new.w5.to_string()),
            ("bonus", old.bonus.to_string(), new.bonus.to_string()),
            (
                "multiplier",
                format!("{:?}", old.multiplier),
                format!("{:?}", new.multiplier),
            ),
            ("era_id", old.era_id.clone(), new.era_id.clone()),
        ];
        for (field, a, b) in pairs {
            if a != b {
                writeln!(&mut body, "| {} | {field} | `{a}` | `{b}` |", old.draw_date)?;
            }
        }
    }
    std::fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn write_meta_json(root: &std::path::Path, eras: &ErasFile) -> Result<()> {
    let mut games = serde_json::Map::new();
    for gdef in &eras.game {
        let game = match gdef.id.as_str() {
            "powerball" => Game::Powerball,
            "megamillions" => Game::Megamillions,
            _ => continue,
        };
        let draws: Vec<data::Draw> = data::read_csv(&game.draws_path(root)).unwrap_or_default();
        games.insert(
            gdef.id.clone(),
            json!({
                "rows": draws.len(),
                "first_draw": draws.first().map(|d| d.draw_date.to_string()),
                "last_draw": draws.last().map(|d| d.draw_date.to_string()),
                "ny_dataset": gdef.ny_dataset,
                "ny_dataset_url": gdef.ny_dataset_url,
                "current_era": gdef.current_era().id,
            }),
        );
    }
    let meta = json!({
        "schema_version": 1,
        "updated_at": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "games": games,
    });
    let path = root.join("data").join("meta.json");
    std::fs::write(&path, serde_json::to_string_pretty(&meta)? + "\n")
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}
