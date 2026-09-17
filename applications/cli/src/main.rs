use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use ui24_audio_processing::{
    AudioFormat, AudioMetadataReader, FlacEncoder, SymphoniaMetadataReader, WavEncoder,
};
use ui24_core::{ChannelAssignment, Session, SessionMetadata, SessionTrack};
use ui24_session_generator::{generate_session_folder, generate_session_zip};

/// Destination audio format for `convert` and `create`.
///
/// Only [`OutputFormat::Flac`] has been confirmed compatible with real
/// Ui24R hardware (see
/// `documentation/format_specifications/ui24r_session_format.md`).
/// [`OutputFormat::Wav`] is provided for local, non-hardware-verified
/// exports. [`OutputFormat::Mp3`] is accepted as a value but not yet
/// implemented: no MP3 encoder is wired in this workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lower")]
enum OutputFormat {
    /// FLAC output (default). Confirmed compatible with real Ui24R hardware.
    Flac,
    /// Canonical PCM WAV output. Local use only; not confirmed compatible
    /// with the Ui24R mixer.
    Wav,
    /// MP3 output. Not implemented yet.
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
        /// Destination session directory.
        output_dir: PathBuf,
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
        } => create(&input_dir, &output_dir, name, zip, format),
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
    if format_choice == OutputFormat::Mp3 {
        return Err(mp3_not_implemented_error());
    }
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

    if format_choice == OutputFormat::Wav {
        eprintln!(
            "warning: WAV output has not been verified against real Ui24R hardware; only FLAC is confirmed compatible"
        );
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
        OutputFormat::Mp3 => Err(mp3_not_implemented_error()),
    }
}

fn create(
    input_dir: &Path,
    output_dir: &Path,
    name: Option<String>,
    as_zip: bool,
    format_choice: OutputFormat,
) -> Result<(), String> {
    if format_choice == OutputFormat::Mp3 {
        return Err(mp3_not_implemented_error());
    }
    let audio_extension = format_choice.extension();
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
            let output_file = staging_dir.join(format!("{display_name}.{audio_extension}"));
            match format_choice {
                OutputFormat::Flac => FlacEncoder
                    .convert_to_flac_file(&source, format, &output_file)
                    .map_err(|error| format!("cannot convert {}: {error}", input.display()))?,
                OutputFormat::Wav => WavEncoder
                    .convert_to_wav_file(&source, format, &output_file)
                    .map_err(|error| format!("cannot convert {}: {error}", input.display()))?,
                OutputFormat::Mp3 => unreachable!("rejected above"),
            }
            converted_files.push(output_file);
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
        if as_zip {
            generate_session_zip(&session, &converted_files, audio_extension, output_dir)
                .map_err(|error| error.to_string())
        } else {
            generate_session_folder(&session, &converted_files, audio_extension, output_dir)
                .map_err(|error| error.to_string())
        }
    })();

    let _ = std::fs::remove_dir_all(&staging_dir);
    result?;
    if format_choice == OutputFormat::Wav {
        eprintln!(
            "warning: WAV output has not been verified against real Ui24R hardware; only FLAC is confirmed compatible"
        );
    }
    if as_zip {
        println!("created session archive {}", output_dir.display());
    } else {
        println!("created session in {}", output_dir.display());
    }
    Ok(())
}

fn mp3_not_implemented_error() -> String {
    "MP3 output is not implemented yet; use --format flac or --format wav".to_owned()
}

fn audio_format(path: &Path) -> Result<AudioFormat, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("{} has no supported audio extension", path.display()))?;
    AudioFormat::from_extension(extension)
        .ok_or_else(|| format!("unsupported audio extension: .{extension}"))
}
