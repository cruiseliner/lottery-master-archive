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
