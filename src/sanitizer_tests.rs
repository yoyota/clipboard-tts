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

#[test]
fn option_returns_none_for_https_url() {
    assert_eq!(sanitize_option("https://example.com"), None);
}

#[test]
fn option_returns_none_for_http_url() {
    assert_eq!(sanitize_option("http://example.com/page"), None);
}

#[test]
fn option_returns_some_for_plain_text_with_word_https() {
    // "https" without "://" is not a URL — should still be spoken
    assert!(sanitize_option("read more at https dot example dot com").is_some());
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

// ── code detection ───────────────────────────────────────────────────────

#[test]
fn option_returns_none_for_line_comment() {
    assert_eq!(sanitize_option("let x = 1; // set x"), None);
}

#[test]
fn option_returns_none_for_block_comment() {
    assert_eq!(sanitize_option("/* initialize */ int x = 0;"), None);
}

#[test]
fn option_returns_none_for_scope_resolution() {
    assert_eq!(sanitize_option("std::collections::HashMap"), None);
}

#[test]
fn option_returns_none_for_braces_and_semicolon() {
    assert_eq!(sanitize_option("fn foo() { bar(); }"), None);
}

#[test]
fn option_returns_none_for_file_path() {
    assert_eq!(sanitize_option("/home/user/documents/file.txt"), None);
    assert_eq!(sanitize_option("/usr/bin/cargo"), None);
}

#[test]
fn option_returns_none_for_four_space_indent_and_brace() {
    assert_eq!(sanitize_option("if condition {\n    do_thing();\n}"), None);
}

#[test]
fn option_returns_some_for_prose_with_semicolon() {
    // semicolons appear in plain English — should not be treated as code
    assert!(sanitize_option("I bought apples; they were fresh").is_some());
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
