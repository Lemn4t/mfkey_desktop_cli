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
#[path = "../tests/ui_theme.rs"]
mod tests;
