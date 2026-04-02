use std::path::PathBuf;

/// Common interface every TTS backend must satisfy.
pub trait TtsEngine: Send {
    /// Synthesize `text` and return raw PCM/WAV bytes.
    fn synthesize(&self, text: &str) -> Result<Vec<u8>, TtsError>;
}

// ─── errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("subprocess error: {0}")]
    Subprocess(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("empty text — nothing to synthesize")]
    EmptyText,
}

// ─── Piper backend ────────────────────────────────────────────────────────────

/// Calls the `piper` CLI as a subprocess, piping text via stdin and reading
/// WAV bytes from stdout.  This avoids linking against libpiper at the cost
/// of one process-spawn per utterance (typically < 5 ms overhead).
pub struct PiperEngine {
    /// Path to the `piper` binary (e.g. `/usr/local/bin/piper`).
    pub binary: PathBuf,
    /// Path to the ONNX voice model (e.g. `en_US-lessac-medium.onnx`).
    pub model: PathBuf,
}

impl PiperEngine {
    pub fn new(binary: impl Into<PathBuf>, model: impl Into<PathBuf>) -> Self {
        Self { binary: binary.into(), model: model.into() }
    }
}

impl TtsEngine for PiperEngine {
    fn synthesize(&self, text: &str) -> Result<Vec<u8>, TtsError> {
        if text.trim().is_empty() {
            return Err(TtsError::EmptyText);
        }

        use std::process::{Command, Stdio};
        use std::io::Write;

        let mut child = Command::new(&self.binary)
            .args([
                "--model",      self.model.to_str().unwrap_or_default(),
                "--output-raw", // raw PCM to stdout; swap for "--output-file -" for WAV
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(TtsError::Io)?;

        // Write the text to stdin, then close it so piper knows input is done.
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(TtsError::Io)?;
        }

        let output = child.wait_with_output().map_err(TtsError::Io)?;

        if !output.status.success() {
            return Err(TtsError::Subprocess(format!(
                "piper exited with status {}",
                output.status
            )));
        }

        Ok(output.stdout)
    }
}

// ─── MS Edge TTS backend ─────────────────────────────────────────────────────

/// Calls the `edge-tts` Python CLI (pip install edge-tts) as a subprocess.
/// Returns WAV/MP3 bytes captured from stdout via `--write-media /dev/stdout`.
///
/// Requires Python and `edge-tts` installed in PATH; no API key needed.
pub struct EdgeTtsEngine {
    /// Voice name, e.g. `"en-US-GuyNeural"` or `"en-US-JennyNeural"`.
    pub voice: String,
}

impl EdgeTtsEngine {
    pub fn new(voice: impl Into<String>) -> Self {
        Self { voice: voice.into() }
    }
}

impl TtsEngine for EdgeTtsEngine {
    fn synthesize(&self, text: &str) -> Result<Vec<u8>, TtsError> {
        if text.trim().is_empty() {
            return Err(TtsError::EmptyText);
        }

        use std::process::{Command, Stdio};

        let output = Command::new("edge-tts")
            .args([
                "--voice", &self.voice,
                "--text",  text,
                "--write-media", "/dev/stdout",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(TtsError::Io)?;

        if !output.status.success() {
            return Err(TtsError::Subprocess(format!(
                "edge-tts exited with status {}",
                output.status
            )));
        }

        Ok(output.stdout)
    }
}

// ─── engine selection ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum EngineKind {
    Piper { binary: PathBuf, model: PathBuf },
    EdgeTts { voice: String },
}

/// Build the concrete engine from a config enum.
pub fn build_engine(kind: EngineKind) -> Box<dyn TtsEngine> {
    match kind {
        EngineKind::Piper { binary, model } => {
            Box::new(PiperEngine::new(binary, model))
        }
        EngineKind::EdgeTts { voice } => {
            Box::new(EdgeTtsEngine::new(voice))
        }
    }
}

// ─── unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // ── mock engine for testing downstream consumers ─────────────────────────

    struct MockEngine {
        calls: Arc<Mutex<Vec<String>>>,
        response: Vec<u8>,
    }

    impl MockEngine {
        fn new(response: Vec<u8>) -> (Self, Arc<Mutex<Vec<String>>>) {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let engine = Self { calls: Arc::clone(&calls), response };
            (engine, calls)
        }
    }

    impl TtsEngine for MockEngine {
        fn synthesize(&self, text: &str) -> Result<Vec<u8>, TtsError> {
            if text.trim().is_empty() {
                return Err(TtsError::EmptyText);
            }
            self.calls.lock().unwrap().push(text.to_string());
            Ok(self.response.clone())
        }
    }

    #[test]
    fn mock_engine_records_calls() {
        let (engine, calls) = MockEngine::new(vec![0xDE, 0xAD]);
        engine.synthesize("hello").unwrap();
        engine.synthesize("world").unwrap();
        assert_eq!(*calls.lock().unwrap(), vec!["hello", "world"]);
    }

    #[test]
    fn mock_engine_returns_bytes() {
        let bytes = vec![1u8, 2, 3, 4];
        let (engine, _) = MockEngine::new(bytes.clone());
        assert_eq!(engine.synthesize("test").unwrap(), bytes);
    }

    #[test]
    fn mock_engine_rejects_empty_text() {
        let (engine, _) = MockEngine::new(vec![]);
        let err = engine.synthesize("").unwrap_err();
        assert!(matches!(err, TtsError::EmptyText));
    }

    #[test]
    fn mock_engine_rejects_whitespace_only() {
        let (engine, _) = MockEngine::new(vec![]);
        let err = engine.synthesize("   ").unwrap_err();
        assert!(matches!(err, TtsError::EmptyText));
    }

    // ── engine builder ───────────────────────────────────────────────────────

    #[test]
    fn build_engine_piper_variant() {
        // Just ensure it constructs without panic; we don't call synthesize
        // here because no binary is present in the test environment.
        let _engine = build_engine(EngineKind::Piper {
            binary: PathBuf::from("/usr/local/bin/piper"),
            model:  PathBuf::from("en_US-lessac-medium.onnx"),
        });
    }

    #[test]
    fn build_engine_edge_variant() {
        let _engine = build_engine(EngineKind::EdgeTts {
            voice: "en-US-GuyNeural".to_string(),
        });
    }

    // ── error display ────────────────────────────────────────────────────────

    #[test]
    fn tts_error_empty_text_display() {
        let e = TtsError::EmptyText;
        assert!(e.to_string().contains("empty text"));
    }

    #[test]
    fn tts_error_subprocess_display() {
        let e = TtsError::Subprocess("code 1".to_string());
        assert!(e.to_string().contains("code 1"));
    }

    #[test]
    fn tts_error_network_display() {
        let e = TtsError::Network("timeout".to_string());
        assert!(e.to_string().contains("timeout"));
    }
}
