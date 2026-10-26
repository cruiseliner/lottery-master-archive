//! check/update logic on small SYNTHETIC fixtures (numbers below are test inputs, not data).
use chrono::{NaiveDate, TimeZone, Utc};
use lottotool::Game;
use lottotool::check::{self, Options, Status};
use lottotool::data::{self, DRAWS_HEADER, Draw, INGEST_LOG_HEADER, IngestLogRow, MISSING_HEADER};
use lottotool::eras::ErasFile;
use lottotool::ingest_log;
use lottotool::ny;
use std::path::{Path, PathBuf};

fn eras() -> ErasFile {
    ErasFile::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("eras.toml")).unwrap()
}

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

fn draw(date: &str, w: [u32; 5], b: u32, m: Option<u32>, era: &str) -> Draw {
    Draw {
        draw_date: d(date),
        era_id: era.into(),
        w1: w[0],
        w2: w[1],
        w3: w[2],
        w4: w[3],
        w5: w[4],
        bonus: b,
        multiplier: m,
        dp_w1: None,
        dp_w2: None,
        dp_w3: None,
        dp_w4: None,
        dp_w5: None,
        dp_bonus: None,
        source: "test".into(),
    }
}

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("lottotool-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, game: Game, draws: &[Draw]) {
    let e = eras();
    let g = e.game(game.id()).unwrap();
    data::write_csv(&game.draws_path(root), draws, DRAWS_HEADER).unwrap();
    data::write_csv(
        &game.missing_path(root),
        &ny::missing_rows(game, g, draws),
        MISSING_HEADER,
    )
    .unwrap();
}

fn status(res: &[check::CheckResult], name: &str) -> Status {
    res.iter()
        .find(|c| c.check == name)
        .unwrap_or_else(|| panic!("no check {name}"))
        .status
}

// Mega Millions current era: Tue 2026-10-13, Fri 2026-10-16, Tue 2026-10-20.
fn mm_ok() -> Vec<Draw> {
    vec![
        draw("2026-10-13", [1, 2, 3, 4, 5], 6, None, "mm-2025"),
        draw("2026-10-16", [10, 20, 30, 40, 70], 24, None, "mm-2025"),
        draw("2026-10-20", [7, 8, 9, 10, 11], 1, None, "mm-2025"),
    ]
}

#[test]
fn clean_fixture_passes() {
    let root = tmp("clean");
    write(&root, Game::Megamillions, &mm_ok());
    let now = Utc.with_ymd_and_hms(2026, 10, 21, 12, 0, 0).unwrap();
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: false,
            now: Some(now),
            crosscheck: None,
        },
    );
    assert!(!check::any_fail(&res), "{res:#?}");
    assert_eq!(status(&res, "staleness"), Status::Pass);
}

#[test]
fn out_of_range_and_gap_fail() {
    let root = tmp("bad");
    let mut v = mm_ok();
    v[1].bonus = 25; // 1/24 matrix
    v.remove(0);
    v.insert(0, draw("2026-10-09", [1, 2, 3, 4, 5], 6, None, "mm-2025")); // skips Tue 10-13
    write(&root, Game::Megamillions, &v);
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(status(&res, "draws.ranges"), Status::Fail);
    assert_eq!(status(&res, "draws.schedule_gaps"), Status::Warn); // R-04 historical gaps
    assert_eq!(status(&res, "staleness"), Status::Skip);
    let skip = res.iter().find(|c| c.check == "staleness").unwrap();
    assert!(!skip.detail.is_empty(), "SKIP must carry a reason");
}

#[test]
fn duplicate_whites_fail() {
    let root = tmp("dupw");
    let mut v = mm_ok();
    v[2].w2 = v[2].w1;
    write(&root, Game::Megamillions, &v);
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(status(&res, "draws.ranges"), Status::Fail);
}

