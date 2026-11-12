//! v0.2 Step 3 coverage set (D-033). Synthetic only: the pinned base designs and seeded
//! relabellings; no ledger, no real picks.

use lottotool::eras::{Era, ErasFile};
use lottotool::wheel::{self, MM_DESIGN, PB_DESIGN, Wheel};
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

#[test]
fn pinned_designs_verify_for_pb_69_and_mm_70() {
    let pb = wheel::verify(&PB_DESIGN.wheel()).unwrap();
    assert_eq!((pb.groups, pb.tickets), (4, 63));
    assert_eq!(pb.group_sizes, vec![21, 21, 11, 16]);
    assert_eq!(pb.required_pairs, 210 + 210 + 55 + 120);
    let mm = wheel::verify(&MM_DESIGN.wheel()).unwrap();
    assert_eq!((mm.groups, mm.tickets), (4, 65));
    assert_eq!(mm.group_sizes, vec![21, 21, 13, 15]);
    assert_eq!(mm.required_pairs, 210 + 210 + 78 + 105);
    assert_eq!(era("powerball").white_max, PB_DESIGN.white_max);
    assert_eq!(era("megamillions").white_max, MM_DESIGN.white_max);
}

#[test]
fn removing_any_ticket_breaks_verification() {
    for d in [PB_DESIGN, MM_DESIGN] {
        let full = d.wheel();
        for i in 0..full.tickets.len() {
            let mut w = full.clone();
            w.tickets.remove(i);
            let e = wheel::verify(&w).unwrap_err().to_string();
            assert!(e.contains("is on no ticket; coverage fails"), "{i}: {e}");
        }
    }
}

#[test]
fn verify_rejects_bad_structures() {
    let base = PB_DESIGN.wheel();
    // Five groups: pigeonhole no longer forces two drawn whites into one group.
    let mut w = base.clone();
    let last = w.groups.pop().unwrap();
    w.groups.push(last[..8].to_vec());
    w.groups.push(last[8..].to_vec());
    assert!(
        wheel::verify(&w)
            .unwrap_err()
            .to_string()
            .contains("pigeonhole")
    );
    // A number missing from every group.
    let mut w = base.clone();
    w.groups[3].retain(|&x| x != 69);
    assert!(
        wheel::verify(&w)
            .unwrap_err()
            .to_string()
            .contains("in no group")
    );
    // A number in two groups.
    let mut w = base.clone();
    w.groups[0].push(22);
    assert!(
        wheel::verify(&w)
            .unwrap_err()
            .to_string()
            .contains("more than one group")
    );
    // Duplicate and out-of-range whites.
    let mut w = base.clone();
    w.tickets[0] = [1, 1, 2, 3, 4];
    assert!(
        wheel::verify(&w)
            .unwrap_err()
            .to_string()
            .contains("duplicate")
    );
    let mut w = base.clone();
    w.tickets[0][4] = 70;
    assert!(
        wheel::verify(&w)
            .unwrap_err()
            .to_string()
            .contains("outside 1..=69")
    );
}

#[test]
fn relabelling_is_a_permutation_and_keeps_coverage() {
    for (d, seed) in [(PB_DESIGN, 7u64), (MM_DESIGN, 8)] {
        let mut rng = lottotool::rng::seeded(seed);
        for _ in 0..20 {
            let p = wheel::random_permutation(&mut rng, d.white_max);
            let mut sorted = p.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (1..=d.white_max).collect::<Vec<_>>());
            let w = wheel::relabel(&d.wheel(), &p);
            wheel::verify(&w).unwrap();
        }
    }
}

