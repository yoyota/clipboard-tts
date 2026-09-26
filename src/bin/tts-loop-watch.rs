//! Watch a directory for new audio files. When one appears, stop the current
//! loop, wait until clipboard-tts finishes playing it once, then loop it.
//!
//! Playback runs in the `tts-loop` systemd user unit — the same one the
//! `tts-loop` / `tts-loop-stop` scripts use — so they all work together.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use anyhow::Context;
use clap::Parser;
use notify::event::{AccessKind, AccessMode, ModifyKind, RenameMode};
use notify::{EventKind, RecursiveMode, Watcher};

use clipboard_tts::loop_schedule::{Action, Scheduler};

const UNIT: &str = "tts-loop";
const EXTENSIONS: [&str; 3] = ["mp3", "wav", "ogg"];

#[derive(Parser, Debug)]
#[command(about)]
struct Cli {
    /// Directory to watch.
    #[arg(long, default_value_os_t = default_dir())]
    dir: PathBuf,

    /// Quiet time after the last write event before a file counts as done.
    #[arg(long, default_value_t = 300)]
    debounce_ms: u64,

    /// Extra wait after the clip's duration, for audio buffer latency.
    #[arg(long, default_value_t = 500)]
    margin_ms: u64,
}

fn default_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("Music")
}

fn is_audio(path: &Path) -> bool {
    let hidden = path
        .file_name()
        .and_then(|n| n.to_str())
        .is_none_or(|n| n.starts_with('.'));
    let has_audio_ext = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| EXTENSIONS.iter().any(|x| ext.eq_ignore_ascii_case(x)));
    !hidden && has_audio_ext
}

/// A file finished writing, or was renamed into place.
fn is_complete(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Access(AccessKind::Close(AccessMode::Write))
            | EventKind::Modify(ModifyKind::Name(RenameMode::To))
    )
}

fn duration_of(path: &Path) -> anyhow::Result<Duration> {
    let out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .context("run ffprobe")?;
    anyhow::ensure!(
        out.status.success(),
        "ffprobe failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&out.stderr).trim()
    );
    let secs: f64 = String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .with_context(|| format!("parse ffprobe output for {}", path.display()))?;
    Duration::try_from_secs_f64(secs)
        .with_context(|| format!("invalid duration {secs} for {}", path.display()))
}

fn stop_loop() {
    let _ = Command::new("systemctl")
        .args(["--user", "stop", UNIT])
        .stderr(Stdio::null())
        .status();
}

fn start_loop(path: &Path) {
    stop_loop();
    let started = Command::new("systemd-run")
        .args(["--user", "--quiet", "--collect", &format!("--unit={UNIT}")])
        .args([
            "/usr/bin/ffplay",
            "-nodisp",
            "-loglevel",
            "quiet",
            "-loop",
            "0",
        ])
        .arg(path)
        .status();
    match started {
        Ok(s) if s.success() => {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let _ = Command::new("notify-send")
                .args(["-t", "1500", "🔁 Looping", &name])
                .status();
        }
        other => eprintln!("failed to start loop for {}: {other:?}", path.display()),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let event = match res {
            Ok(event) => event,
            Err(e) => {
                eprintln!("watch error: {e}");
                return;
            }
        };
        if !is_complete(&event.kind) {
            return;
        }
        for path in event.paths.into_iter().filter(|p| is_audio(p)) {
            // Fails only once main has exited; nothing left to notify.
            let _ = tx.send(path);
        }
    })?;
    watcher
        .watch(&cli.dir, RecursiveMode::NonRecursive)
        .with_context(|| format!("watch {}", cli.dir.display()))?;
    eprintln!("watching {}", cli.dir.display());

    let mut sched = Scheduler::new(
        Duration::from_millis(cli.debounce_ms),
        Duration::from_millis(cli.margin_ms),
    );
    loop {
        let received = match sched.next_deadline() {
            Some(d) => rx.recv_timeout(d.saturating_duration_since(Instant::now())),
            None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        let action = match received {
            Ok(path) => sched.on_event(path, Instant::now()),
            Err(RecvTimeoutError::Timeout) => sched.poll(Instant::now()),
            Err(RecvTimeoutError::Disconnected) => anyhow::bail!("watcher stopped"),
        };
        match action {
            Some(Action::StopLoop) => stop_loop(),
            Some(Action::Measure(path)) => {
                let duration = duration_of(&path).unwrap_or_else(|e| {
                    eprintln!("{e:#}; looping without waiting");
                    Duration::ZERO
                });
                sched.schedule(duration);
            }
            Some(Action::StartLoop(path)) => start_loop(&path),
            None => {}
        }
    }
}
