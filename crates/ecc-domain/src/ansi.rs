//! Zero-dependency ANSI color utilities.
//! Respects `NO_COLOR` env var convention (checked at call site).

/// Wrap text in bold ANSI escape codes.
pub fn bold(text: &str, enabled: bool) -> String {
    wrap(text, "1", enabled)
}

/// Wrap text in dim ANSI escape codes.
pub fn dim(text: &str, enabled: bool) -> String {
    wrap(text, "2", enabled)
}

/// Wrap text in red ANSI escape codes.
pub fn red(text: &str, enabled: bool) -> String {
    wrap(text, "31", enabled)
}

/// Wrap text in green ANSI escape codes.
pub fn green(text: &str, enabled: bool) -> String {
    wrap(text, "32", enabled)
}

/// Wrap text in yellow ANSI escape codes.
pub fn yellow(text: &str, enabled: bool) -> String {
    wrap(text, "33", enabled)
}

/// Wrap text in cyan ANSI escape codes.
pub fn cyan(text: &str, enabled: bool) -> String {
    wrap(text, "36", enabled)
}

/// Wrap text in white ANSI escape codes.
pub fn white(text: &str, enabled: bool) -> String {
    wrap(text, "37", enabled)
}

/// Wrap text in magenta ANSI escape codes.
pub fn magenta(text: &str, enabled: bool) -> String {
    wrap(text, "35", enabled)
}

/// Wrap text in gray ANSI escape codes.
pub fn gray(text: &str, enabled: bool) -> String {
    wrap(text, "90", enabled)
}

/// Wrap text in cyan background ANSI escape codes.
pub fn bg_cyan(text: &str, enabled: bool) -> String {
    wrap(text, "46", enabled)
}

fn wrap(text: &str, code: &str, enabled: bool) -> String {
    if enabled {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

/// Strip all ANSI escape sequences from a string.
pub fn strip_ansi(text: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").expect("valid regex");
    re.replace_all(text, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bold_enabled_wraps_text() {
        assert_eq!(bold("hello", true), "\x1b[1mhello\x1b[0m");
    }

    #[test]
    fn bold_disabled_returns_plain() {
        assert_eq!(bold("hello", false), "hello");
    }

    #[test]
    fn red_enabled_wraps_text() {
        assert_eq!(red("err", true), "\x1b[31merr\x1b[0m");
    }

    #[test]
    fn strip_ansi_removes_codes() {
        let colored = bold(&red("hello", true), true);
        assert_eq!(strip_ansi(&colored), "hello");
    }

    #[test]
    fn strip_ansi_noop_on_plain() {
        assert_eq!(strip_ansi("plain text"), "plain text");
    }

    #[test]
    fn all_colors_produce_correct_codes() {
        assert!(green("x", true).contains("[32m"));
        assert!(yellow("x", true).contains("[33m"));
        assert!(cyan("x", true).contains("[36m"));
        assert!(white("x", true).contains("[37m"));
        assert!(magenta("x", true).contains("[35m"));
        assert!(gray("x", true).contains("[90m"));
        assert!(dim("x", true).contains("[2m"));
        assert!(bg_cyan("x", true).contains("[46m"));
    }
}
