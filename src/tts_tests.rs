use super::*;

// ── constants ─────────────────────────────────────────────────────────────────

#[test]
fn preview_max_chars_is_251() {
    // Linux NAME_MAX = 255 bytes; ".mp3" = 4 bytes → 251 chars for the stem.
    assert_eq!(PREVIEW_MAX_CHARS, 251);
}

// ── preview() ─────────────────────────────────────────────────────────────────

#[test]
fn short_text_is_unchanged() {
    assert_eq!(preview("hello world"), "hello world");
}

#[test]
fn empty_text_preview_is_empty() {
    assert_eq!(preview(""), "");
}

#[test]
fn text_at_limit_is_unchanged() {
    let text = "a".repeat(PREVIEW_MAX_CHARS);
    let preview_text = preview(&text);
    assert_eq!(text, preview_text);
}

#[test]
fn text_over_limit_is_truncated() {
    let text = "a".repeat(PREVIEW_MAX_CHARS + 1);
    let preview_text = preview(&text);
    assert_ne!(text, preview_text);
    assert_eq!(preview_text.len(), PREVIEW_MAX_CHARS);
}

// ── preview() → save_path() pipeline ─────────────────────────────────────────

#[test]
fn long_text_produces_valid_filename() {
    let text = "a".repeat(PREVIEW_MAX_CHARS + 10);
    let path = save_path("/tmp", preview(&text));
    let filename_len = std::path::Path::new(&path).file_name().unwrap().len();
    assert!(filename_len <= 255);
}

#[test]
fn empty_preview_produces_valid_path() {
    let path = save_path("/tmp/audio", preview(""));
    assert_eq!(path, "/tmp/audio/.mp3");
}

// ── save_path() ───────────────────────────────────────────────────────────────

#[test]
fn save_path_is_under_save_dir() {
    let path = save_path("/tmp/audio", "hello");
    assert!(path.starts_with("/tmp/audio/"));
}

#[test]
fn save_path_ends_with_mp3() {
    let path = save_path("/tmp/audio", "hello");
    assert!(path.ends_with(".mp3"));
}

#[test]
fn save_path_embeds_preview_text() {
    let path = save_path("/tmp/audio", "my audio preview");
    assert_eq!(path, "/tmp/audio/my audio preview.mp3");
}
