use wasm_bindgen::prelude::*;

use crate::{AudioFormat, FlacEncoder, Mp3Encoder};

/// Converts supported audio bytes into FLAC bytes for the browser bridge.
///
/// The JavaScript layer passes the raw file bytes and the original extension;
/// the browser keeps the source file unless the conversion succeeds.
#[wasm_bindgen]
pub fn convert_audio_to_flac_bytes(bytes: &[u8], extension: &str) -> Vec<u8> {
    let format = match extension
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "wav" => AudioFormat::Wav,
        "flac" => AudioFormat::Flac,
        "aiff" | "aif" => AudioFormat::Aiff,
        "mp3" => AudioFormat::Mp3,
        _ => return Vec::new(),
    };

    let encoder = FlacEncoder::default();
    encoder.convert_to_flac(bytes, format).unwrap_or_default()
}

/// Converts supported audio bytes into 320 kbps MP3 bytes for the browser bridge.
#[wasm_bindgen]
pub fn convert_audio_to_mp3_bytes(bytes: &[u8], extension: &str) -> Vec<u8> {
    let format = match extension
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "wav" => AudioFormat::Wav,
        "flac" => AudioFormat::Flac,
        "aiff" | "aif" => AudioFormat::Aiff,
        "mp3" => AudioFormat::Mp3,
        _ => return Vec::new(),
    };

    let encoder = Mp3Encoder;
    encoder.convert_to_mp3(bytes, format).unwrap_or_default()
}
