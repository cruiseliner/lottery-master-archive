//! CLI-level tests (Claude t48u gate 1): run the real `lottotool` binary in a temp copy of the
//! repo (eras.toml, .gitignore, data/), as a git work tree so the gitignore guard is exercised.
//! FAKE tickets only; one synthetic 2030 draw is appended to the temp copy's draws.csv.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_lottotool")
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let to = dst.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &to);
        } else {
            std::fs::copy(e.path(), &to).unwrap();
        }
    }
}

fn temp_repo(name: &str) -> PathBuf {
    temp_copy(name, true)
}

fn temp_copy(name: &str, git: bool) -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = std::env::temp_dir().join(format!(
        "lottotool-cli-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::copy(src.join("eras.toml"), root.join("eras.toml")).unwrap();
    std::fs::copy(src.join(".gitignore"), root.join(".gitignore")).unwrap();
    copy_dir(&src.join("data"), &root.join("data"));
    if git && Command::new("git").arg("--version").output().is_ok() {
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(&root)
                .status()
                .unwrap()
                .success()
        );
    }
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

fn text(o: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

fn last_draw_date(root: &Path, game: &str) -> String {
    let csv = std::fs::read_to_string(root.join("data").join(game).join("draws.csv")).unwrap();
    csv.lines()
        .last()
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .to_string()
}

#[test]
fn ticket_add_refuses_an_archived_draw_from_data() {
    let root = temp_repo("backdate");
    for (game, short) in [("powerball", "pb"), ("megamillions", "mm")] {
        let date = last_draw_date(&root, game);
        let o = run(
            &root,
            &[
                "ticket",
                "add",
                "--game",
                short,
                "--draw-date",
                &date,
                "--whites",
                "1,2,3,4,5",
                "--special",
                "6",
            ],
        );
        let (out, err) = text(&o);
        assert!(!o.status.success(), "{game} {date}: accepted\n{out}\n{err}");
        assert!(err.contains("already in the archive"), "{game}: {err}");
    }
    assert!(
        !root.join("ledger/tickets/tickets.csv").exists(),
        "nothing written"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn grade_excludes_on_time_voids_counts_them_and_labels_baseline_as_history() {
    let root = temp_repo("grade");
    let add = |whites: &str| {
        let o = run(
            &root,
            &[
                "ticket",
                "add",
                "--game",
                "pb",
                "--draw-date",
                "2030-01-02",
                "--whites",
                whites,
                "--special",
                "6",
            ],
        );
        assert!(o.status.success(), "{:?}", text(&o));
    };
    add("1,2,3,4,5");
    add("10,20,3,4,5");
    // Voided now, years before the 2030-01-02 22:00 ET cutoff: an on-time void.
    let o = run(&root, &["ticket", "void", "T0001", "--reason", "fake test"]);
    assert!(o.status.success(), "{:?}", text(&o));
    let (out, _) = text(&run(&root, &["ticket", "list"]));
    assert!(out.contains("[VOID "), "{out}");
    assert!(out.contains("2 ticket(s), 1 voided"), "{out}");

    // Synthetic draw for 2030-01-02 appended to the temp copy only.
    let draws = root.join("data/powerball/draws.csv");
    let mut csv = std::fs::read_to_string(&draws).unwrap();
    csv.push_str("2030-01-02,pb-2015,10,20,30,40,50,26,2,,,,,,,test:fake\n");
    std::fs::write(&draws, csv).unwrap();

    let o = run(&root, &["grade", "--all", "--game", "pb"]);
    let (out, err) = text(&o);
    assert!(o.status.success(), "{out}\n{err}");
    assert!(
        !out.contains("T0001"),
        "voided ticket must not be graded:\n{out}"
    );
    assert!(out.contains("T0002 powerball 2030-01-02"), "{out}");
    assert!(out.contains("match  2 white (10 20)"), "{out}");
    assert!(
        out.contains("1 graded, 0 pending, 0 in a prize tier, 1 voided excluded, 0 LATE_VOID (kept), 0 LATE_ENTRY"),
        "{out}"
    );
    assert!(
        out.contains("powerball (all eligible graded tickets in the ledger): avg white matches 2.000 over n=1 ticket(s); chance baseline 25/69 = 0.362; chance spread ±0.562 (1 SD); too few tickets (<30) to read anything into; 0 LATE_ENTRY excluded"),
        "{out}"
    );
    assert!(out.contains("history of these tickets measured against chance; not a prediction"));

    let o = run(&root, &["--json", "grade", "--all", "--game", "pb"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["voided_excluded"], 1);
    assert_eq!(v["graded"], 1);
    assert_eq!(v["white_match_measurement"][0]["baseline_formula"], "25/69");
    assert_eq!(v["white_match_measurement"][0]["eligible_tickets"], 1);
    assert_eq!(v["white_match_measurement"][0]["too_few_tickets"], true);
    assert!(
        v["measurement_note"]
            .as_str()
            .unwrap()
            .contains("not a prediction")
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Remove the last row of a temp copy's draws.csv; returns (draw_date, removed line).
fn pop_last_draw(root: &Path, game: &str) -> (String, String) {
    let p = root.join("data").join(game).join("draws.csv");
    let csv = std::fs::read_to_string(&p).unwrap();
    let mut lines: Vec<&str> = csv.lines().collect();
    let last = lines.pop().unwrap().to_string();
    std::fs::write(&p, lines.join("\n") + "\n").unwrap();
    (last.split(',').next().unwrap().to_string(), last)
}

#[test]
fn late_entry_tickets_are_excluded_from_the_measurement() {
    let root = temp_repo("late-entry");
    // Pretend the latest PB draw is not archived yet: its 22:00 ET cutoff is in the past, so a
    // ticket added now is accepted but LATE_ENTRY (D-030).
    let (late_date, row) = pop_last_draw(&root, "powerball");
    let o = run(
        &root,
        &[
            "ticket",
            "add",
            "--game",
            "pb",
            "--draw-date",
            &late_date,
            "--whites",
            "1,2,3,4,5",
            "--special",
            "6",
        ],
    );
    assert!(o.status.success(), "{:?}", text(&o));
    // One on-time ticket for a synthetic 2030 draw.
    let o = run(
        &root,
        &[
            "ticket",
            "add",
            "--game",
            "pb",
            "--draw-date",
            "2030-01-02",
            "--whites",
            "10,20,3,4,5",
            "--special",
            "6",
        ],
    );
    assert!(o.status.success(), "{:?}", text(&o));
    // Archive both draws in the temp copy (restore the real row + one synthetic row).
    let p = root.join("data/powerball/draws.csv");
    let mut csv = std::fs::read_to_string(&p).unwrap();
    csv.push_str(&format!(
        "{row}\n2030-01-02,pb-2015,10,20,30,40,50,26,2,,,,,,,test:fake\n"
    ));
    std::fs::write(&p, csv).unwrap();

    let o = run(&root, &["grade", "--all", "--game", "pb"]);
    let (out, err) = text(&o);
    assert!(o.status.success(), "{out}\n{err}");
    assert!(
        out.contains(&format!("T0001 powerball {late_date}")),
        "{out}"
    );
    assert!(out.contains("[LATE_ENTRY]"), "{out}");
    assert!(out.contains("2 graded, 0 pending"), "{out}");
    assert!(out.contains("1 LATE_ENTRY"), "{out}");
    // Only the on-time ticket (2 matches) counts: avg 2.000 over n=1, one excluded.
    assert!(
        out.contains("avg white matches 2.000 over n=1 ticket(s)"),
        "late entry must not count toward the average:\n{out}"
    );
    assert!(out.contains("1 LATE_ENTRY excluded"), "{out}");

    let o = run(&root, &["--json", "grade", "--all", "--game", "pb"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let m = &v["white_match_measurement"][0];
    assert_eq!(m["eligible_tickets"], 1);
    assert_eq!(m["late_entry_excluded"], 1);
    assert_eq!(m["avg_white_matches"], 2.0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn outside_a_git_work_tree_ticket_add_warns_and_proceeds() {
    let root = temp_copy("no-git", false);
    let o = run(
        &root,
        &[
            "ticket",
            "add",
            "--game",
            "pb",
            "--draw-date",
            "2030-01-02",
            "--whites",
            "1,2,3,4,5",
            "--special",
            "6",
        ],
    );
    let (out, err) = text(&o);
    assert!(o.status.success(), "{out}\n{err}");
    assert!(
        err.contains("not inside a git work tree; gitignore guard not applicable"),
        "{err}"
    );
    assert!(root.join("ledger/tickets/tickets.csv").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn crowd_prints_breakdown_and_disclaimer_in_text_and_json() {
    let root = temp_copy("crowd", false);
    let o = run(
        &root,
        &[
            "crowd",
            "--game",
            "pb",
            "--whites",
            "1,2,3,4,5",
            "--special",
            "6",
        ],
    );
    let (out, err) = text(&o);
    assert!(o.status.success(), "{out}\n{err}");
    assert!(
        out.starts_with(
            "crowd score 67/100 (HIGH) for powerball whites 1 2 3 4 5 PB 6 (era pb-2015; raw 60/90"
        ),
        "{out}"
    );
    for id in [
        "birthdays",
        "sequence",
        "same_decade",
        "last_digit",
        "lucky_numbers",
        "low_sum",
        "past_winner",
    ] {
        assert!(out.contains(id), "{id}: {out}");
    }
    assert!(
        out.contains("does NOT make a ticket likelier to win"),
        "{out}"
    );
    assert!(
        out.contains("reference: a random pick typically scores 0"),
        "{out}"
    );

    let o = run(
        &root,
        &[
            "--json",
            "crowd",
            "--game",
            "mm",
            "--whites",
            "9,26,38,55,67",
        ],
    );
    let (out, err) = text(&o);
    assert!(o.status.success(), "{out}\n{err}");
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["crowd"]["score"], 0);
    assert_eq!(v["crowd"]["band"], "low");
    assert_eq!(v["crowd"]["raw_max"], 90);
    assert_eq!(v["crowd"]["calibration"]["combinations"], 12_103_014);
    assert!(v["crowd"]["reference"].as_str().unwrap().contains("95.9%"));
    assert!(v["crowd"]["past_winners"].as_array().unwrap().is_empty());
    assert_eq!(v["weights"], "judgment");
    assert!(
        v["crowd"]["disclaimer"]
            .as_str()
            .unwrap()
            .contains("exactly the same odds")
    );
    assert_eq!(v["crowd"]["factors"].as_array().unwrap().len(), 7);

    // Invalid input fails without output on stdout.
    let o = run(&root, &["crowd", "--game", "pb", "--whites", "1,2,3,4,70"]);
    assert!(!o.status.success());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn crowd_scores_a_ledger_ticket_by_id() {
    let root = temp_repo("crowd-ticket");
    let o = run(
        &root,
        &[
            "ticket",
            "add",
            "--game",
            "mm",
            "--draw-date",
            "2030-01-04",
            "--whites",
            "10,20,30,40,50",
            "--special",
            "7",
        ],
    );
    assert!(o.status.success(), "{:?}", text(&o));
    let o = run(&root, &["crowd", "--ticket", "T0001"]);
    let (out, err) = text(&o);
    assert!(o.status.success(), "{out}\n{err}");
    assert!(
        out.contains("crowd score 46/100 (MEDIUM) for megamillions whites 10 20 30 40 50 MB 7 (era mm-2025; raw 41/90"),
        "{out}"
    );
    assert!(
        out.contains("does NOT make a ticket likelier to win"),
        "{out}"
    );
    let o = run(&root, &["crowd", "--ticket", "T0099"]);
    assert!(!o.status.success());
    assert!(text(&o).1.contains("T0099 not in the ledger"));
    // --ticket conflicts with --whites.
    let o = run(
        &root,
        &["crowd", "--ticket", "T0001", "--whites", "1,2,3,4,5"],
    );
    assert!(!o.status.success());
    let _ = std::fs::remove_dir_all(&root);
}
