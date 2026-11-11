//! v0.2 Step 1 (D-031) ticket ledger + grade tests. FAKE data only: synthetic draws dated in
//! 2030 and synthetic cache rows in temp dirs. No real tickets, picks or payout tables.
use chrono::{NaiveDate, TimeZone, Utc};
use lottotool::Game;
use lottotool::data::{Draw, PayoutRow};
use lottotool::eras::{ErasFile, GameDef};
use lottotool::tickets::{
    GradeStatus, Guard, NewTicket, TICKETS_HEADER, TIER_LOOKUP_NEEDED, TicketRow, VOIDS_HEADER,
    VoidRow, append as append_with, chance_white_baseline, chance_white_sd, grade as grade_with,
    guard_status, guard_status_with, is_late_void, ledger_path, lookup_prize, read_ledger,
    read_voids, validate, void, voids_path,
};
use std::path::{Path, PathBuf};

/// Append with an empty archive (no draw for the ticket date yet).
fn append(
    root: &Path,
    g: &GameDef,
    t: &NewTicket,
    now: chrono::DateTime<Utc>,
) -> anyhow::Result<TicketRow> {
    append_with(root, g, t, &[], now)
}

/// Grade without a void record.
fn grade(
    _g: &GameDef,
    t: &TicketRow,
    draws: &[Draw],
    cache: &[PayoutRow],
) -> anyhow::Result<lottotool::tickets::Grade> {
    grade_with(t, draws, cache, None)
}

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

fn eras() -> ErasFile {
    ErasFile::load(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("eras.toml")
            .as_path(),
    )
    .unwrap()
}

fn game(id: &str) -> GameDef {
    eras().game(id).unwrap().clone()
}

// 2030-01-02 is a Wednesday (PB), 2030-01-01 a Tuesday (MM).
const PB_DAY: &str = "2030-01-02";
const MM_DAY: &str = "2030-01-01";

fn pb(whites: &[u32], special: u32, pp: bool) -> NewTicket {
    NewTicket {
        game: Game::Powerball,
        draw_date: d(PB_DAY),
        whites: whites.to_vec(),
        special,
        power_play: pp,
        cost_usd: None,
        note: "fake".into(),
    }
}

fn tmp_root(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "lottotool-tickets-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn fake_draw(date: &str, era: &str, w: [u32; 5], bonus: u32, mult: Option<u32>) -> Draw {
    Draw {
        draw_date: d(date),
        era_id: era.into(),
        w1: w[0],
        w2: w[1],
        w3: w[2],
        w4: w[3],
        w5: w[4],
        bonus,
        multiplier: mult,
        dp_w1: None,
        dp_w2: None,
        dp_w3: None,
        dp_w4: None,
        dp_w5: None,
        dp_bonus: None,
        source: "test:fake".into(),
    }
}

fn fake_cache(product: &str, white: u32, bonus: bool, usd: Option<u64>) -> PayoutRow {
    PayoutRow {
        game: "powerball".into(),
        draw_date: d(PB_DAY),
        scope: "national".into(),
        product: product.into(),
        tier: format!("{white}+{}", u8::from(bonus)),
        white_matched: white,
        bonus_matched: bonus,
        winners: Some(1),
        prize_usd: usd,
        prize_text: usd.map(|u| format!("${u}")).unwrap_or("Grand Prize".into()),
        source_url: "test:fake".into(),
        fetched_at: "2030-01-03T00:00:00Z".into(),
    }
}

fn before_draw() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 2, 15, 0, 0).unwrap()
}

// ------------------------------------------------------------------ validation

#[test]
fn valid_tickets_priced_from_eras_toml() {
    let g = game("powerball");
    assert_eq!(
        validate(&g, &pb(&[1, 2, 3, 4, 69], 26, false)).unwrap(),
        ("pb-2015".into(), 2)
    );
    assert_eq!(validate(&g, &pb(&[1, 2, 3, 4, 69], 26, true)).unwrap().1, 3);
    let mm = game("megamillions");
    let t = NewTicket {
        game: Game::Megamillions,
        draw_date: d(MM_DAY),
        whites: vec![10, 20, 30, 40, 70],
        special: 24,
        power_play: false,
        cost_usd: Some(5),
        note: String::new(),
    };
    assert_eq!(validate(&mm, &t).unwrap(), ("mm-2025".into(), 5));
}