/// Seeded random draws: every one shares 2+ whites with some relabelled ticket.
#[test]
fn random_draws_always_hit_two_whites() {
    for (game, d) in [("powerball", PB_DESIGN), ("megamillions", MM_DESIGN)] {
        let e = era(game);
        let plan = wheel::plan(game, &e, &d, 99, 4, &[]).unwrap();
        let m = lottotool::odds::Matrix {
            white_count: 5,
            white_max: d.white_max,
            bonus_max: e.bonus_max,
        };
        let mut rng = lottotool::rng::seeded(1234);
        for _ in 0..20_000 {
            let (draw, _) = lottotool::rng::pick(&mut rng, &m);
            assert!(
                plan.tickets
                    .iter()
                    .any(|t| t.whites.iter().filter(|x| draw.contains(x)).count() >= 2),
                "{game}: draw {draw:?} missed"
            );
        }
    }
}

#[test]
#[ignore = "exhaustive: all C(69,5) + C(70,5) draws; run with --release -- --ignored"]
fn exhaustive_every_draw_hits_two_whites() {
    for d in [PB_DESIGN, MM_DESIGN] {
        assert_eq!(wheel::exhaustive_failure(&d.wheel()), None);
        let mut rng = lottotool::rng::seeded(5);
        let p = wheel::random_permutation(&mut rng, d.white_max);
        assert_eq!(
            wheel::exhaustive_failure(&wheel::relabel(&d.wheel(), &p)),
            None
        );
    }
}

#[test]
fn exhaustive_check_finds_a_hole_when_a_ticket_is_missing() {
    // Small synthetic case (n = 9, 4 groups) so the exhaustive check runs in debug builds.
    let full = Wheel {
        n: 9,
        groups: vec![vec![1, 2, 3], vec![4, 5], vec![6, 7], vec![8, 9]],
        tickets: vec![[1, 2, 3, 4, 5], [6, 7, 8, 9, 1]],
    };
    wheel::verify(&full).unwrap();
    assert_eq!(wheel::exhaustive_failure(&full), None);
    let mut broken = full.clone();
    broken.tickets.pop();
    assert!(wheel::verify(&broken).is_err());
    assert!(wheel::exhaustive_failure(&broken).is_some());
}

#[test]
fn plan_is_seeded_and_picks_the_lowest_mean_crowd_score() {
    let e = era("powerball");
    let a = wheel::plan("powerball", &e, &PB_DESIGN, 42, 8, &[]).unwrap();
    let b = wheel::plan("powerball", &e, &PB_DESIGN, 42, 8, &[]).unwrap();
    assert_eq!(a.permutation, b.permutation);
    assert_eq!(a.tickets.len(), 63);
    let one = wheel::plan("powerball", &e, &PB_DESIGN, 42, 1, &[]).unwrap();
    assert!(a.mean_crowd_score <= one.mean_crowd_score);
    for t in &a.tickets {
        assert!((1..=e.bonus_max).contains(&t.special));
    }
    let mean = a.tickets.iter().map(|t| t.crowd_score).sum::<u32>() as f64 / a.tickets.len() as f64;
    assert!((mean - a.mean_crowd_score).abs() < 1e-9);
    assert!(wheel::plan("powerball", &e, &PB_DESIGN, 42, 0, &[]).is_err());
    assert!(wheel::plan("powerball", &e, &MM_DESIGN, 42, 1, &[]).is_err());
}

#[test]
fn budget_is_refused_below_the_minimum() {
    assert_eq!(wheel::total_cost(63, 3), 189);
    assert_eq!(wheel::total_cost(65, 5), 325);
    let e = wheel::check_budget(188, 189, 63).unwrap_err().to_string();
    assert!(e.contains("minimum budget $189"), "{e}");
    assert!(e.contains("no partial"), "{e}");
    wheel::check_budget(189, 189, 63).unwrap();
}

#[test]
fn guarantee_text_is_exact() {
    assert_eq!(
        wheel::GUARANTEE,
        "At least one ticket matches 2 or more whites for every possible draw."
    );
    let all = wheel::CAVEATS.join(" ");
    for s in [
        "guarantees no prize",
        "pays $0",
        "Expected value is negative",
        "same odds",
        "independent",
    ] {
        assert!(all.contains(s), "{s}");
    }
    // 2 whites without the special is not a prize tier (both games use the same tiers).
    assert!(!lottotool::tickets::is_prize_tier(2, false));
}
