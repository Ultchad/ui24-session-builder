//! Shared PCM decoding used by every output-format encoder.
use crate::error::AudioConversionError;
use crate::AudioFormat;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

/// Decoded interleaved PCM samples plus their source format parameters.
pub(crate) struct DecodedPcm {
    pub samples: Vec<i32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

/// Decodes a supported audio container into interleaved PCM samples.
///
/// This is the shared first stage of every output-format encoder
/// (`FlacEncoder`, `WavEncoder`): decode once, then encode into whichever
/// destination container is requested.
pub(crate) fn decode_pcm(
    source: &[u8],
    format: AudioFormat,
) -> Result<DecodedPcm, AudioConversionError> {
    if source.is_empty() {
        return Err(AudioConversionError::InvalidInput(
            "The audio source is empty.".to_owned(),
        ));
    }

    let mut hint = Hint::new();
    hint.with_extension(format.extension());
    let stream = MediaSourceStream::new(
        Box::new(std::io::Cursor::new(source.to_owned())),
        MediaSourceStreamOptions::default(),
    );
    let mut probed = get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|error| AudioConversionError::Decode(error.to_string()))?;
    let track = probed.format.default_track().ok_or_else(|| {
        AudioConversionError::InvalidInput("The audio source has no default track.".to_owned())
    })?;
    let track_id = track.id;
    let codec_parameters = track.codec_params.clone();
    let sample_rate = codec_parameters.sample_rate.ok_or_else(|| {
        AudioConversionError::InvalidInput("The audio source has no sample rate.".to_owned())
    })?;
    let channels = codec_parameters
        .channels
        .ok_or_else(|| {
            AudioConversionError::InvalidInput("The audio source has no channels.".to_owned())
        })?
        .count();
    let bits_per_sample = codec_parameters.bits_per_sample.unwrap_or(16);
    let mut decoder = get_codecs()
        .make(&codec_parameters, &DecoderOptions::default())
        .map_err(|error| AudioConversionError::Decode(error.to_string()))?;
    let mut samples = Vec::new();

    loop {
        let packet = match probed.format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::ResetRequired) => {
                return Err(AudioConversionError::Decode(
                    "The audio decoder requires a reset.".to_owned(),
                ));
            }
            Err(symphonia::core::errors::Error::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(error) if error.to_string() == "end of stream" => break,
            Err(error) => {
                return Err(AudioConversionError::Decode(format!(
                    "next_packet: {error}"
                )))
            }
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(error) if error.to_string() == "end of stream" => break,
            Err(error) => return Err(AudioConversionError::Decode(format!("decode: {error}"))),
        };
        append_samples(decoded, bits_per_sample, &mut samples);
    }

    Ok(DecodedPcm {
        samples,
        sample_rate,
        channels: u16::try_from(channels).map_err(|_| {
            AudioConversionError::InvalidInput("The channel count is too large.".to_owned())
        })?,
        bits_per_sample: u16::try_from(bits_per_sample).map_err(|_| {
            AudioConversionError::InvalidInput("The bit depth is too large.".to_owned())
        })?,
    })
}

fn append_samples(buffer: AudioBufferRef<'_>, bits_per_sample: u32, destination: &mut Vec<i32>) {
    let mut sample_buffer = SampleBuffer::<i32>::new(buffer.capacity() as u64, *buffer.spec());
    sample_buffer.copy_interleaved_ref(buffer);
    let shift = 32_u32.saturating_sub(bits_per_sample);
    destination.extend(sample_buffer.samples().iter().map(|sample| sample >> shift));
}
