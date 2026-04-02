/// Strips every character that is not ASCII alphanumeric (A–Z, a–z, 0–9).
/// Consecutive removed characters are collapsed into a single space so that
/// word boundaries are preserved for TTS rendering.
///
/// # Examples
/// ```
/// use clipboard_tts::sanitizer::sanitize;
/// assert_eq!(sanitize("Hello, World!"), "Hello World");
/// assert_eq!(sanitize("foo@bar.com"),   "foo bar com");
/// ```
pub fn sanitize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_space = true; // suppress leading space

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_was_space = false;
        } else if !last_was_space {
            out.push(' ');
            last_was_space = true;
        }
    }

    // trim trailing space that may have been appended
    if out.ends_with(' ') {
        out.pop();
    }

    out
}

/// Returns `None` when the sanitized result is empty or blank, signalling
/// that the clipboard content should be silently skipped.
pub fn sanitize_option(input: &str) -> Option<String> {
    let s = sanitize(input);
    if s.is_empty() { None } else { Some(s) }
}

// ─── unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── basic correctness ────────────────────────────────────────────────────

    #[test]
    fn plain_alphanumeric_is_unchanged() {
        assert_eq!(sanitize("HelloWorld123"), "HelloWorld123");
    }

    #[test]
    fn single_word_no_punctuation() {
        assert_eq!(sanitize("Rust"), "Rust");
    }

    #[test]
    fn trailing_punctuation_removed() {
        assert_eq!(sanitize("Hello!"), "Hello");
    }

    #[test]
    fn leading_punctuation_removed() {
        assert_eq!(sanitize("...Hello"), "Hello");
    }

    #[test]
    fn punctuation_between_words_becomes_single_space() {
        assert_eq!(sanitize("Hello, World!"), "Hello World");
    }

    #[test]
    fn multiple_consecutive_special_chars_collapse_to_one_space() {
        assert_eq!(sanitize("foo!!!bar"), "foo bar");
        assert_eq!(sanitize("foo   bar"), "foo bar");
        assert_eq!(sanitize("foo\t\n\rbar"), "foo bar");
    }

    // ── email / URL patterns ────────────────────────────────────────────────

    #[test]
    fn email_address_sanitized() {
        assert_eq!(sanitize("foo@bar.com"), "foo bar com");
    }

    #[test]
    fn url_sanitized() {
        assert_eq!(sanitize("https://example.com/path?q=1"), "https example com path q 1");
    }

    // ── numbers and mixed content ────────────────────────────────────────────

    #[test]
    fn digits_preserved() {
        assert_eq!(sanitize("Room 42"), "Room 42");
    }

    #[test]
    fn mixed_alphanumeric_and_symbols() {
        assert_eq!(sanitize("C++20 is great!"), "C 20 is great");
    }

    #[test]
    fn version_string() {
        assert_eq!(sanitize("v1.2.3-beta"), "v1 2 3 beta");
    }

    // ── unicode and non-ASCII ────────────────────────────────────────────────

    #[test]
    fn unicode_letters_are_stripped() {
        // é, ü, 한 etc. are not ASCII alphanumeric → become spaces
        assert_eq!(sanitize("café"), "caf");
        assert_eq!(sanitize("naïve"), "na ve");
    }

    #[test]
    fn emoji_stripped() {
        assert_eq!(sanitize("Hello 🌍"), "Hello");
        assert_eq!(sanitize("🔥fire"), "fire");
    }

    #[test]
    fn chinese_characters_stripped() {
        assert_eq!(sanitize("Hello世界"), "Hello");
    }

    // ── edge cases ───────────────────────────────────────────────────────────

    #[test]
    fn empty_string_returns_empty() {
        assert_eq!(sanitize(""), "");
    }

    #[test]
    fn whitespace_only_returns_empty() {
        assert_eq!(sanitize("   \t\n"), "");
    }

    #[test]
    fn all_special_chars_returns_empty() {
        assert_eq!(sanitize("!@#$%^&*()"), "");
    }

    #[test]
    fn newlines_and_tabs_collapsed() {
        assert_eq!(sanitize("line1\nline2\ttabbed"), "line1 line2 tabbed");
    }

    #[test]
    fn no_leading_space_in_output() {
        let result = sanitize("!!!hello");
        assert!(!result.starts_with(' '), "output must not start with a space");
        assert_eq!(result, "hello");
    }

    #[test]
    fn no_trailing_space_in_output() {
        let result = sanitize("hello!!!");
        assert!(!result.ends_with(' '), "output must not end with a space");
        assert_eq!(result, "hello");
    }

    #[test]
    fn single_character_alpha() {
        assert_eq!(sanitize("A"), "A");
    }

    #[test]
    fn single_character_special() {
        assert_eq!(sanitize("!"), "");
    }

    // ── sanitize_option ──────────────────────────────────────────────────────

    #[test]
    fn option_returns_some_for_valid_text() {
        assert_eq!(sanitize_option("Hello"), Some("Hello".to_string()));
    }

    #[test]
    fn option_returns_none_for_empty_result() {
        assert_eq!(sanitize_option("!!!"), None);
        assert_eq!(sanitize_option(""),    None);
        assert_eq!(sanitize_option("   "), None);
    }
}