#[test]
fn validation_rejects_bad_input() {
    let g = game("powerball");
    let err = |t: NewTicket| validate(&g, &t).unwrap_err().to_string();
    assert!(err(pb(&[0, 2, 3, 4, 5], 1, false)).contains("out of range"));
    assert!(err(pb(&[1, 2, 3, 4, 70], 1, false)).contains("out of range"));
    assert!(err(pb(&[1, 2, 3, 4, 5], 27, false)).contains("special ball 27"));
    assert!(err(pb(&[1, 2, 3, 3, 5], 1, false)).contains("duplicate"));
    assert!(err(pb(&[1, 2, 3, 4], 1, false)).contains("exactly 5"));
    let mut t = pb(&[1, 2, 3, 4, 5], 1, false);
    t.draw_date = d("2030-01-03"); // Thursday
    assert!(err(t).contains("not a scheduled"));
    let mut t = pb(&[1, 2, 3, 4, 5], 1, true);
    t.cost_usd = Some(2);
    assert!(err(t).contains("does not match"));
    let mut t = pb(&[1, 2, 3, 4, 5], 1, false);
    t.draw_date = d("2014-01-04"); // Saturday in inferred pb-2012 era
    assert!(err(t).contains("encoded prize chart"));
    let mut t = pb(&[1, 2, 3, 4, 5], 1, false);
    t.note = "a\nb".into();
    assert!(err(t).contains("single line"));

    let mm = game("megamillions");
    let t = NewTicket {
        game: Game::Megamillions,
        draw_date: d(MM_DAY),
        whites: vec![1, 2, 3, 4, 5],
        special: 25, // mm-2025 bonus_max = 24
        power_play: false,
        cost_usd: None,
        note: String::new(),
    };
    assert!(
        validate(&mm, &t)
            .unwrap_err()
            .to_string()
            .contains("special ball 25")
    );
    let t = NewTicket {
        special: 1,
        power_play: true,
        ..t
    };
    assert!(
        validate(&mm, &t)
            .unwrap_err()
            .to_string()
            .contains("only to Powerball")
    );
}

// ------------------------------------------------------------------ append-only ledger

