use super::*;

fn force_color() {
    colored::control::set_override(true);
}

#[test]
fn title_plain_has_no_ansi_and_no_art() {
    let lines = render_title(OutputMode::Plain);
    assert_eq!(lines[0], "MIFARE Classic Key Recovery Tool");
    for line in &lines {
        assert!(
            !line.contains('\x1b'),
            "plain output must have no ANSI codes: {line:?}"
        );
    }
}

#[test]
fn title_fancy_contains_subtitle_and_ansi() {
    force_color();
    let lines = render_title(OutputMode::Fancy);
    assert!(lines.iter().any(|l| l.contains("Flipper Zero")));
    assert!(
        lines.iter().any(|l| l.contains('\x1b')),
        "fancy output should be colored"
    );
}

#[test]
fn progress_pct_handles_zero_total() {
    let p = Progress {
        current: 3,
        total: 0,
    };
    assert_eq!(p.pct(), 0.0);
}

#[test]
fn progress_pct_computes_fraction() {
    let p = Progress {
        current: 1,
        total: 4,
    };
    assert_eq!(p.pct(), 25.0);
}
