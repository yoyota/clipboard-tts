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
#[path = "sanitizer_tests.rs"]
mod tests;
