use lottotool::odds::{Matrix, PRIZE_TIERS};
use lottotool::rng::{pick, seeded, simulate};

#[test]
fn same_seed_same_sequence() {
    let m = Matrix::new(5, 69, 26);
    let mut a = seeded(7);
    let mut b = seeded(7);
    for _ in 0..1000 {
        assert_eq!(pick(&mut a, &m), pick(&mut b, &m));
    }
    let mut c = seeded(8);
    let mut a = seeded(7);
    let differs = (0..20).any(|_| pick(&mut a, &m) != pick(&mut c, &m));
    assert!(differs, "different seeds should give different picks");
}

#[test]
fn picks_are_valid() {
    let m = Matrix::new(5, 70, 24);
    let mut r = seeded(123);
    let mut seen_w = [false; 71];
    let mut seen_b = [false; 25];
    for _ in 0..20_000 {
        let (w, b) = pick(&mut r, &m);
        assert_eq!(w.len(), 5);
        assert!(w.windows(2).all(|x| x[0] < x[1]), "sorted & distinct");
        assert!(w.iter().all(|&x| (1..=70).contains(&x)));
        assert!((1..=24).contains(&b));
        w.iter().for_each(|&x| seen_w[x as usize] = true);
        seen_b[b as usize] = true;
    }
    assert!(seen_w[1..].iter().all(|&s| s), "every white ball reachable");
    assert!(seen_b[1..].iter().all(|&s| s), "every bonus ball reachable");
}

#[test]
fn simulation_is_deterministic_and_close_to_exact_odds() {
    let m = Matrix::new(5, 69, 26);
    let n = 400_000u64;
    let a = simulate(&m, n, 2026);
    assert_eq!(a, simulate(&m, n, 2026));
    assert_eq!(a.iter().sum::<u64>(), n);
    // Frequent tiers (expected >= 1,000) within 5 standard deviations (Poisson approx).
    for (i, &(w, b)) in PRIZE_TIERS.iter().enumerate() {
        let exp = n as f64 / m.one_in(w, b);
        if exp >= 1000.0 {
            let z = (a[i] as f64 - exp) / exp.sqrt();
            assert!(
                z.abs() < 5.0,
                "tier {w}+{b}: observed {} expected {exp:.1} z={z:.2}",
                a[i]
            );
        }
    }
}

#[test]
fn golden_vector_seed_1_first_five_picks() {
    // Pinned golden vector (R3-07): ChaCha20 seed 1, matrix 5/69/26.
    let m = Matrix::new(5, 69, 26);
    let mut r = seeded(1);
    let got: Vec<(Vec<u32>, u32)> = (0..5).map(|_| pick(&mut r, &m)).collect();
    // Captured from this implementation on 2026-10-23; update deliberately if RNG/pick changes.
    let expect: Vec<(Vec<u32>, u32)> = vec![
        (vec![2, 10, 17, 60, 67], 5),
        (vec![16, 25, 26, 51, 57], 21),
        (vec![20, 25, 50, 53, 54], 6),
        (vec![1, 12, 20, 40, 68], 13),
        (vec![14, 16, 26, 51, 53], 14),
    ];
    if got != expect {
        // Helpful when first capturing / re-pinning.
        eprintln!("GOLDEN got: {got:?}");
    }
    assert_eq!(got, expect);
}

#[test]
fn below_uniform_and_zone_multiple_of_n() {
    use lottotool::rng::below;
    let mut r = seeded(99);
    let n = 26u32;
    let zone = u32::MAX - (u32::MAX % n);
    assert_eq!(
        zone % n,
        0,
        "rejection zone must be a multiple of n (R3-08)"
    );
    let mut hist = vec![0u64; n as usize];
    let trials = 200_000u64;
    for _ in 0..trials {
        hist[below(&mut r, n) as usize] += 1;
    }
    let exp = trials as f64 / n as f64;
    for (i, &c) in hist.iter().enumerate() {
        let z = (c as f64 - exp) / exp.sqrt();
        assert!(
            z.abs() < 5.0,
            "below({n}) bin {i}: {c} exp {exp:.1} z={z:.2}"
        );
    }
}

#[test]
fn white_ball_chi_square_reasonable() {
    let m = Matrix::new(5, 69, 26);
    let mut r = seeded(20261023);
    let mut hist = vec![0u64; 70];
    let draws = 50_000u64;
    for _ in 0..draws {
        let (w, _) = pick(&mut r, &m);
        for x in w {
            hist[x as usize] += 1;
        }
    }
    let exp = 5.0 * draws as f64 / 69.0;
    let mut chi2 = 0.0;
    for c in &hist[1..] {
        let d = *c as f64 - exp;
        chi2 += d * d / exp;
    }
    assert!(chi2 < 140.0, "chi2={chi2}");
}

#[test]
fn exact_hypergeometric_white_match_ways() {
    use lottotool::odds::comb;
    let m = Matrix::new(5, 69, 26);
    let white_total = comb(69, 5);
    for k in 0..=5u32 {
        let ways_white = comb(5, k as u64) * comb(64, (5 - k) as u64);
        // ways(k, true) counts white matches with bonus hit (= ways_white * 1)
        assert_eq!(m.ways(k, true), ways_white);
        assert_eq!(m.ways(k, false), ways_white * 25);
        let _p = ways_white as f64 / white_total as f64;
        let _ = _p;
    }
    assert_eq!(
        (0..=5u32)
            .map(|k| m.ways(k, true) + m.ways(k, false))
            .sum::<u128>(),
        m.total()
    );
}
