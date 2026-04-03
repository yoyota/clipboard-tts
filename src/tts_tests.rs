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

// TODO(human): implement boundary tests for preview()
// Test that text exactly at the limit and over the limit are handled correctly.
//   ● Learn by Doing

//   Context: preview() truncates text to PREVIEW_MAX_CHARS (251 bytes) to stay within Linux's NAME_MAX. The existing tests cover short and empty inputs, but the boundary — exactly at 251 and one
//   over — is untested. This is the most important case because off-by-one errors here cause silent filename corruption.

//   Your Task: In src/tts_tests.rs, implement two tests under the TODO(human) comment. Name them text_at_limit_is_unchanged and text_over_limit_is_truncated.

//   Guidance: Use "a".repeat(N) to build inputs. For the over-limit case, think carefully: should you assert the full string content, or just the length? Consider what PREVIEW_MAX_CHARS is exported
//    as (pub(crate) is not needed — it's in scope via use super::*).

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
