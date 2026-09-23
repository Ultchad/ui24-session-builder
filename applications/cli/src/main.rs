use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use ui24_audio_processing::{
    AudioFormat, AudioMetadataReader, FlacEncoder, Mp3Encoder, SymphoniaMetadataReader, WavEncoder,
};
use ui24_core::{ChannelAssignment, Session, SessionMetadata, SessionTrack};
use ui24_session_generator::{
    generate_configuration, generate_session_folder, generate_session_zip,
};

/// Destination audio format for `convert` and `create`.
///
/// [`OutputFormat::Flac`] and [`OutputFormat::Wav`] have both been confirmed
/// to load and play back multitrack sessions on a real Ui24R mixer (see
/// `documentation/format_specifications/ui24r_session_format.md`). Real
/// hardware testing found that [`OutputFormat::Mp3`] sessions are rejected
/// by the mixer with a session error, even with a `.uirecsession` `ext`
/// field that matches the actual file extension; MP3 is provided for local,
/// non-hardware-compatible exports only. MP3 output is constant-bitrate
/// 320 kbps.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lower")]
enum OutputFormat {
    /// FLAC output (default). Confirmed compatible with real Ui24R hardware.
    Flac,
    /// Canonical PCM WAV output. Confirmed compatible with real Ui24R
    /// hardware, but not the vendor-documented format.
    Wav,
    /// MP3 output at constant 320 kbps. Confirmed to fail (session error) on
    /// real Ui24R hardware; local use only.
    Mp3,
}

impl OutputFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Flac => "flac",
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
        }
    }
}

#[derive(Parser)]
#[command(
    name = "ui24-session-builder",
    version,
    about = "Offline Ui24R audio session tools",
    before_help = "USB preparation for Ui24R: format the key as FAT32 only. Create a root folder named Multitrack, then add one folder per session (chosen name) and place the session ZIP contents inside that folder."
)]
struct CommandLine {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print metadata extracted from an audio file.
    Analyze {
        /// WAV, FLAC, AIFF, or MP3 input file.
        input: PathBuf,
    },
    /// Convert one audio file or every supported audio file in a directory.
    Convert {
        /// Source audio file or directory.
        input: PathBuf,
        /// Destination file for a single input, or output directory for a directory input.
        output: PathBuf,
        /// Destination audio format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Flac)]
        format: OutputFormat,
    },
    /// Create a Ui24R session folder from all supported audio files in a directory.
    Create {
        /// Directory containing WAV, FLAC, AIFF, or MP3 files.
        input_dir: PathBuf,
        /// Destination session directory. When omitted, writes `.uirecsession` in the input directory.
        output_dir: Option<PathBuf>,
        /// Optional session display name.
        #[arg(long)]
        name: Option<String>,
        /// Write a ZIP archive instead of a session directory.
        #[arg(long)]
        zip: bool,
        /// Destination audio format for every track.
        #[arg(long, value_enum, default_value_t = OutputFormat::Flac)]
        format: OutputFormat,
    },
}

