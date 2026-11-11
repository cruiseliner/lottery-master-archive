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
    if Command::new("git").arg("--version").output().is_ok() {
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
        out.contains("powerball: avg white matches 2.000 over 1 graded ticket(s); chance baseline 25/69 = 0.362"),
        "{out}"
    );
    assert!(out.contains("history of these tickets measured against chance; not a prediction"));

    let o = run(&root, &["--json", "grade", "--all", "--game", "pb"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["voided_excluded"], 1);
    assert_eq!(v["graded"], 1);
    assert_eq!(v["white_match_measurement"][0]["baseline_formula"], "25/69");
    assert!(
        v["measurement_note"]
            .as_str()
            .unwrap()
            .contains("not a prediction")
    );
    let _ = std::fs::remove_dir_all(&root);
}