#[test]
fn append_only_never_rewrites_existing_bytes() {
    let root = tmp_root("append");
    let g = game("powerball");
    let a = append(&root, &g, &pb(&[5, 4, 3, 2, 1], 7, false), before_draw()).unwrap();
    assert_eq!(a.ticket_id, "T0001");
    assert_eq!(a.whites(), [1, 2, 3, 4, 5], "whites stored sorted");
    assert_eq!(
        a.created_at, "2030-01-02T15:00:00Z",
        "created_at is UTC with Z"
    );
    let first = std::fs::read(ledger_path(&root)).unwrap();
    assert!(String::from_utf8_lossy(&first).starts_with(TICKETS_HEADER));

    let b = append(
        &root,
        &g,
        &pb(&[10, 20, 30, 40, 50], 8, true),
        before_draw(),
    )
    .unwrap();
    assert_eq!(b.ticket_id, "T0002");
    let second = std::fs::read(ledger_path(&root)).unwrap();
    assert!(second.len() > first.len());
    assert_eq!(
        &second[..first.len()],
        &first[..],
        "existing rows must be byte-identical"
    );
    assert_eq!(read_ledger(&root).unwrap(), vec![a, b]);

    // Invalid ticket: nothing appended.
    assert!(append(&root, &g, &pb(&[1, 1, 2, 3, 4], 1, false), before_draw()).is_err());
    assert_eq!(std::fs::read(ledger_path(&root)).unwrap(), second);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn corrupt_ledger_header_fails_closed_without_writing() {
    let root = tmp_root("corrupt");
    let p = ledger_path(&root);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, "id,numbers\nx,y\n").unwrap();
    assert!(read_ledger(&root).is_err());
    let g = game("powerball");
    assert!(append(&root, &g, &pb(&[1, 2, 3, 4, 5], 1, false), before_draw()).is_err());
    assert_eq!(std::fs::read_to_string(&p).unwrap(), "id,numbers\nx,y\n");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn refuses_to_write_when_ledger_not_gitignored() {
    let git_ok = std::process::Command::new("git")
        .arg("--version")
        .output()
        .is_ok();
    if !git_ok {
        return;
    }
    let root = tmp_root("gitguard");
    assert!(
        std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .unwrap()
            .success()
    );
    let g = game("powerball");
    let e = append(&root, &g, &pb(&[1, 2, 3, 4, 5], 1, false), before_draw()).unwrap_err();
    assert!(e.to_string().contains("not gitignored"), "{e}");
    assert!(!ledger_path(&root).exists());
    std::fs::write(root.join(".gitignore"), "/ledger/\n").unwrap();
    append(&root, &g, &pb(&[1, 2, 3, 4, 5], 1, false), before_draw()).unwrap();
    let _ = std::fs::remove_dir_all(&root);
}

// ------------------------------------------------------------------ grading

fn ticket_row(whites: &[u32], special: u32, pp: bool) -> lottotool::tickets::TicketRow {
    let root = tmp_root("row");
    let r = append(
        &root,
        &game("powerball"),
        &pb(whites, special, pp),
        before_draw(),
    )
    .unwrap();
    let _ = std::fs::remove_dir_all(&root);
    r
}

#[test]
fn pending_when_draw_not_ingested() {
    let g = game("powerball");
    let t = ticket_row(&[1, 2, 3, 4, 5], 6, false);
    let other = fake_draw("2029-12-31", "pb-2015", [1, 2, 3, 4, 5], 6, Some(2));
    let gr = grade(&g, &t, &[other], &[]).unwrap();
    assert_eq!(gr.status, GradeStatus::Pending);
    assert!(gr.tier.is_none() && !gr.prize_tier && gr.prize_usd.is_none());
    assert!(!gr.late_entry);
}

#[test]
fn special_ball_in_whites_edge_case() {
    let g = game("powerball");
    // Ticket special 10 equals a drawn WHITE; ticket white 26 equals the drawn PB.
    let t = ticket_row(&[1, 2, 3, 26, 60], 10, false);
    let draw = fake_draw(PB_DAY, "pb-2015", [10, 20, 30, 40, 50], 26, Some(3));
    let gr = grade(&g, &t, &[draw], &[]).unwrap();
    assert_eq!(gr.white_matches, Vec::<u32>::new());
    assert!(!gr.special_match);
    assert_eq!(gr.tier.as_deref(), Some("0+0"));
    assert!(!gr.prize_tier);
    assert_eq!(gr.prize, "NO_PRIZE");
}

#[test]
fn prize_tiers_and_amount_sources() {
    let g = game("powerball");
    let draw = fake_draw(PB_DAY, "pb-2015", [10, 20, 30, 40, 50], 26, Some(3));
    let t = ticket_row(&[10, 20, 30, 1, 2], 26, false); // 3+1
    // No cache: tier known, amount TIER_LOOKUP_NEEDED (never invented).
    let gr = grade(&g, &t, std::slice::from_ref(&draw), &[]).unwrap();
    assert_eq!(gr.tier.as_deref(), Some("3+1"));
    assert!(gr.prize_tier);
    assert!(gr.prize.starts_with(TIER_LOOKUP_NEEDED), "{}", gr.prize);
    assert!(gr.prize_usd.is_none());
    assert!(gr.multiplier_note.is_none(), "no PP on ticket");

    // Fake cache: base row used for non-PP ticket, power_play row for PP ticket.
    let cache = vec![
        fake_cache("base", 3, true, Some(111)),
        fake_cache("power_play", 3, true, Some(333)),
        fake_cache("base", 5, true, None),
    ];
    let gr = grade(&g, &t, std::slice::from_ref(&draw), &cache).unwrap();
    assert_eq!(gr.prize_usd, Some(111));
    let tpp = ticket_row(&[10, 20, 30, 1, 2], 26, true);
    let gr = grade(&g, &tpp, std::slice::from_ref(&draw), &cache).unwrap();
    assert_eq!(gr.prize_usd, Some(333));
    assert_eq!(gr.drawn_multiplier, Some(3));
    assert_eq!(
        gr.multiplier_note.as_deref(),
        Some("Power Play 3x (archive)")
    );

    // Jackpot row with text only -> lookup needed.
    let jp = ticket_row(&[10, 20, 30, 40, 50], 26, false);
    let gr = grade(&g, &jp, std::slice::from_ref(&draw), &cache).unwrap();
    assert_eq!(gr.tier.as_deref(), Some("5+1"));
    assert!(gr.prize.starts_with(TIER_LOOKUP_NEEDED));

    // 2 whites without PB is not a prize tier on the current chart.
    let t2 = ticket_row(&[10, 20, 1, 2, 3], 1, false);
    let gr = grade(&g, &t2, std::slice::from_ref(&draw), &cache).unwrap();
    assert_eq!(gr.tier.as_deref(), Some("2+0"));
    assert!(!gr.prize_tier);

    // 0+1 is a prize tier.
    let t0 = ticket_row(&[1, 2, 3, 4, 5], 26, false);
    assert!(
        grade(&g, &t0, std::slice::from_ref(&draw), &[])
            .unwrap()
            .prize_tier
    );

    // PP ticket on a draw whose multiplier is missing in the archive.
    let no_mult = fake_draw(PB_DAY, "pb-2015", [10, 20, 30, 40, 50], 26, None);
    let gr = grade(&g, &tpp, &[no_mult], &[]).unwrap();
    assert!(gr.multiplier_note.unwrap().contains("MISSING"));
}

#[test]
fn ambiguous_cache_amount_is_lookup_needed() {
    let cache = vec![
        fake_cache("base", 4, false, Some(100)),
        fake_cache("base", 4, false, Some(200)),
    ];
    let (usd, txt) = lookup_prize(&cache, Game::Powerball, 4, false, false);
    assert!(usd.is_none());
    assert!(txt.starts_with(TIER_LOOKUP_NEEDED));
}

#[test]
fn late_entry_is_flagged_after_2200_et_cutoff() {
    let g = game("powerball");
    let root = tmp_root("late");
    // 2030-01-03 03:00Z = 2030-01-02 22:00 EST: exactly the D-030 cutoff -> LATE_ENTRY.
    let at_cutoff = Utc.with_ymd_and_hms(2030, 1, 3, 3, 0, 0).unwrap();
    let t = append(&root, &g, &pb(&[1, 2, 3, 4, 5], 6, false), at_cutoff).unwrap();
    assert!(grade(&g, &t, &[], &[]).unwrap().late_entry);
    // One second earlier is on time.
    let on_time = Utc.with_ymd_and_hms(2030, 1, 3, 2, 59, 59).unwrap();
    let t2 = append(&root, &g, &pb(&[1, 2, 3, 4, 6], 6, false), on_time).unwrap();
    assert!(!grade(&g, &t2, &[], &[]).unwrap().late_entry);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn back_dated_ticket_for_archived_draw_is_refused() {
    let g = game("powerball");
    let root = tmp_root("backdate");
    let draw = fake_draw(PB_DAY, "pb-2015", [10, 20, 30, 40, 50], 26, Some(2));
    let e = append_with(
        &root,
        &g,
        &pb(&[10, 20, 30, 40, 50], 26, false),
        &[draw],
        before_draw(),
    )
    .unwrap_err();
    assert!(e.to_string().contains("already in the archive"), "{e}");
    assert!(!ledger_path(&root).exists(), "nothing written");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn partial_last_line_refuses_append_for_tickets_and_voids() {
    let g = game("powerball");
    let root = tmp_root("partial");
    append(&root, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    // Simulate a crash mid-write: valid prefix plus a partial row with no trailing newline.
    let mut bytes = std::fs::read(ledger_path(&root)).unwrap();
    bytes.extend_from_slice(b"T0002,2030-01-02T15:00:00Z,powerball,2030-01-02,pb-2015,1,2");
    std::fs::write(ledger_path(&root), &bytes).unwrap();
    let e = append(&root, &g, &pb(&[1, 2, 3, 4, 7], 6, false), before_draw()).unwrap_err();
    assert!(format!("{e:#}").contains("newline"), "{e:#}");
    assert_eq!(
        std::fs::read(ledger_path(&root)).unwrap(),
        bytes,
        "tickets file untouched"
    );

    let root2 = tmp_root("partial-void");
    append(&root2, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    append(&root2, &g, &pb(&[1, 2, 3, 4, 6], 6, false), before_draw()).unwrap();
    let partial = format!("{VOIDS_HEADER}\nT0001,2030-01-02T15:00:00Z,typo");
    std::fs::write(voids_path(&root2), &partial).unwrap();
    let e = void(&root2, "T0002", "typo", before_draw()).unwrap_err();
    assert!(format!("{e:#}").contains("newline"), "{e:#}");
    assert_eq!(
        std::fs::read_to_string(voids_path(&root2)).unwrap(),
        partial
    );
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&root2);
}

#[test]
fn late_void_is_flagged_and_still_graded() {
    let g = game("powerball");
    let root = tmp_root("latevoid");
    let t = append(&root, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    // Voided the morning after the draw (allowed, but LATE_VOID).
    let after = Utc.with_ymd_and_hms(2030, 1, 3, 14, 0, 0).unwrap();
    let v = void(&root, "T0001", "changed my mind", after).unwrap();
    assert!(is_late_void(&t, &v).unwrap());
    let draw = fake_draw(PB_DAY, "pb-2015", [10, 20, 30, 40, 50], 26, Some(2));
    let gr = grade_with(&t, &[draw], &[], Some(&v)).unwrap();
    assert!(gr.late_void);
    assert_eq!(gr.status, GradeStatus::Graded);
    assert_eq!(gr.void.as_ref(), Some(&v));
    // An on-time void (before the cutoff) is not late.
    let early = VoidRow {
        voided_at: "2030-01-02T16:00:00Z".into(),
        ..v
    };
    assert!(!is_late_void(&t, &early).unwrap());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn chance_baselines_are_k_squared_over_n() {
    let pb = game("powerball");
    let mm = game("megamillions");
    let b = chance_white_baseline(pb.current_era());
    assert!((b - 25.0 / 69.0).abs() < 1e-12 && format!("{b:.3}") == "0.362");
    let b = chance_white_baseline(mm.current_era());
    assert!((b - 25.0 / 70.0).abs() < 1e-12 && format!("{b:.3}") == "0.357");
}

// ------------------------------------------------------------------ voids

#[test]
fn void_appends_record_and_never_mutates_ticket_rows() {
    let root = tmp_root("void");
    let g = game("powerball");
    let a = append(&root, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    let ledger_before = std::fs::read(ledger_path(&root)).unwrap();

    let v = void(&root, "T0001", "typo in white ball", before_draw()).unwrap();
    assert_eq!(v.ticket_id, "T0001");
    assert_eq!(v.voided_at, "2030-01-02T15:00:00Z");
    assert_eq!(
        std::fs::read(ledger_path(&root)).unwrap(),
        ledger_before,
        "ticket rows untouched"
    );
    let vtext = std::fs::read_to_string(voids_path(&root)).unwrap();
    assert!(vtext.starts_with(VOIDS_HEADER));
    assert!(
        voids_path(&root)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("tickets")
    );

    // Corrected ticket is a new row with a new id; original row still present and unchanged.
    let b = append(&root, &g, &pb(&[1, 2, 3, 4, 6], 6, false), before_draw()).unwrap();
    assert_eq!(b.ticket_id, "T0002");
    let after = std::fs::read(ledger_path(&root)).unwrap();
    assert_eq!(&after[..ledger_before.len()], &ledger_before[..]);
    assert_eq!(read_ledger(&root).unwrap(), vec![a, b]);
    assert_eq!(read_voids(&root).unwrap(), vec![v]);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn void_unknown_id_fails_and_writes_nothing() {
    let root = tmp_root("void-unknown");
    let g = game("powerball");
    append(&root, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    let e = void(&root, "T0099", "nope", before_draw()).unwrap_err();
    assert!(e.to_string().contains("no ticket T0099"), "{e}");
    assert!(!voids_path(&root).exists());
    // Empty reason refused too.
    assert!(void(&root, "T0001", "  ", before_draw()).is_err());
    assert!(!voids_path(&root).exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn double_void_is_refused_and_log_unchanged() {
    let root = tmp_root("void-double");
    let g = game("powerball");
    append(&root, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    void(&root, "T0001", "first", before_draw()).unwrap();
    let before = std::fs::read(voids_path(&root)).unwrap();
    let e = void(&root, "T0001", "second", before_draw()).unwrap_err();
    assert!(e.to_string().contains("already voided"), "{e}");
    assert_eq!(std::fs::read(voids_path(&root)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn hand_edited_void_log_fails_closed() {
    let root = tmp_root("void-corrupt");
    let g = game("powerball");
    append(&root, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    std::fs::write(
        voids_path(&root),
        format!("{VOIDS_HEADER}\nT0042,2030-01-02T15:00:00Z,ghost\n"),
    )
    .unwrap();
    assert!(
        read_voids(&root)
            .unwrap_err()
            .to_string()
            .contains("unknown ticket")
    );
    std::fs::write(
        voids_path(&root),
        format!("{VOIDS_HEADER}\nT0001,2030-01-02T15:00:00Z,a\nT0001,2030-01-02T15:00:01Z,b\n"),
    )
    .unwrap();
    assert!(
        read_voids(&root)
            .unwrap_err()
            .to_string()
            .contains("more than once")
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ------------------------------------------------------------------ round 2 (Claude t48u)

#[test]
fn zero_byte_ledger_files_are_treated_as_missing() {
    let root = tmp_root("empty");
    std::fs::create_dir_all(ledger_path(&root).parent().unwrap()).unwrap();
    std::fs::write(ledger_path(&root), b"").unwrap();
    std::fs::write(voids_path(&root), b"").unwrap();
    assert!(read_ledger(&root).unwrap().is_empty());
    assert!(read_voids(&root).unwrap().is_empty());
    let g = game("powerball");
    let t = append(&root, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    assert_eq!(t.ticket_id, "T0001");
    assert!(
        std::fs::read_to_string(ledger_path(&root))
            .unwrap()
            .starts_with(TICKETS_HEADER)
    );
    void(&root, "T0001", "fake", before_draw()).unwrap();
    assert!(
        std::fs::read_to_string(voids_path(&root))
            .unwrap()
            .starts_with(VOIDS_HEADER)
    );
    assert_eq!(read_voids(&root).unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn duplicate_ticket_ids_fail_closed() {
    let root = tmp_root("dupid");
    let g = game("powerball");
    append(&root, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    let text = std::fs::read_to_string(ledger_path(&root)).unwrap();
    let row = text.lines().nth(1).unwrap().to_string();
    let doubled = format!("{text}{row}\n");
    std::fs::write(ledger_path(&root), &doubled).unwrap();
    let e = read_ledger(&root).unwrap_err();
    assert!(e.to_string().contains("duplicate ticket_id T0001"), "{e}");
    assert!(append(&root, &g, &pb(&[1, 2, 3, 4, 6], 6, false), before_draw()).is_err());
    assert!(void(&root, "T0001", "x", before_draw()).is_err());
    assert_eq!(
        std::fs::read_to_string(ledger_path(&root)).unwrap(),
        doubled
    );
    assert!(!voids_path(&root).exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn late_entry_cutoff_in_edt_summer() {
    // 2030-07-03 is a Wednesday (PB). 22:00 EDT (UTC-4) = 2030-07-04 02:00Z.
    let g = game("powerball");
    let root = tmp_root("edt");
    let mut t = pb(&[1, 2, 3, 4, 5], 6, false);
    t.draw_date = d("2030-07-03");
    let at = Utc.with_ymd_and_hms(2030, 7, 4, 2, 0, 0).unwrap();
    let late = append(&root, &g, &t, at).unwrap();
    assert!(grade(&g, &late, &[], &[]).unwrap().late_entry);
    let mut t2 = t.clone();
    t2.whites = vec![1, 2, 3, 4, 6];
    // 01:59:59Z = 21:59:59 EDT: on time (an EST reading would put the cutoff at 03:00Z).
    let ok = append(
        &root,
        &g,
        &t2,
        Utc.with_ymd_and_hms(2030, 7, 4, 1, 59, 59).unwrap(),
    )
    .unwrap();
    assert!(!grade(&g, &ok, &[], &[]).unwrap().late_entry);
    // 02:30Z would be on time under EST; under EDT it is late.
    let mut t3 = t.clone();
    t3.whites = vec![1, 2, 3, 4, 7];
    let mid = append(
        &root,
        &g,
        &t3,
        Utc.with_ymd_and_hms(2030, 7, 4, 2, 30, 0).unwrap(),
    )
    .unwrap();
    assert!(grade(&g, &mid, &[], &[]).unwrap().late_entry);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn git_guard_checks_from_a_subdirectory_root() {
    if std::process::Command::new("git")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let top = tmp_root("guard-sub");
    assert!(
        std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&top)
            .status()
            .unwrap()
            .success()
    );
    std::fs::write(top.join(".gitignore"), "/ledger/\n").unwrap();
    let sub = top.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let g = game("powerball");
    // --root = sub: no sub/.git, but the work tree is found and sub/ledger/ is NOT ignored.
    let e = append(&sub, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap_err();
    assert!(e.to_string().contains("not gitignored"), "{e}");
    assert!(!ledger_path(&sub).exists());
    assert!(matches!(
        guard_status(&sub, &ledger_path(&sub)).unwrap(),
        Guard::NotIgnored(_)
    ));
    // Repo root works; outside any repo the status is NotInRepo (warned, not silent).
    append(&top, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    assert_eq!(
        guard_status(&top, &ledger_path(&top)).unwrap(),
        Guard::Ignored
    );
    let outside = tmp_root("guard-outside");
    assert_eq!(
        guard_status(&outside, &ledger_path(&outside)).unwrap(),
        Guard::NotInRepo
    );
    let _ = std::fs::remove_dir_all(&top);
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn mixed_amount_and_text_rows_get_accurate_message() {
    let cache = vec![
        fake_cache("base", 4, false, Some(100)),
        fake_cache("base", 4, false, None),
    ];
    let (usd, txt) = lookup_prize(&cache, Game::Powerball, 4, false, false);
    assert!(usd.is_none());
    assert!(txt.starts_with(TIER_LOOKUP_NEEDED));
    assert!(txt.contains("mix dollar amounts and text-only"), "{txt}");
    assert!(!txt.contains("more than one amount"));
}

// ------------------------------------------------------------------ round 3 (Claude t49u)

#[test]
fn git_failure_refuses() {
    let root = tmp_root("git-missing");
    let p = ledger_path(&root);
    // git cannot be spawned at all.
    let e = guard_status_with("/nonexistent/lottotool-no-git", &root, &p).unwrap_err();
    assert!(e.to_string().contains("refusing"), "{e}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // git runs but fails with something other than "not a git repository".
        let fake = root.join("fake-git");
        std::fs::write(
            &fake,
            "#!/bin/sh\necho 'fatal: detected dubious ownership' >&2\nexit 128\n",
        )
        .unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        let e = guard_status_with(fake.to_str().unwrap(), &root, &p).unwrap_err();
        assert!(e.to_string().contains("refusing"), "{e}");
        // check-ignore fatal (exit 128) after a good rev-parse also refuses.
        let fake2 = root.join("fake-git2");
        std::fs::write(
            &fake2,
            format!(
                "#!/bin/sh\nif [ \"$3\" = rev-parse ]; then echo '{}'; exit 0; fi\nexit 128\n",
                root.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&fake2, std::fs::Permissions::from_mode(0o755)).unwrap();
        let e = guard_status_with(fake2.to_str().unwrap(), &root, &p).unwrap_err();
        assert!(e.to_string().contains("check-ignore failed"), "{e}");
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn tracked_ledger_is_refused_even_if_gitignored() {
    if std::process::Command::new("git")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let top = tmp_root("tracked");
    let git = |args: &[&str]| {
        assert!(
            std::process::Command::new("git")
                .args(args)
                .current_dir(&top)
                .status()
                .unwrap()
                .success()
        )
    };
    git(&["init", "-q"]);
    std::fs::write(top.join(".gitignore"), "/ledger/\n").unwrap();
    let g = game("powerball");
    append(&top, &g, &pb(&[1, 2, 3, 4, 5], 6, false), before_draw()).unwrap();
    // Force-track the private file: from now on the guard must refuse every write.
    git(&["add", "-f", "ledger/tickets/tickets.csv"]);
    let before = std::fs::read(ledger_path(&top)).unwrap();
    assert!(matches!(
        guard_status(&top, &ledger_path(&top)).unwrap(),
        Guard::NotIgnored(_)
    ));
    let e = append(&top, &g, &pb(&[1, 2, 3, 4, 6], 6, false), before_draw()).unwrap_err();
    assert!(e.to_string().contains("not gitignored"), "{e}");
    assert_eq!(std::fs::read(ledger_path(&top)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&top);
}

#[test]
fn chance_spread_per_ticket() {
    let pb = game("powerball");
    let mm = game("megamillions");
    assert_eq!(format!("{:.3}", chance_white_sd(pb.current_era())), "0.562");
    assert_eq!(format!("{:.3}", chance_white_sd(mm.current_era())), "0.559");
}
