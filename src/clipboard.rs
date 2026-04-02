use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};

use arboard::Clipboard;

use crate::sanitizer::sanitize_option;

/// A single clipboard event carrying the sanitized text ready for TTS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardEvent {
    pub text: String,
}

/// Computes a cheap hash of `s` to detect content changes without
/// storing the full previous string.
fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// Blocking clipboard watcher. Calls `on_event` every time new, non-empty,
/// sanitized text appears on the clipboard.
///
/// # Arguments
/// * `poll_interval` – how often to sample the clipboard (≥ 100 ms recommended)
/// * `on_event`      – callback invoked with each [`ClipboardEvent`]
///
/// # Errors
/// Returns an error if the clipboard cannot be opened at startup.
pub fn watch<F>(poll_interval: Duration, mut on_event: F) -> Result<(), arboard::Error>
where
    F: FnMut(ClipboardEvent),
{
    let mut clipboard = Clipboard::new()?;
    let mut last_hash: Option<u64> = None;

    loop {
        if let Ok(raw) = clipboard.get_text() {
            if let Some(clean) = sanitize_option(&raw) {
                let h = hash_str(&clean);
                if last_hash != Some(h) {
                    last_hash = Some(h);
                    on_event(ClipboardEvent { text: clean });
                }
            }
        }
        std::thread::sleep(poll_interval);
    }
}

// ─── unit tests ──────────────────────────────────────────────────────────────
// These tests exercise the hash-based deduplication logic in isolation,
// without requiring a real clipboard (which is not available in CI).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_string_produces_same_hash() {
        assert_eq!(hash_str("hello"), hash_str("hello"));
    }

    #[test]
    fn different_strings_produce_different_hashes() {
        assert_ne!(hash_str("hello"), hash_str("world"));
    }

    #[test]
    fn empty_string_has_stable_hash() {
        // Just verifying it doesn't panic and is deterministic.
        let h1 = hash_str("");
        let h2 = hash_str("");
        assert_eq!(h1, h2);
    }

    /// Simulate the deduplication logic that `watch` uses: only emit when
    /// the hash changes.
    #[test]
    fn dedup_skips_repeated_content() {
        let inputs = vec![
            "Hello World",
            "Hello World", // duplicate — should be skipped
            "New content",
        ];

        let mut last_hash: Option<u64> = None;
        let mut emitted: Vec<String> = Vec::new();

        for raw in inputs {
            if let Some(clean) = sanitize_option(raw) {
                let h = hash_str(&clean);
                if last_hash != Some(h) {
                    last_hash = Some(h);
                    emitted.push(clean);
                }
            }
        }

        assert_eq!(emitted, vec!["Hello World", "New content"]);
    }

    #[test]
    fn dedup_emits_again_after_content_changes_back() {
        let inputs = vec!["A", "B", "A"];

        let mut last_hash: Option<u64> = None;
        let mut emitted: Vec<String> = Vec::new();

        for raw in inputs {
            if let Some(clean) = sanitize_option(raw) {
                let h = hash_str(&clean);
                if last_hash != Some(h) {
                    last_hash = Some(h);
                    emitted.push(clean);
                }
            }
        }

        // "A" appears twice — once after init, once after "B" changes it.
        assert_eq!(emitted, vec!["A", "B", "A"]);
    }

    #[test]
    fn special_char_only_clipboard_content_is_silently_skipped() {
        let inputs = vec!["Hello", "!@#$", "World"];

        let mut last_hash: Option<u64> = None;
        let mut emitted: Vec<String> = Vec::new();

        for raw in inputs {
            if let Some(clean) = sanitize_option(raw) {
                let h = hash_str(&clean);
                if last_hash != Some(h) {
                    last_hash = Some(h);
                    emitted.push(clean);
                }
            }
        }

        // "!@#$" sanitizes to "" → None → skipped entirely
        assert_eq!(emitted, vec!["Hello", "World"]);
    }

    #[test]
    fn clipboard_event_eq() {
        let a = ClipboardEvent { text: "hello".into() };
        let b = ClipboardEvent { text: "hello".into() };
        let c = ClipboardEvent { text: "world".into() };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
