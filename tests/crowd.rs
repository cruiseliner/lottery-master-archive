//! Anti-crowd heuristic (D-032). Synthetic combinations and synthetic draws only.

use chrono::NaiveDate;
use lottotool::crowd::{self, CrowdReport, DISCLAIMER, MAX_SCORE};
use lottotool::data::Draw;
use lottotool::eras::{Era, ErasFile};
use std::path::Path;

fn era(game: &str) -> Era {
    ErasFile::load(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("eras.toml")
            .as_path(),
    )
    .unwrap()
    .game(game)
    .unwrap()
    .current_era()
    .clone()
}

fn pb(whites: &[u32], special: Option<u32>) -> CrowdReport {
    crowd::score("powerball", &era("powerball"), whites, special, &[]).unwrap()
}

fn points(r: &CrowdReport, id: &str) -> u32 {
    r.factors.iter().find(|f| f.id == id).unwrap().points
}

fn draw(date: &str, w: [u32; 5], bonus: u32) -> Draw {
    Draw {
        draw_date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        era_id: "pb-2015".into(),
        w1: w[0],
        w2: w[1],
        w3: w[2],
        w4: w[3],
        w5: w[4],
        bonus,
        multiplier: None,
        dp_w1: None,
        dp_w2: None,
        dp_w3: None,
        dp_w4: None,
        dp_w5: None,
        dp_bonus: None,
        source: "synthetic test draw".into(),
    }
}

#[test]
fn one_to_five_scores_high_and_crowded() {
    let r = pb(&[1, 2, 3, 4, 5], Some(6));
    assert_eq!(r.score, 70);
    assert_eq!(r.band, "high");
    assert_eq!(points(&r, "birthdays"), 25); // all <= 31 and all <= 12
    assert_eq!(points(&r, "sequence"), 25); // equally spaced, step 1
    assert_eq!(points(&r, "same_decade"), 10);
    assert_eq!(points(&r, "low_sum"), 10);
    assert_eq!(points(&r, "past_winner"), 0);
}

#[test]
fn spread_combination_scores_low() {
    let r = pb(&[8, 24, 39, 52, 66], Some(19));
    assert_eq!(r.score, 0, "{:#?}", r.factors);
    assert_eq!(r.band, "low");
    let mm = crowd::score(
        "megamillions",
        &era("megamillions"),
        &[9, 26, 38, 55, 67],
        None,
        &[],
    )
    .unwrap();
    assert_eq!(mm.score, 0, "{:#?}", mm.factors);
}

#[test]
fn factor_maxima_sum_to_max_score_and_score_is_capped() {
    let r = pb(&[1, 2, 3, 4, 5], None);
    assert_eq!(r.factors.iter().map(|f| f.max).sum::<u32>(), MAX_SCORE);
    assert_eq!(r.max_score, 100);
    for f in &r.factors {
        assert!(f.points <= f.max, "{f:?}");
        assert!(!f.reason.is_empty(), "{f:?}");
        assert!(
            f.basis == "judgment" || f.basis.starts_with("Wang et al. 2016"),
            "{f:?}"
        );
    }
}

#[test]
fn input_order_does_not_matter() {
    let a = pb(&[5, 1, 4, 2, 3], None);
    let b = pb(&[1, 2, 3, 4, 5], None);
    assert_eq!(a.whites, [1, 2, 3, 4, 5]);
    assert_eq!(a.score, b.score);
}

#[test]
fn consecutive_runs_and_equal_spacing() {
    assert_eq!(points(&pb(&[20, 21, 22, 23, 60], None), "sequence"), 18);
    assert_eq!(points(&pb(&[20, 21, 22, 40, 60], None), "sequence"), 8);
    assert_eq!(points(&pb(&[20, 21, 40, 41, 60], None), "sequence"), 0);
    // Step 7 sequence (7 14 21 28 35), also a lucky 7.
    let r = pb(&[7, 14, 21, 28, 35], None);
    assert_eq!(points(&r, "sequence"), 25);
    assert!(r.factors[1].reason.contains("step 7"));
}

#[test]
fn birthday_counts_and_month_bonus() {
    assert_eq!(points(&pb(&[2, 9, 17, 25, 31], None), "birthdays"), 20);
    assert_eq!(points(&pb(&[2, 9, 17, 25, 32], None), "birthdays"), 12);
    assert_eq!(points(&pb(&[2, 9, 17, 45, 62], None), "birthdays"), 5);
    assert_eq!(points(&pb(&[2, 9, 40, 45, 62], None), "birthdays"), 0);
    assert_eq!(points(&pb(&[1, 3, 6, 9, 12], None), "birthdays"), 25);
}

