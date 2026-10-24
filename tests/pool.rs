//! R2-04: pool groups derived from game+ball+max; mixed maxima are refused.
use lottotool::eras::{ErasFile, eras_in_pool, pool_group_id};
use lottotool::summary::pooled_frequencies;
use std::path::PathBuf;

fn eras() -> ErasFile {
    ErasFile::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("eras.toml")).unwrap()
}

#[test]
fn eras_toml_groups_are_derived_from_max() {
    let e = eras();
    for g in &e.game {
        for era in &g.era {
            assert_eq!(
                era.white_group,
                pool_group_id(&g.id, "white", era.white_max),
                "{}",
                era.id
            );
            assert_eq!(
                era.bonus_group,
                pool_group_id(&g.id, "bonus", era.bonus_max),
                "{}",
                era.id
            );
        }
    }
}

#[test]
fn mm_2017_and_2025_share_white_pool_not_bonus() {
    let e = eras();
    let mm = e.game("megamillions").unwrap();
    let a = mm.era.iter().find(|x| x.id == "mm-2017").unwrap();
    let b = mm.era.iter().find(|x| x.id == "mm-2025").unwrap();
    assert_eq!(a.white_group, "mm-white-70");
    assert_eq!(a.white_group, b.white_group);
    assert_ne!(a.bonus_group, b.bonus_group);
    assert_eq!(a.bonus_group, "mm-bonus-25");
    assert_eq!(b.bonus_group, "mm-bonus-24");
}

#[test]
fn pb_2012_and_2015_do_not_share_pools() {
    let e = eras();
    let pb = e.game("powerball").unwrap();
    let a = pb.era.iter().find(|x| x.id == "pb-2012").unwrap();
    let b = pb.era.iter().find(|x| x.id == "pb-2015").unwrap();
    assert_ne!(a.white_group, b.white_group);
    assert_ne!(a.bonus_group, b.bonus_group);
}

#[test]
fn refuses_pool_with_differing_maxima() {
    let mut e = eras();
    let pb = e.game.iter_mut().find(|g| g.id == "powerball").unwrap();
    for era in pb.era.iter_mut() {
        era.white_group = "white".into(); // old default: 59 and 69 together
    }
    let pb = e.game("powerball").unwrap();
    let err = eras_in_pool(pb, "white", true).unwrap_err();
    assert!(err.contains("refuse to pool"), "{err}");
    let err2 = pooled_frequencies(pb, &[], "white", "pb-bonus-26").unwrap_err();
    assert!(err2.contains("refuse to pool"), "{err2}");
    // validate() also rejects such a file
    assert!(e.validate().is_err());
}
