use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use ui24_audio_processing::{
    AudioFormat, AudioMetadataReader, FlacEncoder, SymphoniaMetadataReader,
};

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
        /// WAV, FLAC, or AIFF input file.
        input: PathBuf,
    },
    /// Convert a WAV, FLAC, or AIFF file to FLAC.
    Convert {
        /// Source audio file.
        input: PathBuf,
        /// Destination FLAC file.
        output: PathBuf,
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
    println!("bit depth: {} bits", metadata.bit_depth);
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

fn audio_format(path: &Path) -> Result<AudioFormat, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("{} has no supported audio extension", path.display()))?;
    AudioFormat::from_extension(extension)
        .ok_or_else(|| format!("unsupported audio extension: .{extension}"))
}
