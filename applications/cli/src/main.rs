use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use ui24_audio_processing::{
    AudioFormat, AudioMetadataReader, FlacEncoder, SymphoniaMetadataReader,
};
use ui24_core::{ChannelAssignment, Session, SessionMetadata, SessionTrack};
use ui24_session_generator::{generate_session_folder, generate_session_zip};

#[derive(Parser)]
#[command(
    name = "ui24-session-builder",
    version,
    about = "Offline Ui24R audio session tools"
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
    /// Convert a WAV, FLAC, AIFF, or MP3 file to FLAC.
    Convert {
        /// Source audio file.
        input: PathBuf,
        /// Destination FLAC file.
        output: PathBuf,
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
        Command::Convert { input, output } => convert(&input, &output),
        Command::Create {
            input_dir,
            output_dir,
            name,
            zip,
        } => create(&input_dir, &output_dir, name, zip),
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

fn convert(input: &Path, output: &Path) -> Result<(), String> {
    let format = audio_format(input)?;
    let source = std::fs::read(input)
        .map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    FlacEncoder
        .convert_to_flac_file(&source, format, output)
        .map_err(|error| error.to_string())?;
    println!("wrote {}", output.display());
    Ok(())
}

fn create(
    input_dir: &Path,
    output_dir: &Path,
    name: Option<String>,
    as_zip: bool,
) -> Result<(), String> {
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
    let encoder = FlacEncoder;
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
            let output_file = staging_dir.join(format!("{display_name}.flac"));
            encoder
                .convert_to_flac_file(&source, format, &output_file)
                .map_err(|error| format!("cannot convert {}: {error}", input.display()))?;
            converted_files.push(output_file);
            tracks.push(SessionTrack {
                display_name: display_name.clone(),
                file_name: format!("{display_name}.flac"),
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
            generate_session_zip(&session, &converted_files, output_dir)
                .map_err(|error| error.to_string())
        } else {
            generate_session_folder(&session, &converted_files, output_dir)
                .map_err(|error| error.to_string())
        }
    })();

    let _ = std::fs::remove_dir_all(&staging_dir);
    result?;
    if as_zip {
        println!("created session archive {}", output_dir.display());
    } else {
        println!("created session in {}", output_dir.display());
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
