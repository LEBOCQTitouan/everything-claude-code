//! Reads and validates the `--feature-stdin` argument from standard input.
//!
//! Four validation guards are applied in strict order:
//! 1. **TTY check** — if `is_tty` is true, reject immediately without reading.
//! 2. **Bounded read** — read at most 64 KB + 1 byte to detect over-size input.
//! 3. **UTF-8 validation** — reject non-UTF-8 byte sequences at the boundary.
//! 4. **Trailing-LF strip + non-empty check** — strip at most one trailing `\n`,
//!    then reject the empty string.
//!
//! The TTY check MUST come first — no bytes are ever read from a TTY.

use std::io::Read;

/// Errors returned by [`read_feature_from_stdin`].
#[derive(Debug, thiserror::Error)]
pub enum FeatureInputError {
    /// The feature string was empty (or contained only a single newline).
    #[error("feature is empty")]
    Empty,
    /// The stdin bytes were not valid UTF-8.
    #[error("invalid UTF-8 on stdin")]
    InvalidUtf8,
    /// The stdin buffer exceeded the 64 KB size limit.
    #[error("stdin exceeds 64KB limit")]
    TooLarge,
    /// Stdin is connected to a TTY; the caller must pipe input.
    #[error("stdin is a TTY; pipe input or use positional feature arg")]
    IsTty,
    /// An I/O error occurred while reading stdin.
    #[error("stdin read error: {0}")]
    Io(std::io::Error),
}

/// Read and validate a feature string from a reader (usually stdin).
///
/// # Guards (applied in order)
/// 1. If `is_tty` → `Err(FeatureInputError::IsTty)` — no read performed.
/// 2. Read at most 64 KB + 1 byte; if more → `Err(FeatureInputError::TooLarge)`.
/// 3. Validate UTF-8; if invalid → `Err(FeatureInputError::InvalidUtf8)`.
/// 4. Strip at most one trailing `\n`; if result is empty → `Err(FeatureInputError::Empty)`.
pub fn read_feature_from_stdin<R: Read>(
    reader: R,
    is_tty: bool,
) -> Result<String, FeatureInputError> {
    // Guard 1: TTY check — must come before any read.
    if is_tty {
        return Err(FeatureInputError::IsTty);
    }

    // Guard 2: Bounded read — take at most 64KB + 1 byte to detect over-size input.
    const LIMIT: u64 = 64 * 1024;
    let mut buf = Vec::new();
    reader
        .take(LIMIT + 1)
        .read_to_end(&mut buf)
        .map_err(FeatureInputError::Io)?;

    if buf.len() > LIMIT as usize {
        return Err(FeatureInputError::TooLarge);
    }

    // Guard 3: UTF-8 validation.
    let mut s = String::from_utf8(buf).map_err(|_| FeatureInputError::InvalidUtf8)?;

    // Guard 4: Strip at most one trailing LF.
    if s.ends_with('\n') {
        s.pop();
    }

    // Guard 5: Non-empty check.
    if s.is_empty() {
        return Err(FeatureInputError::Empty);
    }

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_input_strips_single_trailing_lf() {
        let result = read_feature_from_stdin(b"foo\n" as &[u8], false);
        assert_eq!(result.unwrap(), "foo");
    }

    #[test]
    fn feature_input_preserves_second_lf() {
        let result = read_feature_from_stdin(b"foo\n\n" as &[u8], false);
        assert_eq!(result.unwrap(), "foo\n");
    }

    #[test]
    fn feature_input_preserves_cr_before_lf() {
        let result = read_feature_from_stdin(b"foo\r\n" as &[u8], false);
        assert_eq!(result.unwrap(), "foo\r");
    }

    #[test]
    fn feature_input_rejects_invalid_utf8() {
        let result = read_feature_from_stdin(b"\xff" as &[u8], false);
        assert!(matches!(result, Err(FeatureInputError::InvalidUtf8)));
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "invalid UTF-8 on stdin");
    }

    #[test]
    fn feature_input_accepts_64kb_exactly() {
        let input = vec![b'a'; 64 * 1024];
        let result = read_feature_from_stdin(input.as_slice(), false);
        assert!(
            result.is_ok(),
            "expected Ok for exactly 64KB input, got {result:?}"
        );
    }

    #[test]
    fn feature_input_rejects_64kb_plus_one() {
        // 65_537 bytes = 64KB + 1 — must return TooLarge with the exact display message.
        let input = vec![b'a'; 65_537];
        let result = read_feature_from_stdin(input.as_slice(), false);
        assert!(
            matches!(result, Err(FeatureInputError::TooLarge)),
            "expected Err(TooLarge) for 65537-byte input, got {result:?}"
        );
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "stdin exceeds 64KB limit");
    }

    struct CountingReader<R: std::io::Read> {
        inner: R,
        count: std::rc::Rc<std::cell::Cell<u32>>,
    }

    impl<R: std::io::Read> std::io::Read for CountingReader<R> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.count.set(self.count.get() + 1);
            self.inner.read(buf)
        }
    }

    #[test]
    fn feature_input_rejects_tty_before_any_read() {
        let count = std::rc::Rc::new(std::cell::Cell::new(0u32));
        let reader = CountingReader {
            inner: b"some data".as_ref(),
            count: std::rc::Rc::clone(&count),
        };
        let result = read_feature_from_stdin(reader, true);
        assert!(
            matches!(result, Err(FeatureInputError::IsTty)),
            "expected Err(IsTty), got {result:?}"
        );
        assert_eq!(count.get(), 0, "no read() call should occur on TTY path");
    }

    #[test]
    fn feature_input_rejects_empty_after_strip() {
        // Empty bytes (immediate EOF) must return Err(Empty) with "feature is empty" display.
        let result_empty = read_feature_from_stdin(b"".as_ref(), false);
        assert!(matches!(result_empty, Err(FeatureInputError::Empty)));
        let err_empty = result_empty.unwrap_err();
        assert_eq!(err_empty.to_string(), "feature is empty");

        // A single newline that gets stripped must also return Err(Empty).
        let result_newline = read_feature_from_stdin(b"\n".as_ref(), false);
        assert!(matches!(result_newline, Err(FeatureInputError::Empty)));
        let err_newline = result_newline.unwrap_err();
        assert_eq!(err_newline.to_string(), "feature is empty");
    }

    #[test]
    fn feature_input_preserves_bom_prefix() {
        // UTF-8 BOM is \xEF\xBB\xBF (3 bytes), which decodes to U+FEFF.
        // String::from_utf8 accepts it, so the BOM must be returned byte-identical.
        let result = read_feature_from_stdin(b"\xEF\xBB\xBFfoo" as &[u8], false);
        assert_eq!(result.unwrap(), "\u{FEFF}foo");
    }

    #[test]
    fn feature_input_preserves_metachars_control_and_nul() {
        // Canonical payload: backtick, dquote, squote, dollar, backslash, CR,
        // U+0001, U+001F, U+007F DEL, NUL, text — no trailing LF.
        let payload: &[u8] = b"feat-\x60-\x22-\x27-\x24-\x5c-\r-\x01-\x1f-\x7f-\x00-end";
        let result = read_feature_from_stdin(payload, false);
        let s = result.expect("should accept payload with metachars, control chars, and NUL");
        assert_eq!(
            s.as_bytes(),
            payload,
            "returned bytes must equal input bytes exactly"
        );
    }
}
