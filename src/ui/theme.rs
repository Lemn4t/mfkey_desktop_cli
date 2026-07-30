use colored::{Color, Colorize};

pub const RULE_WIDTH: usize = 64;

pub mod glyph {
    pub const BULLET: &str = "▸";
    pub const CHECK: &str = "✓";
    pub const CROSS: &str = "✗";
    pub const TREE: &str = "└─";
    pub const DOT: &str = "•";
    pub const BANNER_LEFT: &str = "░▒▓█";
    pub const BANNER_RIGHT: &str = "█▓▒░";
    pub const BLOCK: &str = "▓▒░";
    pub const BLOCK_END: &str = "░▒▓";
    pub const LOADING: &str = "▪▫▪";
}

pub fn heavy_rule(plain: bool) -> String {
    if plain {
        "=".repeat(RULE_WIDTH)
    } else {
        "═".repeat(RULE_WIDTH)
    }
}

pub fn light_rule() -> String {
    "─".repeat(RULE_WIDTH)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Info,
    Success,
    Warning,
    Error,
}

impl MessageKind {
    pub fn color(self) -> Color {
        match self {
            MessageKind::Info => Color::Cyan,
            MessageKind::Success => Color::Green,
            MessageKind::Warning => Color::Yellow,
            MessageKind::Error => Color::Red,
        }
    }
}

pub fn banner_title(text: &str, plain: bool) -> String {
    if plain {
        text.to_string()
    } else {
        text.cyan().bold().to_string()
    }
}

pub fn block_style(text: &str, plain: bool) -> String {
    if plain {
        text.to_string()
    } else {
        text.magenta().to_string()
    }
}

pub fn colored(text: &str, kind: MessageKind, plain: bool) -> String {
    if plain {
        text.to_string()
    } else {
        text.color(kind.color()).to_string()
    }
}

pub fn colored_bold(text: &str, kind: MessageKind, plain: bool) -> String {
    if plain {
        text.to_string()
    } else {
        text.color(kind.color()).bold().to_string()
    }
}

pub fn accent(text: &str, plain: bool) -> String {
    if plain {
        text.to_string()
    } else {
        text.yellow().to_string()
    }
}

pub fn emphasis(text: &str, plain: bool) -> String {
    if plain {
        text.to_string()
    } else {
        text.bold().to_string()
    }
}

pub fn muted(text: &str, plain: bool) -> String {
    if plain {
        text.to_string()
    } else {
        text.dimmed().to_string()
    }
}

pub fn indent(level: usize, text: &str) -> String {
    format!("{}{}", "  ".repeat(level), text)
}

#[cfg(test)]
mod tests {
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
}
