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
    let a = ClipboardEvent {
        text: "hello".into(),
    };
    let b = ClipboardEvent {
        text: "hello".into(),
    };
    let c = ClipboardEvent {
        text: "world".into(),
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}
