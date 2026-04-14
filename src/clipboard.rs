use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use arboard::Clipboard;

use crate::sanitizer::{sanitize_option, TextFilter};

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
/// * `filter`        – compiled include/exclude regex filter
/// * `on_event`      – callback invoked with each [`ClipboardEvent`]
///
/// # Errors
/// Returns an error if the clipboard cannot be opened at startup.
pub fn watch<F>(
    poll_interval: Duration,
    filter: Arc<TextFilter>,
    mut on_event: F,
) -> Result<(), arboard::Error>
where
    F: FnMut(ClipboardEvent),
{
    let mut clipboard = Clipboard::new()?;
    let mut last_hash: Option<u64> = None;

    loop {
        if let Some(clean) = clipboard
            .get_text()
            .ok()
            .and_then(|raw| sanitize_option(&raw))
            .filter(|clean| filter.should_speak(clean))
        {
            let h = hash_str(&clean);
            if last_hash != Some(h) {
                last_hash = Some(h);
                on_event(ClipboardEvent { text: clean });
            }
        }
        std::thread::sleep(poll_interval);
    }
}

// ─── unit tests ──────────────────────────────────────────────────────────────
// These tests exercise the hash-based deduplication logic in isolation,
// without requiring a real clipboard (which is not available in CI).

#[cfg(test)]
#[path = "clipboard_tests.rs"]
mod tests;
