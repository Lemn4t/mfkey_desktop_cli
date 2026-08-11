use super::*;

fn force_color() {
    colored::control::set_override(true);
}

#[test]
fn heavy_rule_plain_is_ascii_equals() {
    let rule = heavy_rule(true);
    assert_eq!(rule.len(), RULE_WIDTH);
    assert!(rule.chars().all(|c| c == '='));
}

#[test]
fn heavy_rule_fancy_is_double_line() {
    let rule = heavy_rule(false);
    assert!(rule.chars().all(|c| c == '═'));
}

#[test]
fn light_rule_has_no_plain_variant() {
    assert!(light_rule().chars().all(|c| c == '─'));
}

#[test]
fn message_kind_colors_are_distinct() {
    let colors = [
        MessageKind::Info.color(),
        MessageKind::Success.color(),
        MessageKind::Warning.color(),
        MessageKind::Error.color(),
    ];
    for i in 0..colors.len() {
        for j in (i + 1)..colors.len() {
            assert_ne!(colors[i], colors[j]);
        }
    }
}

#[test]
fn plain_helpers_never_add_ansi() {
    let text = "hello";
    assert_eq!(banner_title(text, true), text);
    assert_eq!(block_style(text, true), text);
    assert_eq!(colored(text, MessageKind::Success, true), text);
    assert_eq!(colored_bold(text, MessageKind::Error, true), text);
    assert_eq!(accent(text, true), text);
    assert_eq!(emphasis(text, true), text);
    assert_eq!(muted(text, true), text);
}

#[test]
fn fancy_helpers_add_ansi_and_keep_the_text() {
    force_color();
    let text = "hello";
    for styled in [
        banner_title(text, false),
        block_style(text, false),
        colored(text, MessageKind::Success, false),
        colored_bold(text, MessageKind::Error, false),
        accent(text, false),
        emphasis(text, false),
        muted(text, false),
    ] {
        assert!(styled.contains('\x1b'), "expected ANSI in {styled:?}");
        assert!(
            styled.contains(text),
            "expected original text in {styled:?}"
        );
    }
}

#[test]
fn indent_adds_two_spaces_per_level() {
    assert_eq!(indent(0, "x"), "x");
    assert_eq!(indent(1, "x"), "  x");
    assert_eq!(indent(2, "x"), "    x");
}
