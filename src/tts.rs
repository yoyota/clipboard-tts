use google_cloud_texttospeech_v1::client::TextToSpeech;
use google_cloud_texttospeech_v1::model::synthesis_input::InputSource;
use google_cloud_texttospeech_v1::model::{
    AudioConfig, AudioEncoding, SynthesisInput, VoiceSelectionParams,
};
use std::fs;

// Linux NAME_MAX is 255 bytes; subtract the ".mp3" extension.
// Caps the filename and log preview — does not truncate what is sent to the API.
const FILE_EXTENSION: &str = ".mp3";
const PREVIEW_MAX_CHARS: usize = 255 - FILE_EXTENSION.len();

/// Truncates `text` to at most `PREVIEW_MAX_CHARS` bytes for use as a filename.
/// Safe for ASCII-only input (which `sanitize()` guarantees).
fn preview(text: &str) -> &str {
    &text[..text.len().min(PREVIEW_MAX_CHARS)]
}

/// Constructs the absolute save path for an audio file from a preview slice.
fn save_path(dir: &str, preview: &str) -> String {
    format!("{}/{}{}", dir, preview, FILE_EXTENSION)
}

pub async fn synthesize(
    client: &TextToSpeech,
    text: String,
    save_dir: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let input = SynthesisInput::default()
        .set_input_source(InputSource::Text(text.clone()));

    let voice = VoiceSelectionParams::default()
        .set_language_code("en-US")
        .set_name("en-US-Chirp3-HD-Charon");

    let audio_config =
        AudioConfig::default().set_audio_encoding(AudioEncoding::Mp3);

    let response = client
        .synthesize_speech()
        .set_input(input)
        .set_voice(voice)
        .set_audio_config(audio_config)
        .send()
        .await?;

    let filename = save_path(save_dir, preview(&text));
    fs::write(&filename, &response.audio_content)?;

    Ok(response.audio_content.to_vec())
}

// ─── unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "tts_tests.rs"]
mod tests;
