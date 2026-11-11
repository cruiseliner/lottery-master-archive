//! Anti-crowd heuristic (D-032). Synthetic combinations and synthetic draws only.

use chrono::NaiveDate;
use lottotool::crowd::{self, CrowdReport, DISCLAIMER, MAX_SCORE, PastWinner, RAW_MAX};
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
    assert_eq!(r.raw_points, 60);
    assert_eq!(r.raw_max, 90);
    assert_eq!(r.score, 67); // round(60 * 100 / 90)
    assert_eq!(r.band, "high");
    assert_eq!(points(&r, "birthdays"), 20); // all <= 31 and all <= 12
    assert_eq!(points(&r, "sequence"), 20); // equally spaced, step 1
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
    assert_eq!(r.factors.iter().map(|f| f.max).sum::<u32>(), RAW_MAX);
    assert_eq!(RAW_MAX, 90);
    assert_eq!(r.max_score, MAX_SCORE);
    assert_eq!(MAX_SCORE, 100);
    let max = |id: &str| r.factors.iter().find(|f| f.id == id).unwrap().max;
    assert_eq!(max("birthdays"), 20); // Adam t51s258
    assert_eq!(max("sequence"), 20); // Adam t51s258
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
    assert_eq!(points(&pb(&[20, 21, 22, 23, 60], None), "sequence"), 14);
    assert_eq!(points(&pb(&[20, 21, 22, 40, 60], None), "sequence"), 6);
    assert_eq!(points(&pb(&[20, 21, 40, 41, 60], None), "sequence"), 0);
    // Step 7 sequence (7 14 21 28 35), also a lucky 7.
    let r = pb(&[7, 14, 21, 28, 35], None);
    assert_eq!(points(&r, "sequence"), 20);
    // Full sequences only (Adam t51s258 left 4-of-5 open): 4 equally spaced + 1 off scores 0.
    assert_eq!(points(&pb(&[10, 20, 30, 40, 66], None), "sequence"), 0);
    assert!(r.factors[1].reason.contains("step 7"));
}

#[test]
fn birthday_counts_and_month_bonus() {
    assert_eq!(points(&pb(&[2, 9, 17, 25, 31], None), "birthdays"), 16);
    assert_eq!(points(&pb(&[2, 9, 17, 25, 32], None), "birthdays"), 10);
    assert_eq!(points(&pb(&[2, 9, 17, 45, 62], None), "birthdays"), 4);
    assert_eq!(points(&pb(&[2, 9, 40, 45, 62], None), "birthdays"), 0);
    assert_eq!(points(&pb(&[1, 3, 6, 9, 12], None), "birthdays"), 20);
}

#[test]
fn same_decade_and_last_digit() {
    assert_eq!(points(&pb(&[41, 43, 45, 47, 49], None), "same_decade"), 10);
    assert_eq!(points(&pb(&[39, 43, 45, 47, 49], None), "same_decade"), 0);
    let col = pb(&[3, 13, 23, 33, 43], None);
    assert_eq!(points(&col, "last_digit"), 10);
    assert_eq!(points(&col, "sequence"), 20);
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
        hit.past_winners,
        vec![PastWinner {
            draw_date: NaiveDate::from_ymd_opt(2099, 1, 3).unwrap(),
            draw: "main"
        }]
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
    assert!(miss.past_winners.is_empty());
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

#[test]
fn double_play_repeats_are_flagged_and_labeled() {
    let mut d = draw("2099-02-07", [1, 2, 3, 4, 6], 9);
    (d.dp_w1, d.dp_w2, d.dp_w3, d.dp_w4, d.dp_w5, d.dp_bonus) =
        (Some(61), Some(5), Some(33), Some(18), Some(47), Some(12));
    let draws = vec![d, draw("2099-02-09", [18, 5, 33, 47, 61], 3)];
    let e = era("powerball");
    let r = crowd::score("powerball", &e, &[5, 18, 33, 47, 61], None, &draws).unwrap();
    assert_eq!(points(&r, "past_winner"), 10); // flag, not additive
    assert_eq!(
        r.past_winners,
        vec![
            PastWinner {
                draw_date: NaiveDate::from_ymd_opt(2099, 2, 7).unwrap(),
                draw: "double_play"
            },
            PastWinner {
                draw_date: NaiveDate::from_ymd_opt(2099, 2, 9).unwrap(),
                draw: "main"
            },
        ]
    );
    let reason = &r
        .factors
        .iter()
        .find(|f| f.id == "past_winner")
        .unwrap()
        .reason;
    assert!(
        reason.contains("the Double Play draw on 2099-02-07"),
        "{reason}"
    );
    assert!(reason.contains("the draw on 2099-02-09"), "{reason}");
    // A partial Double Play row (any dp_* blank) is not used.
    let mut p = draw("2099-02-11", [1, 2, 3, 4, 6], 9);
    (p.dp_w1, p.dp_w2, p.dp_w3, p.dp_w4, p.dp_w5) =
        (Some(61), Some(5), Some(33), Some(18), Some(47));
    let r = crowd::score("powerball", &e, &[5, 18, 33, 47, 61], None, &[p]).unwrap();
    assert!(r.past_winners.is_empty());
    let miss = crowd::score("powerball", &e, &[8, 24, 39, 52, 66], None, &draws).unwrap();
    let reason = &miss
        .factors
        .iter()
        .find(|f| f.id == "past_winner")
        .unwrap()
        .reason;
    assert!(
        reason.contains("2 main draws, all eras, and 1 Double Play draws checked"),
        "{reason}"
    );
}

#[test]
fn rescaling_keeps_the_bands_as_a_share_of_the_max() {
    assert_eq!(crowd::rescale(0), 0);
    assert_eq!(crowd::rescale(RAW_MAX), 100);
    assert_eq!(crowd::rescale(60), 67);
    assert_eq!(crowd::rescale(17), 19);
    assert_eq!(crowd::rescale(18), 20); // 20% of 90 -> medium
    assert_eq!(crowd::rescale(44), 49);
    assert_eq!(crowd::rescale(45), 50); // 50% of 90 -> high
    assert_eq!(crowd::band(crowd::rescale(17)), "low");
    assert_eq!(crowd::band(crowd::rescale(18)), "medium");
    assert_eq!(crowd::band(crowd::rescale(44)), "medium");
    assert_eq!(crowd::band(crowd::rescale(45)), "high");
    assert_eq!(crowd::rescale(500), 100);
}
