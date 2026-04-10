use super::*;
use id3::Tag;

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

// ── write_lyrics_tag() ────────────────────────────────────────────────────────

#[test]
fn uslt_tag_roundtrip() {
    let path = "/tmp/tts_test_uslt.mp3";
    let text = "hello world".to_string();

    // Write a stub file so id3 has something to append the tag to.
    std::fs::write(path, b"ID3").unwrap();
    write_lyrics_tag(path, text.clone()).unwrap();

    let tag = Tag::read_from_path(path).unwrap();
    let lyric = tag.lyrics().next().unwrap();
    assert_eq!(lyric.text, text);
}

// ── cap_text() ────────────────────────────────────────────────────────────────

#[test]
fn google_tts_max_bytes_is_5000() {
    assert_eq!(GOOGLE_TTS_MAX_BYTES, 5000);
}

#[test]
fn short_text_is_not_capped() {
    assert_eq!(cap_text("hello", 200), "hello");
}

#[test]
fn text_at_cap_is_unchanged() {
    let text = "a".repeat(200);
    assert_eq!(cap_text(&text, 200), text);
}

#[test]
fn text_over_cap_is_truncated() {
    let capped = cap_text(&"a".repeat(210), 200);
    assert_eq!(capped.chars().count(), 200);
}

#[test]
fn cap_text_handles_multibyte_chars_safely() {
    // Each Korean char is 3 bytes — byte-slicing would panic, chars() is safe.
    let capped = cap_text(&"가".repeat(210), 200);
    assert_eq!(capped.chars().count(), 200);
}

#[test]
fn cap_text_respects_custom_max() {
    let capped = cap_text(&"a".repeat(100), 50);
    assert_eq!(capped.chars().count(), 50);
}

#[test]
fn cap_text_default_allows_up_to_api_limit() {
    let text = "a".repeat(GOOGLE_TTS_MAX_BYTES);
    assert_eq!(cap_text(&text, GOOGLE_TTS_MAX_BYTES), text);
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
