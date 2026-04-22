/// Strips every character that is not ASCII alphanumeric (A–Z, a–z, 0–9) or
/// an apostrophe (`'`), so contractions like "don't" survive intact.
/// Consecutive removed characters are collapsed into a single space so that
/// word boundaries are preserved for TTS rendering.
///
/// # Examples
/// ```
/// use clipboard_tts::sanitizer::sanitize;
/// assert_eq!(sanitize("Hello, World!"), "Hello World");
/// assert_eq!(sanitize("foo@bar.com"),   "foo bar com");
/// assert_eq!(sanitize("don't stop"),    "don't stop");
/// ```
pub fn sanitize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_space = true; // suppress leading space

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '\'' {
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
    Some(sanitize(input)).filter(|s| !s.is_empty())
}

/// A compiled include/exclude regex filter for clipboard text.
///
/// Text passes when:
///   1. `includes` is empty **or** at least one include pattern matches.
///   2. No exclude pattern matches.
pub struct TextFilter {
    includes: Vec<regex::Regex>,
    excludes: Vec<regex::Regex>,
}

impl TextFilter {
    /// Compiles all patterns eagerly.
    /// Returns `Err` on the first invalid pattern.
    pub fn new(includes: &[String], excludes: &[String]) -> Result<Self, regex::Error> {
        let includes = includes
            .iter()
            .map(|p| regex::Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;
        let excludes = excludes
            .iter()
            .map(|p| regex::Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { includes, excludes })
    }

    /// Returns `true` when `text` should be sent to TTS.
    pub fn should_speak(&self, text: &str) -> bool {
        (self.includes.is_empty() || self.includes.iter().any(|r| r.is_match(text)))
            && !self.excludes.iter().any(|r| r.is_match(text))
    }
}

// ─── unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "sanitizer_tests.rs"]
mod tests;
