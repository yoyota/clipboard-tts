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
    assert_eq!(
        sanitize("https://example.com/path?q=1"),
        "https example com path q 1"
    );
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
    assert!(
        !result.starts_with(' '),
        "output must not start with a space"
    );
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
    assert_eq!(sanitize_option(""), None);
    assert_eq!(sanitize_option("   "), None);
}

// ── TextFilter::new — construction ──────────────────────────────────────

mod text_filter_new {
    use super::*;

    #[test]
    fn empty_patterns_succeeds() {
        let result = TextFilter::new(&[], &[]);
        assert!(result.is_ok(), "TextFilter::new(&[], &[]) must succeed");
    }

    #[test]
    fn valid_include_pattern_compiles() {
        let result = TextFilter::new(&["hello".to_string()], &[]);
        assert!(
            result.is_ok(),
            "valid include pattern must compile without error"
        );
    }

    #[test]
    fn valid_exclude_pattern_compiles() {
        let result = TextFilter::new(&[], &["secret".to_string()]);
        assert!(
            result.is_ok(),
            "valid exclude pattern must compile without error"
        );
    }

    #[test]
    fn multiple_valid_patterns_in_both_slices_compiles() {
        let includes = vec!["foo".to_string(), r"\d+".to_string()];
        let excludes = vec!["bar".to_string(), r"^skip".to_string()];
        let result = TextFilter::new(&includes, &excludes);
        assert!(
            result.is_ok(),
            "multiple valid patterns in both slices must compile"
        );
    }

    #[test]
    fn invalid_include_pattern_returns_err() {
        let result = TextFilter::new(&["[unclosed".to_string()], &[]);
        assert!(
            result.is_err(),
            "invalid include pattern must return Err, got Ok"
        );
    }

    #[test]
    fn invalid_exclude_pattern_returns_err() {
        let result = TextFilter::new(&[], &["[unclosed".to_string()]);
        assert!(
            result.is_err(),
            "invalid exclude pattern must return Err, got Ok"
        );
    }

    #[test]
    fn first_valid_second_invalid_returns_err() {
        // Fail-fast: even though first pattern is valid, the second is not.
        let result = TextFilter::new(&["valid".to_string(), "[bad".to_string()], &[]);
        assert!(
            result.is_err(),
            "second invalid pattern must still cause Err"
        );
    }
}

// ── TextFilter::should_speak — include-only ──────────────────────────────

mod text_filter_include_only {
    use super::*;

    #[test]
    fn no_include_patterns_always_true() {
        let f = TextFilter::new(&[], &[]).unwrap();
        assert!(
            f.should_speak("anything at all"),
            "empty include list must pass every text"
        );
    }

    #[test]
    fn include_pattern_partial_match_returns_true() {
        let f = TextFilter::new(&["hello".to_string()], &[]).unwrap();
        let text = "say hello world";
        assert!(
            f.should_speak(text),
            "substring match on '{text}' must return true"
        );
    }

    #[test]
    fn include_pattern_anchored_exact_match_returns_true() {
        let f = TextFilter::new(&[r"^hello$".to_string()], &[]).unwrap();
        let text = "hello";
        assert!(
            f.should_speak(text),
            "anchored exact match on '{text}' must return true"
        );
    }

    #[test]
    fn include_pattern_no_match_returns_false() {
        let f = TextFilter::new(&["hello".to_string()], &[]).unwrap();
        let text = "goodbye world";
        assert!(
            !f.should_speak(text),
            "non-matching text '{text}' must return false when include is set"
        );
    }

    #[test]
    fn multiple_include_patterns_or_semantics_second_matches() {
        let f = TextFilter::new(
            &["alpha".to_string(), "beta".to_string()],
            &[],
        )
        .unwrap();
        let text = "this is beta testing";
        assert!(
            f.should_speak(text),
            "text matching only the second include pattern '{text}' must return true"
        );
    }

    #[test]
    fn multiple_include_patterns_none_match_returns_false() {
        let f = TextFilter::new(
            &["alpha".to_string(), "beta".to_string()],
            &[],
        )
        .unwrap();
        let text = "gamma ray";
        assert!(
            !f.should_speak(text),
            "text matching no include pattern '{text}' must return false"
        );
    }
}

// ── TextFilter::should_speak — exclude-only ──────────────────────────────

mod text_filter_exclude_only {
    use super::*;