#[test]
fn same_decade_and_last_digit() {
    assert_eq!(points(&pb(&[41, 43, 45, 47, 49], None), "same_decade"), 10);
    assert_eq!(points(&pb(&[39, 43, 45, 47, 49], None), "same_decade"), 0);
    let col = pb(&[3, 13, 23, 33, 43], None);
    assert_eq!(points(&col, "last_digit"), 10);
    assert_eq!(points(&col, "sequence"), 25);
    assert_eq!(points(&pb(&[3, 13, 23, 33, 44], None), "last_digit"), 5);
    assert_eq!(points(&pb(&[3, 13, 24, 35, 46], None), "last_digit"), 0);
}

#[test]
fn lucky_sevens_elevens_and_round_numbers() {
    assert_eq!(points(&pb(&[7, 30, 44, 52, 66], None), "lucky_numbers"), 3);
    assert_eq!(
        points(&pb(&[7, 11, 44, 52, 66], Some(7)), "lucky_numbers"),
        6
    ); // capped
    assert_eq!(points(&pb(&[10, 20, 30, 40, 52], None), "lucky_numbers"), 4);
    assert_eq!(points(&pb(&[10, 20, 31, 41, 52], None), "lucky_numbers"), 0);
}

#[test]
fn low_sum_thresholds_use_the_uniform_mean() {
    // PB mean = 5 * 70 / 2 = 175: < 87.5 -> 10, < 122.5 -> 5.
    assert_eq!(points(&pb(&[10, 15, 17, 19, 26], None), "low_sum"), 10); // 87
    assert_eq!(points(&pb(&[10, 15, 17, 19, 27], None), "low_sum"), 5); // 88
    assert_eq!(points(&pb(&[20, 22, 24, 26, 30], None), "low_sum"), 5); // 122
    assert_eq!(points(&pb(&[20, 22, 24, 26, 31], None), "low_sum"), 0); // 123
}

#[test]
fn exact_past_winner_is_a_popularity_flag_only() {
    let draws = vec![
        draw("2099-01-03", [50, 3, 40, 10, 22], 9),
        draw("2099-01-06", [1, 2, 3, 4, 6], 9),
    ];
    let e = era("powerball");
    let hit = crowd::score("powerball", &e, &[3, 10, 22, 40, 50], None, &draws).unwrap();
    assert_eq!(points(&hit, "past_winner"), 10);
    assert_eq!(
        hit.past_winner_dates,
        vec![NaiveDate::from_ymd_opt(2099, 1, 3).unwrap()]
    );
    let reason = &hit
        .factors
        .iter()
        .find(|f| f.id == "past_winner")
        .unwrap()
        .reason;
    assert!(
        reason.contains("popularity concern only; odds unchanged"),
        "{reason}"
    );
    let miss = crowd::score("powerball", &e, &[3, 10, 22, 40, 51], None, &draws).unwrap();
    assert_eq!(points(&miss, "past_winner"), 0);
    assert!(miss.past_winner_dates.is_empty());
}

#[test]
fn draw_history_never_changes_any_other_factor() {
    // Not a predictor: "hot" numbers drawn over and over must not move the score.
    let e = era("powerball");
    let hot: Vec<Draw> = (0..200)
        .map(|i| draw("2099-01-03", [8, 24, 39, 52, 60 + (i % 5)], 1))
        .collect();
    for combo in [[8, 24, 39, 52, 66], [1, 2, 3, 4, 5], [7, 14, 21, 28, 35]] {
        let a = crowd::score("powerball", &e, &combo, Some(7), &[]).unwrap();
        let b = crowd::score("powerball", &e, &combo, Some(7), &hot).unwrap();
        let strip = |r: &CrowdReport| {
            r.factors
                .iter()
                .filter(|f| f.id != "past_winner")
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(strip(&a), strip(&b));
        assert_eq!(a.score, b.score);
    }
}

#[test]
fn invalid_combinations_are_refused() {
    let e = era("powerball");
    let bad = |w: &[u32], s: Option<u32>| crowd::score("powerball", &e, w, s, &[]).is_err();
    assert!(bad(&[1, 2, 3, 4], None));
    assert!(bad(&[1, 2, 3, 4, 5, 6], None));
    assert!(bad(&[1, 1, 3, 4, 5], None));
    assert!(bad(&[0, 2, 3, 4, 5], None));
    assert!(bad(&[1, 2, 3, 4, 70], None));
    assert!(bad(&[1, 2, 3, 4, 5], Some(27)));
    assert!(bad(&[1, 2, 3, 4, 5], Some(0)));
    assert!(!bad(&[1, 2, 3, 4, 69], Some(26)));
}

#[test]
fn disclaimer_says_odds_are_unchanged() {
    assert!(DISCLAIMER.contains("does NOT make a ticket likelier to win"));
    assert!(DISCLAIMER.contains("exactly the same odds"));
    assert!(DISCLAIMER.contains("splitting a jackpot"));
    assert_eq!(pb(&[1, 2, 3, 4, 5], None).disclaimer, DISCLAIMER);
}
