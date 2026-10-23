//! Exact reproduction of published odds for every prize tier.
//! Sources (retrieved 2026-10-23, see docs/DECISIONS.md D-004):
//!  * Powerball 5/69+1/26: Florida Lottery Powerball rules PDF, rule (7)(a) odds to 4 decimals
//!    ("1:11,688,053.5200" ...), overall (7)(b) "1:24.8671".
//!  * Mega Millions 5/70+1/24: megamillions.com GetLatestDrawData PrizeMatrix MatrixID 4 `Odds`
//!    (integers), FooterText "Overall chances of winning a prize are 1 in 23".
//!  * Mega Millions 5/70+1/25 (2017-2025): megamillions.com PrizeMatrix MatrixID 3 `Odds`,
//!    FooterText "1 in 24".
use lottotool::odds::{Matrix, PRIZE_TIERS, comb, round_div};

#[test]
fn binomials() {
    assert_eq!(comb(69, 5), 11_238_513);
    assert_eq!(comb(70, 5), 12_103_014);
    assert_eq!(comb(5, 0), 1);
    assert_eq!(comb(3, 5), 0);
}

#[test]
fn powerball_current_all_tiers() {
    let m = Matrix::new(5, 69, 26);
    assert_eq!(m.total(), 292_201_338);
    // (white, bonus, ways, published odds x 10^4)
    let expect: [(u32, bool, u128, u128); 9] = [
        (5, true, 1, 2_922_013_380_000),
        (5, false, 25, 116_880_535_200),
        (4, true, 320, 9_131_291_813),
        (4, false, 8_000, 365_251_673),
        (3, true, 20_160, 144_941_140),
        (3, false, 504_000, 5_797_646),
        (2, true, 416_640, 7_013_281),
        (1, true, 3_176_880, 919_775),
        (0, true, 7_624_512, 383_239),
    ];
    for (w, b, ways, published) in expect {
        assert_eq!(m.ways(w, b), ways, "ways {w}+{b}");
        assert_eq!(m.one_in_scaled(w, b, 4), published, "odds {w}+{b}");
    }
    assert_eq!(m.overall_scaled(4), 248_671); // 1:24.8671
}

#[test]
fn megamillions_current_all_tiers() {
    let m = Matrix::new(5, 70, 24);
    assert_eq!(m.total(), 290_472_336);
    let expect: [(u32, bool, u128, u128); 9] = [
        (5, true, 1, 290_472_336),
        (5, false, 23, 12_629_232),
        (4, true, 325, 893_761),
        (4, false, 7_475, 38_859),
        (3, true, 20_800, 13_965),
        (3, false, 478_400, 607),
        (2, true, 436_800, 665),
        (1, true, 3_385_200, 86),
        (0, true, 8_259_888, 35),
    ];
    for (w, b, ways, published) in expect {
        assert_eq!(m.ways(w, b), ways, "ways {w}+{b}");
        assert_eq!(m.one_in_scaled(w, b, 0), published, "odds {w}+{b}");
    }
    assert_eq!(m.overall_scaled(0), 23);
}

#[test]
fn megamillions_2017_all_tiers() {
    let m = Matrix::new(5, 70, 25);
    assert_eq!(m.total(), 302_575_350);
    let published: [u128; 9] = [
        302_575_350,
        12_607_306,
        931_001,
        38_792,
        14_547,
        606,
        693,
        89,
        37,
    ];
    for (i, &(w, b)) in PRIZE_TIERS.iter().enumerate() {
        assert_eq!(m.one_in_scaled(w, b, 0), published[i], "odds {w}+{b}");
    }
    assert_eq!(m.overall_scaled(0), 24);
}

#[test]
fn ways_partition_the_outcome_space() {
    for (w, b) in [
        (69, 26),
        (70, 24),
        (70, 25),
        (75, 15),
        (56, 46),
        (59, 35),
        (59, 39),
        (52, 52),
    ] {
        let m = Matrix::new(5, w, b);
        let sum: u128 = (0..=5)
            .flat_map(|k| [m.ways(k, true), m.ways(k, false)])
            .sum();
        assert_eq!(sum, m.total(), "matrix {w}/{b}");
    }
}

#[test]
fn round_half_up() {
    assert_eq!(round_div(5, 2), 3);
    assert_eq!(round_div(4, 2), 2);
    assert_eq!(round_div(7, 3), 2);
}
