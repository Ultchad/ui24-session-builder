# UI24 Session Builder

UI24 Session Builder is an offline, open-source tool for preparing playback sessions for the Soundcraft Ui24R digital mixer. It is designed to replace the Windows-only UI Session Maker while sharing the same Rust processing core across CLI, Web, desktop, and mobile targets.

All processing is local. No account, cloud service, telemetry, analytics, or external server is required.

Online Web Application: [https://ultchad.github.io/ui24-session-builder/](https://ultchad.github.io/ui24-session-builder/)

## Current Status

Implemented today:

- Rust workspace and shared core libraries
- Session model and Ui24R channel assignments from `i.0` to `i.21`
- Session validation rules
- WAV, FLAC, AIFF, and MP3 metadata extraction
- WAV, FLAC, AIFF, and MP3 to FLAC conversion
- Generic and native FLAC output
- CLI audio analysis and conversion commands
- CLI session creation from an audio directory
- Official Ui24R fixture schema validation
- Static Web preview compatible with GitHub Pages
- Browser drag-and-drop and multiple-file selection
- Visible table headers, read-only filenames, editable track names, and numeric channel mappings
- Browser-side audio metadata (sample rate, duration, extension) via the Web Audio API
- Browser-side stereo-to-mono channel splitting, with identical-channel detection
- Browser `.uirecsession` download
- Browser-side ZIP session packaging (audio files plus `.uirecsession`, no external library)
- Unit, integration, documentation, and benchmark checks

The Phase 4 generator can now create and validate a `.uirecsession` JSON
configuration, a complete session folder, or a ZIP archive from
already-converted FLAC files. Automatic audio conversion during export remains
pending.

Still in development:

- External audio fixture corpus
- MP3 decoder hardening for malformed or truncated files
- ZIP export
- Real Ui24R fixture validation is now included when the official example is present
- Full session export with automatic audio conversion
- Rust/WebAssembly audio processing in the browser
- Browser microphone recording permission and capture
- Desktop applications for Windows and Linux
- Android and iOS applications

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

Create a session folder from all supported audio files in a directory:

```bash
cargo run -p ui24-session-builder -- create ./tracks ./output-session --name "Live Session"
cargo run -p ui24-session-builder -- create ./tracks ./live-session.zip --name "Live Session" --zip
```

The command analyzes every supported file, rejects mixed sample rates, converts
the sources to FLAC, assigns channels in sorted filename order, and writes the
`.uirecsession` file. Use `--zip` to write a root-level FLAC and `.uirecsession`
archive instead. Compatibility validation against a real Ui24R remains future
work.

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

The current preview supports local `.uirecsession` inspection, multiple local audio files, drag-and-drop, track editing, and local `.uirecsession` download. When raw audio files are added, the browser decodes them with the Web Audio API to compute the real sample rate, total duration, and file extension instead of placeholder values. Stereo files are decoded per channel: if the left and right channels differ, the file is split into two mono WAV tracks named `<name> - L.wav` and `<name> - R.wav`; if both channels are identical, the file is kept as a single track. Non-blocking warnings are shown when a file cannot be decoded or is split. A session can also be downloaded as a `session.zip` package containing the actual local audio files plus `session.uirecsession`, built entirely client-side with a small store-only ZIP writer (no external library). Full browser-side FLAC conversion still requires the future Rust/WebAssembly adapter, so packaged audio currently keeps its source format (typically WAV).

The file picker intentionally lists explicit file extensions instead of the
generic `audio/*` media type. This prevents Firefox Android from offering a
microphone recording action. Microphone capture is not requested or used; it is
deferred to a future, explicit feature.

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