    #[test]
    fn no_exclude_patterns_always_true() {
        let f = TextFilter::new(&[], &[]).unwrap();
        assert!(
            f.should_speak("anything"),
            "empty exclude list must pass every text"
        );
    }

    #[test]
    fn exclude_pattern_no_match_returns_true() {
        let f = TextFilter::new(&[], &["secret".to_string()]).unwrap();
        let text = "this is fine";
        assert!(
            f.should_speak(text),
            "text not matching exclude pattern '{text}' must return true"
        );
    }

    #[test]
    fn exclude_pattern_matches_returns_false() {
        let f = TextFilter::new(&[], &["secret".to_string()]).unwrap();
        let text = "top secret document";
        assert!(
            !f.should_speak(text),
            "text matching exclude pattern '{text}' must return false"
        );
    }

    #[test]
    fn multiple_exclude_patterns_last_matches_returns_false() {
        let f = TextFilter::new(
            &[],
            &["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
        )
        .unwrap();
        let text = "all about gamma rays";
        assert!(
            !f.should_speak(text),
            "text matching only last exclude pattern '{text}' must return false"
        );
    }
}

// ── TextFilter::should_speak — include + exclude combined ────────────────

mod text_filter_include_and_exclude {
    use super::*;

    #[test]
    fn matches_include_no_exclude_returns_true() {
        let f = TextFilter::new(
            &["important".to_string()],
            &["skip".to_string()],
        )
        .unwrap();
        let text = "this is important";
        assert!(
            f.should_speak(text),
            "text matching include and not exclude '{text}' must return true"
        );
    }

    #[test]
    fn matches_include_and_exclude_returns_false() {
        // Exclude wins over include.
        let f = TextFilter::new(
            &["important".to_string()],
            &["skip".to_string()],
        )
        .unwrap();
        let text = "important but skip this";
        assert!(
            !f.should_speak(text),
            "text matching both include and exclude '{text}' must return false (exclude wins)"
        );
    }

    #[test]
    fn matches_no_include_but_matches_exclude_returns_false() {
        // Include gate fails first; result must still be false.
        let f = TextFilter::new(
            &["needed".to_string()],
            &["skip".to_string()],
        )
        .unwrap();
        let text = "please skip this one";
        assert!(
            !f.should_speak(text),
            "text failing include gate (even if it matches exclude) '{text}' must return false"
        );
    }

    #[test]
    fn matches_no_include_and_no_exclude_returns_false() {
        // Include gate is set but nothing matches → false regardless of exclude.
        let f = TextFilter::new(
            &["needed".to_string()],
            &["skip".to_string()],
        )
        .unwrap();
        let text = "completely unrelated";
        assert!(
            !f.should_speak(text),
            "text failing include gate and not matching exclude '{text}' must return false"
        );
    }
}

// ── TextFilter::should_speak — partial-match semantics ───────────────────

mod text_filter_partial_match {
    use super::*;

    #[test]
    fn unanchored_pattern_matches_substring() {
        // "foo" should match anywhere in the string without anchoring.
        let f = TextFilter::new(&["foo".to_string()], &[]).unwrap();
        let text = "prefix foo suffix";
        assert!(
            f.should_speak(text),
            "unanchored pattern 'foo' must match substring in '{text}'"
        );
    }

    #[test]
    fn anchored_pattern_does_not_match_substring() {
        // "^foo$" must NOT match when "foo" is embedded.
        let f = TextFilter::new(&[r"^foo$".to_string()], &[]).unwrap();
        let embedded = "prefix foo suffix";
        assert!(
            !f.should_speak(embedded),
            "anchored pattern '^foo$' must not match substring in '{embedded}'"
        );
    }

    #[test]
    fn anchored_pattern_matches_exact_string() {
        let f = TextFilter::new(&[r"^foo$".to_string()], &[]).unwrap();
        let exact = "foo";
        assert!(
            f.should_speak(exact),
            "anchored pattern '^foo$' must match exact string '{exact}'"
        );
    }

    #[test]
    fn digit_anchor_matches_digit_leading_text_only() {
        let f = TextFilter::new(&[r"^\d+".to_string()], &[]).unwrap();

        let digit_leading = "42 is the answer";
        assert!(
            f.should_speak(digit_leading),
            "pattern '^\\d+' must match digit-leading text '{digit_leading}'"
        );

        let letter_leading = "answer is 42";
        assert!(
            !f.should_speak(letter_leading),
            "pattern '^\\d+' must not match non-digit-leading text '{letter_leading}'"
        );
    }
}
