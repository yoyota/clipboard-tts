use std::fs;

use google_cloud_texttospeech_v1::client::TextToSpeech;
use google_cloud_texttospeech_v1::model::synthesis_input::InputSource;
use google_cloud_texttospeech_v1::model::{
    AudioConfig, AudioEncoding, SynthesisInput, VoiceSelectionParams,
};
use id3::frame::{Content, Lyrics};
use id3::{Frame, Tag, TagLike, Version};
use tracing::warn;

// Linux NAME_MAX is 255 bytes; subtract the ".mp3" extension.
// Caps the filename and log preview — does not truncate what is sent to the API.
const FILE_EXTENSION: &str = ".mp3";
const PREVIEW_MAX_CHARS: usize = 255 - FILE_EXTENSION.len();

/// Maximum bytes the Google Cloud TTS API accepts in a single request.
pub const GOOGLE_TTS_MAX_BYTES: usize = 5000;

/// True when the API rejected our credentials (expired or stale OAuth token).
///
/// The client caches its token and checks expiry with a monotonic clock, which
/// stops during system suspend — so after resume it can keep sending an
/// expired token. Rebuilding the client fetches a fresh one.
pub fn is_unauthenticated(err: &(dyn std::error::Error + 'static)) -> bool {
    err.downcast_ref::<google_cloud_texttospeech_v1::Error>()
        .is_some_and(|e| {
            e.status()
                .is_some_and(|s| s.code.name() == "UNAUTHENTICATED")
                || e.http_status_code() == Some(401)
        })
}

fn cap_text(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// Truncates `text` to at most `PREVIEW_MAX_CHARS` bytes for use as a filename.
/// Safe for ASCII-only input (which `sanitize()` guarantees).
fn preview(text: &str) -> &str {
    &text[..text.len().min(PREVIEW_MAX_CHARS)]
}

/// Constructs the absolute save path for an audio file from a preview slice.
fn save_path(dir: &str, preview: &str) -> String {
    format!("{dir}/{preview}{FILE_EXTENSION}")
}

fn write_lyrics_tag(path: &str, text: String) -> Result<(), id3::Error> {
    let mut tag = Tag::new();
    tag.add_frame(Frame::with_content(
        "USLT",
        Content::Lyrics(Lyrics {
            lang: "eng".to_string(),
            description: String::new(),
            text,
        }),
    ));
    tag.write_to_path(path, Version::Id3v24)
}

pub async fn synthesize(
    client: &TextToSpeech,
    text: String,
    text_cap: usize,
    save_dir: &str,
    speaking_rate: f64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let text = cap_text(&text, text_cap);
    let input = SynthesisInput::default().set_input_source(InputSource::Text(text.clone()));

    let voice = VoiceSelectionParams::default()
        .set_language_code("en-US")
        .set_name("en-US-Chirp3-HD-Charon");

    let audio_config = AudioConfig::default()
        .set_audio_encoding(AudioEncoding::Mp3)
        .set_speaking_rate(speaking_rate);

    let response = client
        .synthesize_speech()
        .set_input(input)
        .set_voice(voice)
        .set_audio_config(audio_config)
        .send()
        .await?;

    let filename = save_path(save_dir, preview(&text));
    fs::write(&filename, &response.audio_content)?;

    write_lyrics_tag(&filename, text)?;

    Ok(response.audio_content.to_vec())
}

/// Like [`synthesize`], but if the API rejects the client's credentials,
/// replaces `client` with a freshly built one and retries once.
pub async fn synthesize_with_reauth(
    client: &mut TextToSpeech,
    text: String,
    text_cap: usize,
    save_dir: &str,
    speaking_rate: f64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match synthesize(client, text.clone(), text_cap, save_dir, speaking_rate).await {
        Err(e) if is_unauthenticated(e.as_ref()) => {
            warn!("credentials rejected; rebuilding client and retrying once");
            *client = TextToSpeech::builder().build().await?;
            synthesize(client, text, text_cap, save_dir, speaking_rate).await
        }
        result => result,
    }
}

// ─── unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "tts_tests.rs"]
mod tests;
