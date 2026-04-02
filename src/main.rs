use std::{path::PathBuf, time::Duration};

use clipboard_tts::{
    clipboard::{self, ClipboardEvent},
    tts::{build_engine, EngineKind, TtsEngine},
};
use rodio::{Decoder, OutputStream, Sink};

fn play(bytes: Vec<u8>) -> anyhow::Result<()> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    let cursor = std::io::Cursor::new(bytes);
    let source = Decoder::new(cursor)?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // ── engine selection ────────────────────────────────────────────────────
    // Switch between Piper and Edge TTS via CLI args or env vars.
    // Default: Edge TTS (no binary required for trying it out).
    let engine: Box<dyn TtsEngine> = match std::env::var("TTS_ENGINE").as_deref() {
        Ok("piper") => build_engine(EngineKind::Piper {
            binary: PathBuf::from(
                std::env::var("PIPER_BIN").unwrap_or_else(|_| "piper".into()),
            ),
            model: PathBuf::from(
                std::env::var("PIPER_MODEL")
                    .unwrap_or_else(|_| "en_US-lessac-medium.onnx".into()),
            ),
        }),
        _ => build_engine(EngineKind::EdgeTts {
            voice: std::env::var("EDGE_VOICE")
                .unwrap_or_else(|_| "en-US-GuyNeural".to_string()),
        }),
    };

    eprintln!("clipboard-tts: listening for clipboard changes…");

    clipboard::watch(Duration::from_millis(500), |ClipboardEvent { text }| {
        eprintln!("→ synthesizing: {text:?}");
        match engine.synthesize(&text) {
            Ok(bytes) => {
                if let Err(e) = play(bytes) {
                    eprintln!("playback error: {e}");
                }
            }
            Err(e) => eprintln!("TTS error: {e}"),
        }
    })?;

    Ok(())
}
