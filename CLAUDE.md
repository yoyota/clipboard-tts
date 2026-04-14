# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build
cargo build --release

# Run (requires GCP credentials)
cargo run -- --poll-ms 500 --log-level info

# Test (unit tests only — no clipboard/audio/network required)
cargo test

# Run a single test
cargo test <test_name>           # e.g. cargo test dedup_skips_repeated_content

# Check without building
cargo check
cargo clippy
```

## Architecture

The binary (`src/main.rs`) is a thin orchestrator. All logic lives in the library crate (`src/lib.rs`) which exposes three modules:

- **`sanitizer`** — Pure function pipeline. `sanitize()` strips non-ASCII-alphanumeric chars and collapses runs into single spaces. `sanitize_option()` wraps it, returning `None` for blank results. Heavily unit-tested.
- **`clipboard`** — Blocking poll loop (`watch()`). Uses `arboard` to read clipboard text, hashes content with `DefaultHasher` to skip duplicates without storing full strings, calls the user callback only on new sanitized content.
- **`tts`** — Async. Calls Google Cloud Text-to-Speech (`en-US-Chirp3-HD-Charon` voice, MP3 output), saves the result to `~/Music/<preview>.mp3`, and returns the bytes for immediate playback.

### Async/sync bridge

`clipboard::watch` is a blocking loop (runs on a dedicated thread via `block_in_place`). TTS is `async`. The bridge in `main.rs` captures the `tokio::runtime::Handle` and calls `handle.block_on(synthesize(...))` inside the sync callback — this is intentional to keep the clipboard module free of async dependencies.

### External requirements

- **Google Cloud credentials** must be available (Application Default Credentials or `GOOGLE_APPLICATION_CREDENTIALS`). The TTS client is built with `.builder().build()` which reads credentials from the environment.
- Audio playback uses `rodio` with the `mp3` feature, which requires a system audio output device.