#[test]
fn staleness_fails_when_a_due_draw_is_absent() {
    let root = tmp("stale");
    write(&root, Game::Megamillions, &mm_ok());
    // Fri 2026-10-23 draw is due 30h after 00:00 ET = Sat 2026-10-24 06:00 EDT = 10:00 UTC.
    let before = Utc.with_ymd_and_hms(2026, 10, 24, 9, 59, 0).unwrap();
    let after = Utc.with_ymd_and_hms(2026, 10, 24, 10, 0, 0).unwrap();
    let e = eras();
    let g = e.game("megamillions").unwrap();
    assert_eq!(check::latest_due_draw(g, before), Some(d("2026-10-20")));
    assert_eq!(check::latest_due_draw(g, after), Some(d("2026-10-23")));
    let res = check::run(
        &root,
        &e,
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: false,
            now: Some(after),
            crosscheck: None,
        },
    );
    assert_eq!(status(&res, "staleness"), Status::Fail);
    assert!(check::any_fail(&res));
}

#[test]
fn undocumented_or_stale_missing_csv_fails() {
    let root = tmp("missing");
    let mut v = mm_ok();
    write(&root, Game::Megamillions, &v);
    // Change data after missing.csv was generated: coverage must no longer match.
    v[0].multiplier = Some(2);
    data::write_csv(&Game::Megamillions.draws_path(&root), &v, DRAWS_HEADER).unwrap();
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(status(&res, "missing.coverage"), Status::Fail);
}

#[test]
fn null_runs_split_by_rule() {
    let e = eras();
    let g = e.game("powerball").unwrap();
    let v = vec![
        draw("2014-01-15", [1, 2, 3, 4, 5], 6, None, "pb-2012"),
        draw("2014-01-18", [1, 2, 3, 4, 5], 6, None, "pb-2012"),
        draw("2014-01-22", [1, 2, 3, 4, 5], 6, None, "pb-2012"),
    ];
    let m: Vec<_> = ny::missing_rows(Game::Powerball, g, &v)
        .into_iter()
        .filter(|r| r.field == "multiplier")
        .collect();
    assert_eq!(m.len(), 2);
    assert_eq!(m[0].status, "MISSING");
    assert_eq!(m[0].rows, 2);
    assert_eq!(m[1].status, "UNDOCUMENTED");
}

#[test]
fn era_lookup_keys_on_first_draw() {
    let e = eras();
    let mm = e.game("megamillions").unwrap();
    assert_eq!(mm.era_for(d("2025-04-04")).unwrap().id, "mm-2017");
    assert_eq!(mm.era_for(d("2025-04-08")).unwrap().id, "mm-2025");
    assert_eq!(mm.era_for(d("2013-10-18")).unwrap().id, "mm-2005");
    assert_eq!(mm.era_for(d("2013-10-22")).unwrap().id, "mm-2013");
    let pb = e.game("powerball").unwrap();
    assert_eq!(pb.era_for(d("2015-10-03")).unwrap().id, "pb-2012");
    assert_eq!(pb.era_for(d("2015-10-07")).unwrap().id, "pb-2015");
    assert!(pb.is_draw_day(d("2021-08-23")));
    assert!(!pb.is_draw_day(d("2021-08-16")));
}

#[test]
fn ny_normalise_strips_padding() {
    let e = eras();
    let g = e.game("megamillions").unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_str(
        r#"[{"draw_date":"2026-10-20T00:00:00.000","winning_numbers":"01 02 03 04 05","mega_ball":"03","multiplier":"04"},
            {"draw_date":"2026-10-16T00:00:00.000","winning_numbers":"10 20 30 40 50","mega_ball":"3"}]"#,
    )
    .unwrap();
    let v = ny::normalise(Game::Megamillions, g, &rows).unwrap();
    assert_eq!(v[0].draw_date, d("2026-10-16"), "sorted ascending");
    assert_eq!(v[1].bonus, 3);
    assert_eq!(v[1].multiplier, Some(4));
    assert_eq!(v[0].multiplier, None);
    assert_eq!(v[1].whites(), [1, 2, 3, 4, 5]);
}

#[test]
fn non_ascending_whites_fail() {
    let root = tmp("asc");
    let mut v = mm_ok();
    // swap w2/w3 so whites are not ascending
    v[0].w2 = 4;
    v[0].w3 = 3;
    write(&root, Game::Megamillions, &v);
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(status(&res, "draws.whites_ascending"), Status::Fail);
}

