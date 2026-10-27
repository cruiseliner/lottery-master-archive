# Crosscheck log

Official winning-number crosschecks (`lottotool check --crosscheck N`).
R2-06 cadence trigger: **must exist before v0.2 grade ships** (Claude Phase 3.1 ack / A-02).

| Date (America/New_York) | Command | Powerball | Mega Millions | Notes |
|---|---|---|---|---|
| 2026-10-23 ~23:00 | `lottotool check --no-stale --crosscheck 10` | PASS 10/10 | PASS 10/10 | Latest PB draw in CSV 2026-10-21; latest MM 2026-10-20. Summary: 26 pass, 0 warn, 0 fail, 4 skip. Exit 0. |
| 2026-10-27 ~16:10 | `lottotool check --no-stale --crosscheck 3` | PASS 3/3 | PASS 3/3 | Latest PB draw in CSV 2026-10-26; latest MM 2026-10-23. Summary: 31 pass, 0 warn, 0 fail, 4 skip. Exit 0. A-02 / C-22. |

### 2026-10-23 ~23:00 America/New_York — detail

```
PASS  powerball    crosscheck                   10/10 latest draws match official winning numbers
PASS  megamillions crosscheck                   10/10 latest draws match official winning numbers
summary: 26 pass, 0 warn, 0 fail, 4 skip
```

Per-draw lists are not emitted by the tool on PASS (aggregate only). Latest 10 CSV dates covered:

- Powerball: 2026-09-28 .. 2026-10-21 (Wed/Sat then Mon/Wed/Sat era)
- Mega Millions: 2026-09-15 .. 2026-10-20 (Tue/Fri)

### 2026-10-27 ~16:10 America/New_York — detail (C-22 / A-02)

```
PASS  powerball    crosscheck                   3/3 latest draws match official winning numbers
PASS  megamillions crosscheck                   3/3 latest draws match official winning numbers
summary: 31 pass, 0 warn, 0 fail, 4 skip
```

Per-draw lists are not emitted by the tool on PASS (aggregate only). Latest 3 CSV dates covered:

- Powerball: 2026-10-21, 2026-10-24, 2026-10-26
- Mega Millions: 2026-10-16, 2026-10-20, 2026-10-23

**C-22 loose end (PB 2026-10-26 numbers Claude could not independently confirm):** archive row `18,26,49,50,54 / PB 1 / PP 3 / DP 5,11,14,24,59 / 8` matches Colorado Lottery draw page, Maryland Lottery detailed results (main + PB + 3x), and USA Today; tool crosscheck against powerball.com also PASS for that date (included in the 3/3).
