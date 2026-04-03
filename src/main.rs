use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use google_cloud_texttospeech_v1::client::TextToSpeech;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use tracing::{error, info, warn};

use clipboard_tts::{
    clipboard::{self, ClipboardEvent},
    tts::synthesize,
};

/// Clipboard Text-to-Speech — speaks whatever you copy.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Clipboard poll interval in milliseconds.
    #[arg(long, default_value_t = 500, global = true)]
    poll_ms: u64,

    /// Log verbosity: error | warn | info | debug | trace
    #[arg(long, default_value = "info", global = true, env = "RUST_LOG")]
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

    let client = Arc::new(TextToSpeech::builder().build().await?);
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let handle = tokio::runtime::Handle::current();
    clipboard::watch(Duration::from_millis(cli.poll_ms), move |ClipboardEvent { text }| {
        match tokio::task::block_in_place(|| handle.block_on(synthesize(&client, text))) {
            Ok(audio) => {
                if let Err(e) = play(&stream_handle, audio) {
                    error!(error = %e, "playback failed");
                }
            }
            Err(e) => warn!(error = %e, "synthesis failed"),
        }
    })?;
    Ok(())
}
