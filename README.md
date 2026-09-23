# UI24 Session Builder

UI24 Session Builder is an offline, open-source tool for preparing playback sessions for the Soundcraft Ui24R digital mixer. It is designed to replace the Windows-only UI Session Maker while sharing the same Rust processing core across CLI, Web, desktop, and mobile targets.

All processing is local. No account, cloud service, telemetry, analytics, or external server is required.

Online Web Application: [https://ultchad.github.io/ui24-session-builder/](https://ultchad.github.io/ui24-session-builder/)

Before deployment, GitHub Actions validates that the static page, the Rust/WASM
bundle, FLAC/MP3 exports, and the per-track stereo actions are all present.

The `Validate Workspace` workflow runs formatting, Rust tests, Clippy,
documentation checks, and a fresh WebAssembly bridge build on pull requests
and pushes to `dev` or `main`.

The static Web UI has also been manually verified in Firefox. Android-specific
file-picker and download behavior remains under validation.

## Current Status

Implemented today:

- Rust workspace and shared core libraries
- Session model and Ui24R channel assignments from `i.0` to `i.21`
- Session validation rules
- WAV, FLAC, AIFF, and MP3 metadata extraction
- WAV, FLAC, AIFF, and MP3 conversion to FLAC, WAV, or constant-bitrate 320 kbps MP3 via `--format` on the CLI (FLAC and WAV have both been confirmed compatible with real Ui24R hardware; MP3 sessions have been confirmed to fail on real hardware with a session error)
- Panic-safe handling of malformed or truncated audio inputs (decoder panics are caught and reported as errors instead of crashing)
- Valid FLAC output for real 24-bit WAV sources, verified with long multitrack files
- Generic and native FLAC and WAV output
- CLI audio analysis and conversion commands
- CLI batch conversion of all supported audio files in a directory
- CLI session creation from an audio directory
- Official Ui24R fixture schema validation
- Static Web preview compatible with GitHub Pages
- Browser drag-and-drop and multiple-file selection
- Visible table headers, read-only filenames, editable track names, and numeric channel mappings
- Browser-side audio metadata (sample rate, duration, extension) via the Web Audio API
- Projected destination extensions (`.flac`, `.wav`, or `.mp3`) are shown immediately in the browser table and session JSON; conversion remains deferred until export
- Browser-side stereo analysis with identical-channel detection and deferred L/R split or mono downmix
- Per-track stereo action: `Downmix mono` collapses a stereo group to one displayed mono track, and `Split stereo` restores the L/R track pair; actual audio conversion remains deferred until export
- Browser-side destination-format selector with local Rust/WebAssembly FLAC, WAV, and 320 kbps MP3 conversion
- Browser `.uirecsession` download
- Browser-side ZIP session packaging (audio files plus `.uirecsession`, no external library)
- Unit, integration, documentation, and benchmark checks

The Phase 4 generator can create and validate a `.uirecsession` JSON
configuration, a complete session folder, or a ZIP archive while converting
audio to the selected FLAC, WAV, or 320 kbps MP3 destination format.

Still in development:

- External audio fixture corpus
- ZIP export
- Real Ui24R fixture validation is now included when the official example is present
- Real-device compatibility validation for full session export
- Browser microphone recording permission and capture
- Desktop applications for Windows and Linux
- Android and iOS applications

Immediate TODO list:

- [x] Align browser export semantics with the actual implementation status: default FLAC, explicit WAV/MP3 conversion, no silent renaming
- [x] Fix `.uirecsession` naming, active-file ZIP packaging, and stereo split handling in the browser workflow
- [x] Harden malformed/truncated input handling in the shared audio decode pipeline
- [x] Complete and ship the Rust/WebAssembly browser bridge for in-browser FLAC conversion
- [x] Add a pure-Rust 320 kbps MP3 encoder for CLI and browser export
- [ ] Validate generated sessions against a real Ui24R device with official fixture files
- [ ] Extend browser and Android test coverage for Firefox/Chrome mobile behavior and file downloads
- [ ] Package desktop and mobile targets once the shared Rust core is stabilized

See [specifications/roadmap.md](specifications/roadmap.md) for the detailed phase status.

## Installation

### Debian 13 / Linux development tools

Install the verified Rust toolchain with:

```bash
sudo apt-get update
sudo apt-get install -y rustc cargo rustfmt rust-clippy
sudo apt-get install -y rust-src rust-analyzer
```

Verify the tools:

```bash
rustc --version
cargo --version
cargo fmt --version
cargo clippy --version
```

The repository currently uses the stable Rust channel. The development environment and verified commands are documented in [development_environment.md](documentation/developer_guides/development_environment.md).

`rust-src` is required by rust-analyzer. On Debian 13, use the matching
backports Rust packages when the stable rust-analyzer package is too old for
the installed VS Code extension.

### Docker

Docker and Docker Compose are optional and are used to preview the static Web application. The preview publishes only host port `8080`; host ports `80` and `443` are not used.

## Build And Test

Run the complete Rust validation from the repository root:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

### CLI releases

Push a semantic-version tag on a commit already merged into `main` to create
a GitHub Release with Linux and Windows CLI binaries:

```bash
git checkout main
git pull --ff-only
git tag v0.0.1
git push origin v0.0.1
```

The release workflow publishes these assets without a version in their names:

```text
ui24-session-builder-linux-x86_64
ui24-session-builder-windows-x86_64.exe
```

The workflow builds the binaries from the tagged commit and rejects tags that
do not point to a commit contained in `main`.

Run the Phase 3 benchmark:

```bash
cargo bench -p ui24_audio_processing --bench flac_encoding
```

## CLI

The CLI is the fastest way to test local audio processing.

### Compile

```bash
cargo build -p ui24-session-builder --release
```

The binary is generated at:

```text
target/release/ui24-session-builder
```

### Run Without Installing

```bash
cargo run -p ui24-session-builder -- --help
```

Analyze a WAV, FLAC, AIFF, or MP3 file:

```bash
cargo run -p ui24-session-builder -- analyze ./input.wav
```

Convert an audio file to FLAC:

```bash
cargo run -p ui24-session-builder -- convert ./input.wav ./output.flac
```

Convert to WAV instead (confirmed compatible with real Ui24R hardware, though it is not the vendor-documented format):

```bash
cargo run -p ui24-session-builder -- convert ./input.wav ./output.wav --format wav
```

Convert every supported audio file in a directory. Non-audio files and
subdirectories are ignored; the output directory is created when absent and
must be a directory when it already exists:

```bash
cargo run -p ui24-session-builder -- convert ./input-audio ./converted-audio --format flac
```

Each output keeps the source filename stem and receives the selected extension,
for example `voice.wav` becomes `voice.flac`. `convert` supports WAV, FLAC,
AIFF, and MP3 input, and MP3 output is encoded at constant 320 kbps.

Create a session folder from all supported audio files in a directory:

```bash
cargo run -p ui24-session-builder -- create ./tracks ./output-session --name "Live Session"
cargo run -p ui24-session-builder -- create ./tracks ./live-session.zip --name "Live Session" --zip
```

When no output directory is provided, `create` analyzes the supported audio
files in the input directory and writes only `.uirecsession` directly there:

```bash
cargo run -p ui24-session-builder -- create ./tracks --name "Live Session"
```

This default mode does not convert or copy audio files, so the written
`ext`/`files` fields describe the real on-disk extension of the input files
rather than `--format` (which is ignored and only prints a note here); a
folder mixing several audio extensions is rejected instead of producing a
mismatched `.uirecsession`. Use an output directory when a complete session
export is required. `--zip` requires an explicit output path.

The command analyzes every supported file, rejects mixed sample rates, converts
the sources to FLAC, assigns channels in sorted filename order, and writes the
`.uirecsession` file. Use `--zip` to write a root-level FLAC and `.uirecsession`
archive instead. FLAC and WAV sessions have been confirmed to load and play
back on a real Ui24R mixer; MP3 sessions have been confirmed to fail with a
session error and should be avoided for real exports.

`convert` and `create` accept `--format flac|wav|mp3` (default `flac`). `flac`
and `wav` have both been confirmed compatible with real Ui24R hardware (the
CLI still prints a note to stderr that WAV is not the vendor-documented
format); `mp3` produces a constant-bitrate 320 kbps export that has been
confirmed to fail on real Ui24R hardware with a session error, so it remains
local use only.

## Web Application

The Web application is currently a static browser preview. It can be deployed to GitHub Pages because it does not require a backend.

### GitHub Pages

Online web application: [https://ultchad.github.io/ui24-session-builder/](https://ultchad.github.io/ui24-session-builder/)

The contents of `applications/web/` are static files and are deployed by
`.github/workflows/deploy-pages.yml` when changes reach `main`:

- `index.html`
- `app.js`
- `styles.css`

In GitHub repository settings, select `Settings > Pages > Source: GitHub
Actions`. The workflow publishes `applications/web/` directly; no Node.js
build or backend is required.

The current preview supports local `.uirecsession` inspection, multiple local audio files, drag-and-drop, track editing, and local `.uirecsession` download. When raw audio files are added, the browser analyzes the original audio with the Web Audio API to compute the real sample rate, total duration, and file extension without encoding it first. Stereo files are decoded per channel: if the left and right channels differ, the file is split into two mono tracks; if both channels are identical, the file is kept as a single track. A "Destination format" selector defaults to `flac` and chooses the local export format. Conversion is deferred until the ZIP export starts, so large files do not block the interface during analysis; the status badge then shows the current conversion and file count. FLAC and WAV conversion plus 320 kbps MP3 conversion are supported locally through Rust/WebAssembly. The browser does not silently rename imports. Removing a track from the editor also removes its underlying audio file, so it is excluded from any downloaded package. Non-blocking warnings are shown when a file cannot be decoded or converted. A session can also be downloaded as a `session.zip` package containing the converted local audio files plus the configuration file, named exactly `.uirecsession` as required by the Ui24R mixer, built entirely client-side with a small store-only ZIP writer (no external library).

### USB preparation for the Ui24R

For the moment, the USB key must be prepared in a strict layout before plugging it into the mixer:

- Format the key as `FAT32` only.
- Create a root folder named `Multitrack`.
- Inside `Multitrack`, create one folder per session using the name you want.
- Put the contents of the generated ZIP archive inside that per-session folder.

Example tree:

```text
USB drive/
└── Multitrack/
    ├── Live Session A/
    │   ├── .uirecsession
    │   └── *.flac
    └── Live Session B/
        ├── .uirecsession
        └── *.flac
```

The file picker intentionally lists explicit file extensions instead of the
generic `audio/*` media type. This prevents Firefox Android from offering a
microphone recording action. Microphone capture is not requested or used; it is
deferred to a future, explicit feature.

### WASM bridge

To generate the browser-side FLAC module, build the WebAssembly artifact from the Rust core:

```bash
./build-wasm.sh
```

This produces the browser package under `applications/web/wasm/`, which is committed because the static page imports it directly. The browser keeps the original file and shows a warning if the module cannot be loaded or conversion fails.

### Docker Preview

Build and start the Web preview:

```bash
docker compose -f docker-compose.web.yml up -d --build
```

Open:

```text
http://localhost:8080
```

The container listens on port `80` internally, but Docker publishes only host port `8080`. Ports `80` and `443` on the server remain available for other services.

Stop the preview:

```bash
docker compose -f docker-compose.web.yml down
```

## Desktop Applications

### Windows

The Windows desktop application is planned and is not available yet. The current Rust core is platform-independent, but no Windows packaging or desktop UI has been implemented.

### Linux

The Linux desktop application is planned and is not available yet. On Linux, use the CLI or the Docker/Web preview while the desktop target is being developed.

## Android

The Android application is planned and is not available yet. The future application will use Flutter and `flutter_rust_bridge` to consume the shared Rust core.

## iOS

The iOS application is also planned and is not available yet. It is listed here because it is part of the long-term cross-platform target.

## Supported Audio Formats

The current Rust audio-processing library supports:

- WAV
- FLAC
- AIFF
- MP3

It can extract:

- Sample rate
- Bit depth
- Channel count
- Duration

The FLAC conversion pipeline preserves the source files and writes generated output to memory, a generic writer, or a destination file. MP3 sources are decoded and exported as 16-bit FLAC; MP3 does not expose a PCM bit depth, so the CLI reports it as unknown.

## Official Ui24R Example

The repository includes an official Windows UI Session Maker example under `tmp/example/Multitrack/`. Its `.uirecsession` file documents a 15-track session with:

- Sample rate: `48000`
- Duration: `340` seconds
- Channel mappings from `i.0` to `i.14`
- FLAC audio extension

The format remains partially reverse-engineered. Unknown fields and compatibility assumptions are documented in [ui24r_session_format.md](documentation/format_specifications/ui24r_session_format.md).

## Project Structure

```text
ui24-session-builder/
├── applications/cli/       Rust command-line application
├── applications/web/       Static GitHub Pages-compatible Web preview
├── libraries/               Shared Rust libraries and session generator
├── tests/                   Integration tests
├── documentation/           Architecture and developer documentation
└── specifications/          Requirements and roadmap
```

See [ARCHITECTURE.md](ARCHITECTURE.md) and [system_overview.md](documentation/architecture/system_overview.md) for architectural details.

## Documentation

- [Project guidelines](PROJECT_GUIDELINES.md)
- [Architecture](ARCHITECTURE.md)
- [Dependency rules](documentation/architecture/dependency_rules.md)
- [Development environment](documentation/developer_guides/development_environment.md)
- [Ui24R session format](documentation/format_specifications/ui24r_session_format.md)
- [Functional requirements](specifications/functional_requirements.md)
- [Roadmap](specifications/roadmap.md)
- [Contributing](CONTRIBUTING.md)

## Principles

- Documentation first
- Offline first
- Shared business logic
- No duplicated platform logic
- No source-file modification during conversion
- No UI business logic

## License

Apache License Version 2.0

## AI Vibe Coding

AI was used in the development of this project

Development is kept reproducible through documented commands, focused commits,
and validation before each implementation milestone.

The repository remains offline-first and platform-specific applications consume
the shared Rust libraries.