#[test]
fn schedule_exception_removes_gap() {
    let mut e = eras();
    let g = e.game.iter_mut().find(|g| g.id == "megamillions").unwrap();
    // Cancel Tue 2026-10-13 so a fixture that skips it is clean for gaps.
    g.schedule_exception
        .push(lottotool::eras::ScheduleException {
            date: d("2026-10-13"),
            reason: "test cancellation".into(),
            source: "test".into(),
        });
    let root = tmp("exc");
    let v = vec![
        draw("2026-10-16", [10, 20, 30, 40, 70], 24, None, "mm-2025"),
        draw("2026-10-20", [7, 8, 9, 10, 11], 1, None, "mm-2025"),
    ];
    // write missing using modified eras
    data::write_csv(&Game::Megamillions.draws_path(&root), &v, DRAWS_HEADER).unwrap();
    data::write_csv(
        &Game::Megamillions.missing_path(&root),
        &ny::missing_rows(Game::Megamillions, g, &v),
        MISSING_HEADER,
    )
    .unwrap();
    let res = check::run(
        &root,
        &e,
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(
        status(&res, "draws.schedule_gaps"),
        Status::Pass,
        "{res:#?}"
    );
}

#[test]
fn schema_files_match_code() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for (f, h) in [
        ("schemas/draws.csv", DRAWS_HEADER),
        ("schemas/missing.csv", MISSING_HEADER),
        ("schemas/payouts.csv", data::PAYOUTS_HEADER),
        ("schemas/ingest_log.csv", data::INGEST_LOG_HEADER),
    ] {
        let s = std::fs::read_to_string(root.join(f)).unwrap();
        assert_eq!(s.trim_end(), h, "{f}");
    }
}

#[test]
fn ingest_log_absent_ok_before_cutoff() {
    let root = tmp("ingest-absent");
    write(&root, Game::Megamillions, &mm_ok());
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(status(&res, "ingest_log.present"), Status::Pass, "{res:#?}");
    assert!(!check::any_fail(&res), "{res:#?}");
}

#[test]
fn ingest_log_header_only_passes_on_current_cutoffs() {
    let root = tmp("ingest-header");
    write(&root, Game::Megamillions, &mm_ok());
    // Header-only file, no post-cutoff draws in fixture.
    data::write_csv(
        &ingest_log::path(&root),
        &[] as &[IngestLogRow],
        INGEST_LOG_HEADER,
    )
    .unwrap();
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(status(&res, "ingest_log.header"), Status::Pass, "{res:#?}");
    assert_eq!(status(&res, "ingest_log.parse"), Status::Pass, "{res:#?}");
    assert_eq!(status(&res, "ingest_log.unique"), Status::Pass, "{res:#?}");
    assert_eq!(status(&res, "ingest_log.refs"), Status::Pass, "{res:#?}");
    assert_eq!(
        status(&res, "ingest_log.coverage"),
        Status::Pass,
        "{res:#?}"
    );
    assert!(!check::any_fail(&res), "{res:#?}");
}

#[test]
fn ingest_log_orphan_row_fails_refs() {
    let root = tmp("ingest-orphan");
    write(&root, Game::Megamillions, &mm_ok());
    let rows = vec![IngestLogRow {
        game: "megamillions".into(),
        draw_date: d("2099-01-01"),
        ny_rows_updated_at: "".into(),
        run_started_at: "2026-10-26T12:00:00Z".into(),
        schedule_slot: "manual".into(),
    }];
    data::write_csv(&ingest_log::path(&root), &rows, INGEST_LOG_HEADER).unwrap();
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(status(&res, "ingest_log.refs"), Status::Fail, "{res:#?}");
    assert!(check::any_fail(&res));
}

#[test]
fn ingest_log_missing_coverage_for_post_cutoff_draw() {
    // MM cutoff is 2026-10-23; a draw after that without a log row must FAIL coverage.
    let root = tmp("ingest-coverage");
    let draws = vec![
        draw("2026-10-23", [3, 7, 46, 49, 51], 5, None, "mm-2025"),
        draw("2026-10-27", [1, 2, 3, 4, 5], 6, None, "mm-2025"),
    ];
    write(&root, Game::Megamillions, &draws);
    data::write_csv(
        &ingest_log::path(&root),
        &[] as &[IngestLogRow],
        INGEST_LOG_HEADER,
    )
    .unwrap();
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(
        status(&res, "ingest_log.coverage"),
        Status::Fail,
        "{res:#?}"
    );
    assert!(check::any_fail(&res));
}