fn main() -> ExitCode {
    match run(CommandLine::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(command_line: CommandLine) -> Result<(), String> {
    match command_line.command {
        Command::Analyze { input } => analyze(&input),
        Command::Convert {
            input,
            output,
            format,
        } => convert(&input, &output, format),
        Command::Create {
            input_dir,
            output_dir,
            name,
            zip,
            format,
        } => create(&input_dir, output_dir.as_deref(), name, zip, format),
    }
}

fn analyze(input: &Path) -> Result<(), String> {
    let format = audio_format(input)?;
    let source = std::fs::read(input)
        .map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    let metadata = SymphoniaMetadataReader
        .read_metadata(&source, format)
        .map_err(|error| error.to_string())?;

    println!("format: {}", metadata.format);
    println!("sample rate: {} Hz", metadata.sample_rate);
    if metadata.bit_depth == 0 {
        println!("bit depth: unknown (compressed source)");
    } else {
        println!("bit depth: {} bits", metadata.bit_depth);
    }
    println!("channels: {}", metadata.channel_count);
    println!("duration samples: {}", metadata.duration_samples);
    println!("duration seconds: {:.3}", metadata.duration_seconds());
    Ok(())
}

fn convert(input: &Path, output: &Path, format_choice: OutputFormat) -> Result<(), String> {
    if input.is_dir() {
        return convert_directory(input, output, format_choice);
    }
    if !input.is_file() {
        return Err(format!(
            "input is not a file or directory: {}",
            input.display()
        ));
    }

    let format = audio_format(input)?;
    let source = std::fs::read(input)
        .map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    convert_source(&source, format, output, format_choice)?;
    println!("wrote {}", output.display());
    Ok(())
}

fn convert_directory(
    input_dir: &Path,
    output_dir: &Path,
    format_choice: OutputFormat,
) -> Result<(), String> {
    if output_dir.exists() && !output_dir.is_dir() {
        return Err(format!(
            "output exists but is not a directory: {}",
            output_dir.display()
        ));
    }
    std::fs::create_dir_all(output_dir)
        .map_err(|error| format!("cannot create {}: {error}", output_dir.display()))?;

    let mut inputs = std::fs::read_dir(input_dir)
        .map_err(|error| format!("cannot read {}: {error}", input_dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot inspect {}: {error}", input_dir.display()))?;
    inputs.retain(|path| path.is_file() && audio_format(path).is_ok());
    inputs.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    if inputs.is_empty() {
        return Err(format!(
            "no supported audio files found in {}",
            input_dir.display()
        ));
    }

    let extension = format_choice.extension();
    let mut output_names = std::collections::HashSet::with_capacity(inputs.len());
    for input in inputs {
        let stem = input
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("invalid audio filename: {}", input.display()))?;
        let output_name = format!("{stem}.{extension}");
        if !output_names.insert(output_name.clone()) {
            return Err(format!("multiple inputs would write {output_name}"));
        }

        let format = audio_format(&input)?;
        let source = std::fs::read(&input)
            .map_err(|error| format!("cannot read {}: {error}", input.display()))?;
        let output = output_dir.join(output_name);
        convert_source(&source, format, &output, format_choice)
            .map_err(|error| format!("cannot convert {}: {error}", input.display()))?;
        println!("wrote {}", output.display());
    }

    match format_choice {
        OutputFormat::Wav => eprintln!(
            "warning: WAV output has been verified on real Ui24R hardware but is not the vendor-documented format; prefer FLAC when possible"
        ),
        OutputFormat::Mp3 => eprintln!(
            "warning: MP3 sessions have been confirmed to fail on real Ui24R hardware (session error); use --format flac or --format wav instead"
        ),
        OutputFormat::Flac => {}
    }
    Ok(())
}

fn convert_source(
    source: &[u8],
    input_format: AudioFormat,
    output: &Path,
    format_choice: OutputFormat,
) -> Result<(), String> {
    match format_choice {
        OutputFormat::Flac => FlacEncoder
            .convert_to_flac_file(source, input_format, output)
            .map_err(|error| error.to_string()),
        OutputFormat::Wav => WavEncoder
            .convert_to_wav_file(source, input_format, output)
            .map_err(|error| error.to_string()),
        OutputFormat::Mp3 => Mp3Encoder
            .convert_to_mp3_file(source, input_format, output)
            .map_err(|error| error.to_string()),
    }
}

fn create(
    input_dir: &Path,
    output_dir: Option<&Path>,
    name: Option<String>,
    as_zip: bool,
    format_choice: OutputFormat,
) -> Result<(), String> {
    if as_zip && output_dir.is_none() {
        return Err("an output path is required when --zip is used".to_owned());
    }
    let mut inputs = std::fs::read_dir(input_dir)
        .map_err(|error| format!("cannot read {}: {error}", input_dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot inspect {}: {error}", input_dir.display()))?;
    inputs.retain(|path| path.is_file() && audio_format(path).is_ok());
    inputs.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    if inputs.is_empty() {
        return Err(format!(
            "no supported audio files found in {}",
            input_dir.display()
        ));
    }
    if inputs.len() > 22 {
        return Err("a Ui24R session cannot contain more than 22 tracks".to_owned());
    }
    // Without an output path, no conversion or copy ever happens: the
    // `.uirecsession` must therefore describe the extension the files
    // already have on disk, never the (possibly default) `--format` value.
    let audio_extension = if output_dir.is_some() {
        format_choice.extension().to_owned()
    } else {
        if format_choice != OutputFormat::Flac {
            eprintln!(
                "note: --format is ignored without an output path; the existing file extension is used instead"
            );
        }
        shared_audio_extension(&inputs)?
    };

    let reader = SymphoniaMetadataReader;
    let mut tracks = Vec::with_capacity(inputs.len());
    let mut converted_files = Vec::with_capacity(inputs.len());
    let mut sample_rate = None;
    let mut duration_samples = 0_u64;
    let staging_dir = std::env::temp_dir().join(format!(
        "ui24-session-builder-staging-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&staging_dir)
        .map_err(|error| format!("cannot create staging directory: {error}"))?;

    let result = (|| {
        for (track_index, input) in inputs.iter().enumerate() {
            let format = audio_format(input)?;
            let source = std::fs::read(input)
                .map_err(|error| format!("cannot read {}: {error}", input.display()))?;
            let metadata = reader
                .read_metadata(&source, format)
                .map_err(|error| format!("cannot analyze {}: {error}", input.display()))?;
            if let Some(expected_rate) = sample_rate {
                if expected_rate != metadata.sample_rate {
                    return Err(format!(
                        "sample rate mismatch: {} uses {} Hz, expected {} Hz",
                        input.display(),
                        metadata.sample_rate,
                        expected_rate
                    ));
                }
            } else {
                sample_rate = Some(metadata.sample_rate);
            }
            duration_samples = duration_samples.max(metadata.duration_samples);

            let display_name = input
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| format!("invalid audio filename: {}", input.display()))?
                .to_owned();
            let channel = ChannelAssignment::new(track_index as u8)
                .ok_or_else(|| format!("invalid channel assignment for {}", input.display()))?;
            if output_dir.is_some() {
                let output_file = staging_dir.join(format!("{display_name}.{audio_extension}"));
                match format_choice {
                    OutputFormat::Flac => FlacEncoder
                        .convert_to_flac_file(&source, format, &output_file)
                        .map_err(|error| format!("cannot convert {}: {error}", input.display()))?,
                    OutputFormat::Wav => WavEncoder
                        .convert_to_wav_file(&source, format, &output_file)
                        .map_err(|error| format!("cannot convert {}: {error}", input.display()))?,
                    OutputFormat::Mp3 => Mp3Encoder
                        .convert_to_mp3_file(&source, format, &output_file)
                        .map_err(|error| format!("cannot convert {}: {error}", input.display()))?,
                }
                converted_files.push(output_file);
            }
            tracks.push(SessionTrack {
                display_name: display_name.clone(),
                file_name: format!("{display_name}.{audio_extension}"),
                metadata,
                channel_assignment: channel,
            });
        }

        let session = Session {
            metadata: SessionMetadata {
                name: name.unwrap_or_else(|| {
                    input_dir
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or("Ui24R Session")
                        .to_owned()
                }),
                sample_rate: sample_rate.expect("inputs is not empty"),
                duration_samples,
            },
            tracks,
        };
        if let Some(output_dir) = output_dir {
            if as_zip {
                generate_session_zip(&session, &converted_files, &audio_extension, output_dir)
                    .map_err(|error| error.to_string())
            } else {
                generate_session_folder(&session, &converted_files, &audio_extension, output_dir)
                    .map_err(|error| error.to_string())
            }
        } else {
            generate_configuration(&session, &audio_extension)
                .and_then(|configuration| configuration.write_json(input_dir.join(".uirecsession")))
                .map_err(|error| error.to_string())
        }
    })();

    let _ = std::fs::remove_dir_all(&staging_dir);
    result?;
    if output_dir.is_some() {
        match format_choice {
            OutputFormat::Wav => eprintln!(
                "warning: WAV output has been verified on real Ui24R hardware but is not the vendor-documented format; prefer FLAC when possible"
            ),
            OutputFormat::Mp3 => eprintln!(
                "warning: MP3 sessions have been confirmed to fail on real Ui24R hardware (session error); use --format flac or --format wav instead"
            ),
            OutputFormat::Flac => {}
        }
    }
    if let Some(output_dir) = output_dir {
        if as_zip {
            println!("created session archive {}", output_dir.display());
        } else {
            println!("created session in {}", output_dir.display());
        }
    } else {
        println!("wrote {}", input_dir.join(".uirecsession").display());
    }
    Ok(())
}

fn audio_format(path: &Path) -> Result<AudioFormat, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("{} has no supported audio extension", path.display()))?;
    AudioFormat::from_extension(extension)
        .ok_or_else(|| format!("unsupported audio extension: .{extension}"))
}

/// Returns the lowercase extension shared by every input, used when writing
/// a `.uirecsession` without converting or copying any file.
fn shared_audio_extension(inputs: &[PathBuf]) -> Result<String, String> {
    let mut extensions = inputs
        .iter()
        .filter_map(|path| path.extension().and_then(|value| value.to_str()))
        .map(str::to_ascii_lowercase);
    let first = extensions
        .next()
        .ok_or_else(|| "no supported audio files found".to_owned())?;
    for other in extensions {
        if other != first {
            return Err(format!(
                "cannot write .uirecsession without an output path: input files use mixed extensions (.{first} and .{other}); convert them to a single format first with `convert` or pass an output path to `create`"
            ));
        }
    }
    Ok(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
        let data_size = samples.len() * 2;
        let mut wav = Vec::with_capacity(44 + data_size);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36_u32 + data_size as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
        wav.extend_from_slice(&2_u16.to_le_bytes());
        wav.extend_from_slice(&16_u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_size as u32).to_le_bytes());
        for sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }
        wav
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ui24-cli-test-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
        dir
    }

    /// Regression test: `create` without an output path must not convert or
    /// copy any file, so the written `.uirecsession` must describe the real
    /// on-disk extension (here `.wav`) instead of the `--format` default
    /// (`flac`), which previously produced a mismatched `.uirecsession` that
    /// a real Ui24R mixer rejects.
    #[test]
    fn create_without_output_uses_real_file_extension_not_format_default() {
        let dir = unique_temp_dir("wav-ext");
        let samples: Vec<i16> = (0..4096).map(|index| (index % 512) as i16 - 256).collect();
        std::fs::write(dir.join("track.wav"), pcm_wav(&samples, 48_000))
            .expect("fixture should be writable");

        create(&dir, None, None, false, OutputFormat::Flac).expect("create should succeed");

        let config = std::fs::read_to_string(dir.join(".uirecsession"))
            .expect(".uirecsession should have been written");
        assert!(
            config.contains("\"ext\": \".wav\""),
            "expected ext to be .wav, got: {config}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn shared_audio_extension_rejects_mixed_inputs() {
        let inputs = vec![PathBuf::from("a.wav"), PathBuf::from("b.mp3")];

        let result = shared_audio_extension(&inputs);

        assert!(result.is_err());
    }

    #[test]
    fn shared_audio_extension_accepts_uniform_inputs_case_insensitively() {
        let inputs = vec![PathBuf::from("a.wav"), PathBuf::from("B.WAV")];

        let result = shared_audio_extension(&inputs).expect("uniform extension should be accepted");

        assert_eq!(result, "wav");
    }
}
