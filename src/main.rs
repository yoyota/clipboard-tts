use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use google_cloud_texttospeech_v1::client::TextToSpeech;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use tracing::{error, info};

use clipboard_tts::{
    clipboard::{self, ClipboardEvent},
    tts::{synthesize, GOOGLE_TTS_MAX_BYTES},
};

/// Clipboard Text-to-Speech — speaks whatever you copy.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Directory where synthesized MP3 files are saved. Defaults to the current working directory.
    #[arg(long)]
    save_dir: Option<PathBuf>,

    /// Clipboard poll interval in milliseconds.
    #[arg(long, default_value_t = 500)]
    poll_ms: u64,

    /// Maximum characters sent to the TTS API per request.
    /// Defaults to the Google Cloud TTS API limit (5000 bytes).
    #[arg(long, default_value_t = GOOGLE_TTS_MAX_BYTES)]
    text_cap: usize,

    /// Speaking rate passed to the TTS API (0.25–4.0, where 1.0 is normal speed).
    #[arg(long, default_value_t = 1.0)]
    speaking_rate: f64,

    /// Log verbosity: error | warn | info | debug | trace
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
}

fn play(stream_handle: &OutputStreamHandle, bytes: Vec<u8>) -> anyhow::Result<()> {
    let sink = Sink::try_new(stream_handle)?;
    sink.append(Decoder::new(std::io::Cursor::new(bytes))?);
    sink.sleep_until_end();
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| cli.log_level.as_str().into()),
        )
        .with_target(false)
        .init();

    info!(poll_ms = cli.poll_ms, "listening for clipboard changes");

    let save_dir = cli
        .save_dir
        .map(Ok)
        .unwrap_or_else(std::env::current_dir)?
        .to_string_lossy()
        .into_owned();
    let text_cap = cli.text_cap;
    let speaking_rate = cli.speaking_rate;
    let client = Arc::new(TextToSpeech::builder().build().await?);
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let handle = tokio::runtime::Handle::current();

    let on_event = move |ClipboardEvent { text }: ClipboardEvent| {
        let result = tokio::task::block_in_place(|| {
            handle.block_on(synthesize(
                &client,
                text,
                text_cap,
                &save_dir,
                speaking_rate,
            ))
        });
        if let Err(e) = result.and_then(|audio| play(&stream_handle, audio).map_err(Into::into)) {
            error!(error = %e);
        }
    };

    clipboard::watch(Duration::from_millis(cli.poll_ms), on_event)?;
    Ok(())
}
