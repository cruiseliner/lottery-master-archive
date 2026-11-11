//! Pins the `lottotool check` line format that `.github/workflows/crosscheck.yml` parses
//! (D-025). The `case` block is read from the workflow file itself and run in bash, so a change
//! to either the output wording or the patterns that drops a FAIL annotation fails this test.
use lottotool::check::{CheckResult, Status, format_line, format_summary};
use lottotool::crosscheck::summarize;
use std::path::Path;

fn workflow_case_block() -> String {
    let wf = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/crosscheck.yml"),
    )
    .unwrap();
    let start = wf
        .find("case \"$line\" in")
        .expect("case block in crosscheck.yml");
    let end = wf[start..].find("esac").expect("esac") + start + 4;
    wf[start..end].to_string()
}

/// Run the workflow's case block on one line; returns the annotation it prints ("" if none).
fn annotate(line: &str) -> String {
    let script = format!("line=\"$1\"\n{}\n", workflow_case_block());
    let out = std::process::Command::new("bash")
        .arg("-c")
        .arg(script)
        .arg("bash")
        .arg(line)
        .output()
        .expect("bash");
    String::from_utf8(out.stdout).unwrap()
}

fn errs(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| format!("2030-01-0{}: fake error", i + 1))
        .collect()
}

#[test]
fn mismatch_line_is_fail_and_annotated_as_error() {
    let c = summarize(
        "powerball",
        8,
        7,
        &["2030-01-02 ours=[..] official=[..]".into()],
        &[],
    );
    assert_eq!(c.status, Status::Fail);
    let line = format_line(&c);
    assert!(
        line.starts_with("FAIL  powerball    crosscheck "),
        "{line:?}"
    );
    assert!(
        annotate(&line).starts_with("::error title=Crosscheck"),
        "{line:?}"
    );
}

#[test]
fn zero_successful_comparisons_is_fail() {
    let c = summarize("megamillions", 8, 0, &[], &errs(8));
    assert_eq!(c.status, Status::Fail);
    let line = format_line(&c);
    assert!(
        line.starts_with("FAIL  megamillions crosscheck "),
        "{line:?}"
    );
    assert!(line.contains("0/8 compared"));
    assert!(annotate(&line).starts_with("::error title=Crosscheck"));
}

#[test]
fn partial_source_errors_are_warn() {
    let c = summarize("powerball", 8, 6, &[], &errs(2));
    assert_eq!(c.status, Status::Warn);
    let line = format_line(&c);
    assert!(
        line.starts_with("WARN  powerball    crosscheck "),
        "{line:?}"
    );
    assert!(annotate(&line).starts_with("::warning title=Crosscheck"));
}

#[test]
fn pass_line_has_no_annotation_and_summary_format_is_pinned() {
    let c = summarize("powerball", 8, 8, &[], &[]);
    assert_eq!(c.status, Status::Pass);
    let line = format_line(&c);
    assert_eq!(
        line,
        "PASS  powerball    crosscheck                   8/8 latest draws match official winning numbers"
    );
    assert_eq!(annotate(&line), "");
    let other = CheckResult {
        game: "powerball".into(),
        check: "draws.range".into(),
        status: Status::Fail,
        detail: "fake".into(),
    };
    assert!(annotate(&format_line(&other)).starts_with("::error title=Integrity FAIL"));
    assert_eq!(
        format_summary(&[c, other]),
        "summary: 1 pass, 0 warn, 1 fail, 0 skip"
    );
}

#[test]
fn zero_draws_is_never_a_vacuous_pass() {
    let c = summarize("powerball", 0, 0, &[], &[]);
    assert_eq!(c.status, Status::Fail);
    let line = format_line(&c);
    assert!(
        line.starts_with("FAIL  powerball    crosscheck "),
        "{line:?}"
    );
    assert!(line.contains("0/0 compared"), "{line:?}");
    assert!(annotate(&line).starts_with("::error title=Crosscheck"));
}