#[test]
fn ingest_log_post_cutoff_with_matching_row_passes() {
    let root = tmp("ingest-covered");
    let draws = vec![
        draw("2026-10-23", [3, 7, 46, 49, 51], 5, None, "mm-2025"),
        draw("2026-10-27", [1, 2, 3, 4, 5], 6, None, "mm-2025"),
    ];
    write(&root, Game::Megamillions, &draws);
    let rows = vec![IngestLogRow {
        game: "megamillions".into(),
        draw_date: d("2026-10-27"),
        ny_rows_updated_at: "2026-10-28T07:30:00Z".into(),
        run_started_at: "2026-10-28T10:17:00Z".into(),
        schedule_slot: "17 6 * * *".into(),
    }];
    data::write_csv(&ingest_log::path(&root), &rows, INGEST_LOG_HEADER).unwrap();
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(
        status(&res, "ingest_log.coverage"),
        Status::Pass,
        "{res:#?}"
    );
    assert_eq!(status(&res, "ingest_log.refs"), Status::Pass, "{res:#?}");
    assert!(!check::any_fail(&res), "{res:#?}");
}

#[test]
fn ingest_log_duplicate_key_fails_unique() {
    let root = tmp("ingest-dup");
    write(&root, Game::Megamillions, &mm_ok());
    let rows = vec![
        IngestLogRow {
            game: "megamillions".into(),
            draw_date: d("2026-10-20"),
            ny_rows_updated_at: "".into(),
            run_started_at: "a".into(),
            schedule_slot: "manual".into(),
        },
        IngestLogRow {
            game: "megamillions".into(),
            draw_date: d("2026-10-20"),
            ny_rows_updated_at: "".into(),
            run_started_at: "b".into(),
            schedule_slot: "manual".into(),
        },
    ];
    data::write_csv(&ingest_log::path(&root), &rows, INGEST_LOG_HEADER).unwrap();
    let res = check::run(
        &root,
        &eras(),
        &[Game::Megamillions],
        &Options {
            stale_as_warn: false,
            no_stale: true,
            now: None,
            crosscheck: None,
        },
    );
    assert_eq!(status(&res, "ingest_log.unique"), Status::Fail, "{res:#?}");
    assert!(check::any_fail(&res));
}

/// Fall-back 2026-11-01: 06:00 America/New_York is EST (UTC-5) → 11:00 UTC.
/// Powerball Sat 2026-10-31 is due Sun 2026-11-01 06:00 ET.
#[test]
fn dst_fallback_2026_11_01_due_at_1100_utc() {
    let e = eras();
    let g = e.game("powerball").unwrap();
    let before = Utc.with_ymd_and_hms(2026, 11, 1, 10, 59, 0).unwrap();
    let after = Utc.with_ymd_and_hms(2026, 11, 1, 11, 1, 0).unwrap();
    assert_eq!(
        check::latest_due_draw(g, before),
        Some(d("2026-10-28")),
        "10:59Z must still be before 06:00 EST due for Sat 10-31"
    );
    assert_eq!(
        check::latest_due_draw(g, after),
        Some(d("2026-10-31")),
        "11:01Z must make Sat 2026-10-31 due"
    );
}

/// Spring-forward 2027-03-14: 06:00 America/New_York is EDT (UTC-4) → 10:00 UTC.
/// Powerball Sat 2027-03-13 is due Sun 2027-03-14 06:00 ET.
#[test]
fn dst_spring_forward_2027_03_14_due_at_1000_utc() {
    let e = eras();
    let g = e.game("powerball").unwrap();
    let before = Utc.with_ymd_and_hms(2027, 3, 14, 9, 59, 0).unwrap();
    let after = Utc.with_ymd_and_hms(2027, 3, 14, 10, 1, 0).unwrap();
    assert_eq!(
        check::latest_due_draw(g, before),
        Some(d("2027-03-10")),
        "09:59Z must still be before 06:00 EDT due for Sat 03-13"
    );
    assert_eq!(
        check::latest_due_draw(g, after),
        Some(d("2027-03-13")),
        "10:01Z must make Sat 2027-03-13 due"
    );
}
