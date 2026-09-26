# clipboard-tts

A Rust daemon that monitors your clipboard and reads aloud whatever you copy, using Google Cloud Text-to-Speech.

## Features

- Real-time clipboard monitoring (configurable poll interval, default 500 ms)
- Text-to-speech via Google Cloud TTS (en-US-Chirp3-HD-Charon HD voice)
- Smart filtering — automatically skips URLs, code blocks, file paths, and indented text
- Include/exclude regex filters for fine-grained control over what gets spoken
- Configurable speaking rate (0.25–4.0×)
- Saves synthesized audio as MP3 files with original text embedded as ID3 metadata
- Hash-based deduplication — copies the same content twice? Spoken only once

## Prerequisites

- Rust 1.92+ with Cargo
- Audio hardware (speakers or headphones)
- [Google Cloud project](https://cloud.google.com/) with the [Text-to-Speech API](https://cloud.google.com/text-to-speech) enabled
- Google Cloud credentials configured (e.g. via `gcloud auth application-default login`)

## Installation

```bash
git clone https://github.com/<your-username>/clipboard-tts
cd clipboard-tts
cargo build --release
```

The binary will be at `target/release/clipboard-tts`.

## Usage

```bash
# Run with defaults — listens to clipboard, speaks aloud, saves MP3s in current directory
./clipboard-tts

# Save audio to a specific directory
./clipboard-tts --save-dir ~/audio

# Speak 1.5× faster
./clipboard-tts --speaking-rate 1.5

# Only speak text that contains "todo" or "note" (OR semantics)
./clipboard-tts --include todo --include note

# Suppress anything that looks like a password or key
./clipboard-tts --exclude "password|api.?key|secret"

# Combine: only speak notes, but not if they look like secrets
./clipboard-tts --include note --exclude secret

# Tune polling and verbosity
./clipboard-tts --poll-ms 200 --log-level debug
```

## Options

| Flag                     | Default   | Description                                               |
| ------------------------ | --------- | --------------------------------------------------------- |
| `--save-dir <PATH>`      | `.` (cwd) | Directory for generated MP3 files                         |
| `--poll-ms <MS>`         | `500`     | Clipboard poll interval in milliseconds                   |
| `--text-cap <N>`         | `5000`    | Max characters sent to TTS per request                    |
| `--speaking-rate <RATE>` | `1.0`     | Speech rate — range `0.25`–`4.0`                          |
| `--include <REGEX>`      | *(all)*   | Repeatable. Only speak text matching at least one pattern |
| `--exclude <REGEX>`      | *(none)*  | Repeatable. Suppress text matching any pattern            |
| `--log-level <LEVEL>`    | `info`    | One of: `error`, `warn`, `info`, `debug`, `trace`         |

**Filter semantics**: include and exclude patterns are applied to *sanitized* text (alphanumeric only). When both are set, exclude always wins — text matching any exclude pattern is suppressed even if it matches an include pattern.

## How It Works

```
Clipboard poll
    → detect change (hash-based dedup)
    → sanitize text (strip special chars, detect and skip code/URLs/paths)
    → apply include/exclude filters
    → call Google Cloud TTS API
    → play MP3 via system audio
    → save MP3 with ID3 metadata
```

The sanitizer strips non-alphanumeric characters and heuristically skips content that looks like code or file paths (e.g. text starting with `/`, containing `{`, `}`, `//`, 4-space indentation, or tab indentation).

## Loop Tools

Helpers for replaying the saved clips on repeat. Playback runs in the `tts-loop` systemd user unit (`ffplay -loop 0`), so every tool below controls the same player.

| Tool                    | What it does                                                          |
| ----------------------- | --------------------------------------------------------------------- |
| `tts-loop`              | Loop the newest audio file in `~/Music`                               |
| `tts-loop-stop`         | Stop the loop                                                         |
| `tts-loop-watch`        | Daemon: loop each new clip after clipboard-tts finishes playing it once |
| `tts-loop-watch-toggle` | Turn the `tts-loop-watch` service on or off                           |

`tts-loop-watch` waits for write events on a new file to go quiet (`--debounce-ms`, default 300), then waits the clip's duration (via `ffprobe`) plus `--margin-ms` (default 500) before looping it. clipboard-tts starts playback right after its last write, so this lines up with the end of the one-time playback. A newer file always replaces the pending one.

Requires `ffplay`/`ffprobe` (ffmpeg) and `notify-send`.

## Repository Layout

```
src/main.rs                 clipboard-tts daemon
src/bin/tts-loop-watch.rs   directory watcher daemon
src/*.rs                    library modules (each with a *_tests.rs file)
scripts/                    shell helpers, installed to ~/.local/bin
systemd/                    user units, installed to ~/.config/systemd/user
install.sh                  build + install everything, restart running units
```

Install or update everything with `./install.sh`. The units are not enabled automatically:

```bash
systemctl --user enable --now clipboard-tts
systemctl --user enable tts-loop-watch   # optional: auto-loop at login
```

## License

MIT
