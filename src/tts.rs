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

pub async fn synthesize(
    client: &TextToSpeech,
    text: String,
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

    let preview = &text[..text.len().min(PREVIEW_MAX_CHARS)];
    let filename = format!("/home/yoyota/Music/{}{}", preview, FILE_EXTENSION);
    fs::write(&filename, &response.audio_content)?;

    Ok(response.audio_content.to_vec())
}
