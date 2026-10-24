//! Parser tests on SYNTHETIC page fragments (structure mirrors the live pages; values are
//! test inputs, not real results). Real payout tables live only in the gitignored cache/.
use chrono::NaiveDate;
use lottotool::payouts::{dotnet_ticks, parse_megamillions, parse_powerball_html, powerball_url};

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

#[test]
fn ticks_match_megamillions_api() {
    // PlayDateTicks returned by the megamillions.com API for the 2017-10-31 drawing.
    assert_eq!(dotnet_ticks(d("2017-10-31")), 636_450_048_000_000_000);
    // ... and for 2026-10-20 (GetLatestDrawData, 2026-10-23).
    assert_eq!(dotnet_ticks(d("2026-10-20")), 639_280_512_000_000_000);
}

#[test]
fn powerball_urls() {
    assert!(lottotool::payouts::validate_state("fl").is_ok());
    assert!(lottotool::payouts::validate_state("FL").is_err());
    assert!(lottotool::payouts::validate_state("florida").is_err());
    assert_eq!(
        powerball_url(d("2026-10-21"), false, Some("fl")).unwrap(),
        "https://www.powerball.com/draw-result?gc=powerball&date=2026-10-21&oc=fl"
    );
    assert_eq!(
        powerball_url(d("2026-10-21"), true, None).unwrap(),
        "https://www.powerball.com/draw-result?gc=pb-double-play&date=2026-10-21"
    );
    assert!(powerball_url(d("2026-10-21"), false, Some("FL")).is_err());
}

const PB_HTML: &str = r#"
<table class="winners-table | table"><thead></thead>
<tbody>
<tr><td class="match-col" data-label="Match"><div class="game-balls float-sm-end w-auto px-0 item-powerball m5-pb"></div></td>
<td class="x" data-label="Powerball Winners">
 0
</td><td class="x" data-label="Powerball Prize"> Grand Prize </td>
<td class="x" data-label="Power Play Winners"> </td><td class="x" data-label="Power Play Prize"></td></tr>
<tr><td class="match-col" data-label="Match"><div class="game-balls float-sm-end w-auto px-0 item-powerball m3"></div></td>
<td class="x" data-label="Powerball Winners">1,234</td><td class="x" data-label="Powerball Prize">$7</td>
<td class="x" data-label="Power Play Winners">56</td><td class="x" data-label="Power Play Prize">$14</td></tr>
</tbody></table>"#;

#[test]
fn parse_powerball_table() {
    let rows = parse_powerball_html(PB_HTML, d("2026-10-21"), "fl", false, "u", "t").unwrap();
    assert_eq!(rows.len(), 3, "{rows:#?}"); // jackpot base only + 3+0 base + 3+0 power play
    assert_eq!(rows[0].tier, "5+1");
    assert_eq!(rows[0].winners, Some(0));
    assert_eq!(rows[0].prize_usd, None);
    assert_eq!(rows[1].tier, "3+0");
    assert_eq!(rows[1].winners, Some(1234));
    assert_eq!(rows[1].prize_usd, Some(7));
    assert_eq!(rows[2].product, "power_play");
    assert_eq!(rows[2].prize_usd, Some(14));
    assert!(rows.iter().all(|r| r.scope == "fl"));
}

#[test]
fn parse_powerball_rejects_layout_change() {
    assert!(
        parse_powerball_html(
            "<html>nothing</html>",
            d("2026-10-21"),
            "national",
            false,
            "u",
            "t"
        )
        .is_err()
    );
}

#[test]
fn parse_megamillions_json() {
    let inner = serde_json::json!({
        "Drawing": {"PlayDate": "2026-10-20T00:00:00", "Megaplier": -1},
        "PrizeTiers": [
            {"Tier": 0, "IsMegaplier": false, "Winners": 0, "Multiplier": ""},
            {"Tier": 1, "IsMegaplier": false, "Winners": 2, "Multiplier": "3x"}
        ],
        "PrizeMatrix": {"PrizeTiers": [
            {"PrizeTier": 0, "TierWhiteBall": 5, "TierMegaBall": true, "IsJackpot": true, "PrizeAmount": 0.0},
            {"PrizeTier": 1, "TierWhiteBall": 5, "TierMegaBall": false, "IsJackpot": false, "PrizeAmount": 1000.0, "Mega3": 3000.0}
        ]}
    });
    let v = serde_json::json!({"d": inner.to_string()});
    let rows = parse_megamillions(&v, d("2026-10-20"), "u", "t").unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].tier, "5+1");
    assert_eq!(rows[0].prize_text, "Jackpot");
    assert_eq!(rows[1].tier, "5+0");
    assert_eq!(rows[1].product, "builtin_3x");
    assert_eq!(rows[1].winners, Some(2));
    assert_eq!(rows[1].prize_usd, Some(3000));
    assert!(
        parse_megamillions(&v, d("2026-10-16"), "u", "t").is_err(),
        "date mismatch rejected"
    );
}
