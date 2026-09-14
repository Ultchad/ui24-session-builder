# UI24 Session Builder

UI24 Session Builder is an offline, open-source tool for preparing playback sessions for the Soundcraft Ui24R digital mixer. It is designed to replace the Windows-only UI Session Maker while sharing the same Rust processing core across CLI, Web, desktop, and mobile targets.

All processing is local. No account, cloud service, telemetry, analytics, or external server is required.

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
- Static Web preview compatible with GitHub Pages
- Browser drag-and-drop and multiple-file selection
- Visible table headers, read-only filenames, editable track names, and numeric channel mappings
- Browser `.uirecsession` download
- Unit, integration, documentation, and benchmark checks

The Phase 4 generator can now create a validated `.uirecsession` JSON
configuration and a complete session folder from already-converted FLAC files.
CLI wiring, automatic audio conversion during export, and ZIP export are still
pending.

Still in development:

- External audio fixture corpus
- MP3 decoder hardening for malformed or truncated files
- ZIP export
- Full session export with automatic audio conversion
- ZIP export
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
```

Verify the tools:

```bash
rustc --version
cargo --version
cargo fmt --version
cargo clippy --version
```

The repository currently uses the stable Rust channel. The development environment and verified commands are documented in [development_environment.md](documentation/developer_guides/development_environment.md).

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
```

The command analyzes every supported file, rejects mixed sample rates, converts
the sources to FLAC, assigns channels in sorted filename order, and writes the
`.uirecsession` file. ZIP export and compatibility validation against a real
Ui24R remain future work.

## Web Application

The Web application is currently a static browser preview. It can be deployed to GitHub Pages because it does not require a backend.

### GitHub Pages

The contents of `applications/web/` are static files:

- `index.html`
- `app.js`
- `styles.css`

Configure GitHub Pages to publish the `applications/web/` directory, or copy these files into the deployment directory used by the repository's future Pages workflow.

The current preview supports local `.uirecsession` inspection, multiple local audio files, drag-and-drop, track editing, and local `.uirecsession` download. Browser-side audio decoding still requires the future Rust/WebAssembly adapter.

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
