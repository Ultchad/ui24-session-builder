# Development Roadmap

## Current Status

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1 | Partially complete | Preliminary format documentation exists; sample corpus and complete validation tooling remain outstanding. |
| Phase 2 | Complete | Rust workspace, domain model, validation engine, and WAV/FLAC/AIFF/MP3 metadata extraction are implemented and tested. |
| Phase 3 | Complete | In-memory conversion, generic output, benchmark, file output, generated-fixture tests, panic-safe malformed/truncated-input hardening, and a WAV output encoder sharing the same decode stage are implemented. |
| Phase 4 | In progress | Validated configuration, folder generation, CLI audio conversion, ZIP export, official fixture schema validation, and a configurable destination audio extension are implemented. |
| Phase 5 | In progress | CLI analysis, conversion, and session creation commands are implemented, including FLAC, WAV, and constant-bitrate 320 kbps MP3 destination formats. |
| Phase 6 | Complete | Rust/WASM bindings and the generated browser bundle convert supported browser audio sources to FLAC locally, with a fallback when the module cannot be loaded. |
| Phase 7 | In progress | Static browser editing, drag-and-drop, multi-file selection, Web Audio metadata, stereo split/downmix controls, Rust/WASM FLAC/WAV/MP3 conversion, in-browser ZIP packaging, and `.uirecsession` download are available. Microphone capture is explicitly deferred. |
| Phase 8 | In progress | GitHub Pages deployment workflow is configured for `main`. |
| Phase 9 | Planned | No Flutter application implementation yet. |

The project has a browser WebAssembly processing layer; real-device compatibility validation remains outstanding.

## Immediate next tasks / TODO

- [x] Finalize browser export semantics: FLAC remains the default target, WAV and MP3 320 kbps conversions are explicit, and imports are not silently renamed
- [x] Fix session packaging and naming bugs: `.uirecsession` is written with the exact required filename and the ZIP includes only the active session files
- [x] Harden the decode pipeline against malformed/truncated inputs, especially MP3 playback edge cases
- [x] Document the required USB layout for the Ui24R: FAT32 key, root `Multitrack` folder, and one folder per session containing the ZIP contents
- [x] Add and ship the Rust/WASM bridge and browser loader for FLAC conversion, including stereo-split tracks and a fallback when the generated module is missing
- [x] Add browser stereo handling options: split true stereo or downmix L/R to one mono track
- [x] Add a pure-Rust 320 kbps MP3 encoder path for CLI and browser export
- [ ] Validate generated sessions on a real Ui24R mixer using the official fixture set and a physical SD/USB export flow
- [ ] Expand browser/mobile verification for Firefox Android, track removal reliability, and download behavior
- [ ] Move from a static browser preview to a broader desktop/mobile distribution target once the core workflow is proven end-to-end

Development tooling now documents matching Rust compiler, `rust-src`,
rust-analyzer, rustfmt, and Clippy installation for Debian stable and
backports.

`documentation/developer_guides/coding_standards.md`,
`documentation/developer_guides/testing_strategy.md`, and
`documentation/developer_guides/localization.md` are now filled in,
describing the conventions already enforced in the workspace instead of
remaining empty placeholders.

## Phase 1

Reverse engineering

Deliverables:

- session format documentation
- sample collection
- format validation

Current status: partially complete. The format specification contains verified observations, but a representative sample corpus and executable format validator are still required.

---

## Phase 2

Core library

Deliverables:

- metadata extraction
- session model
- validation engine

Status: complete.

Implemented in the Rust workspace:

- `Session`, `SessionMetadata`, `SessionTrack`, and `ChannelAssignment`
- 22-track capacity validation
- Empty-session validation
- Channel-conflict validation
- Session and track sample-rate validation
- WAV, FLAC, AIFF, and MP3 metadata extraction through Symphonia
- Unit, integration, Clippy, and documentation checks

---

## Phase 3

FLAC conversion

Deliverables:

- audio conversion
- tests
- performance benchmarks

Status: complete for the current conversion scope.

Implemented:

- Pure-Rust PCM-to-FLAC encoding in memory
- WAV, FLAC, AIFF, and MP3 decoding before FLAC encoding
- Generic `std::io::Write` output adapter
- Native file output adapter through a destination path
- Repeatable PCM-to-FLAC benchmark
- Validation of sample rate, channel count, bit depth, frame alignment, and sample ranges
- Unit tests for successful encoding and invalid parameters
- WAV-to-FLAC and FLAC-to-FLAC round-trip tests
- AIFF-to-FLAC conversion test
- WAV, FLAC, and AIFF metadata preservation tests
- Malformed or truncated input hardening: metadata reading and FLAC
  conversion catch decoder panics (`std::panic::catch_unwind`) and report a
  controlled `Decode`/`ReadFailed` error instead of aborting the process,
  verified against a deliberately truncated MP3 header and against the
  real-world `Cri_wilhelm.mp3` fixture that previously failed decoding
- Shared PCM decode stage (`decode_pcm`) reused by every output-format
  encoder, so each destination format only implements its own encoding step
- `WavEncoder`: canonical PCM WAV output (8/16/24/32-bit), sharing the same
  decode stage, panic-safety, and malformed-input tests as `FlacEncoder`.
  WAV is not confirmed compatible with real Ui24R hardware; it is offered
  for local, non-hardware-verified exports only

Remaining:

- External WAV, FLAC, AIFF, and MP3 fixture corpus for compatibility hardening
- External MP3 fixture corpus and listening validation for the new encoder

---

## Phase 4

Session generation

Deliverables:

- configuration generator
- folder generator
- export validation

Status: in progress.

Implemented:

- `ui24_session_generator` Rust crate
- Validation before configuration generation
- Official `.uirecsession` fields and camelCase JSON names
- Filenames without extensions
- `i.N` channel mapping serialization
- JSON string and file output APIs
- Session folder generation from prepared FLAC sources
- Ordered FLAC copy and `.uirecsession` file creation
- ZIP archive export with root-level FLAC files and `.uirecsession`
- Official `.uirecsession` fixture schema validation
- Golden tests based on the observed Ui24R schema
- Configurable destination audio extension: `generate_configuration`,
  `generate_session_folder`, and `generate_session_zip` accept the audio
  extension used for a session (`"flac"` or `"wav"`) instead of assuming
  FLAC, while `VERIFIED_UI24R_AUDIO_EXTENSION` (`"flac"`) documents the
  only extension confirmed compatible with real hardware

Remaining:

- Byte/structure compatibility validation against generated exports on real hardware

---

## Phase 5

CLI

Deliverables:

```bash
ui24-session-builder create
```

Status: in progress.

Implemented for development and testing:

- `analyze <input>` for WAV, FLAC, AIFF, and MP3 metadata
- `convert <input> <output> [--format flac|wav|mp3]` for destination-format
  conversion (`flac` is the default and only extension confirmed compatible
  with real Ui24R hardware; `wav` is a real, working local export; `mp3`
  produces a constant-bitrate 320 kbps local export)
- `convert <input-dir> <output-dir> [--format flac|wav|mp3]` for batch
  conversion of supported audio files directly inside a directory; unrelated
  files and subdirectories are ignored, and the output directory is created
  when absent
- `create <input-dir> <output-dir> [--format flac|wav|mp3]` for sorted
  multi-track session creation in the chosen format, printing a compatibility
  warning to stderr when a format other than FLAC is used
- `create <input-dir> [--format flac|wav|mp3]` writes only `.uirecsession` in
  the input directory when no output directory is supplied; `--zip` still
  requires an explicit output path
- `create <input-dir> <output.zip> --zip [--format flac|wav|mp3]` for ZIP
  session creation in the chosen format

The CLI ZIP workflow has been tested with the supplied WAV fixtures and
produces root-level FLAC files plus `.uirecsession`.

The CLI session workflow rejects mixed sample rates and exports prepared
audio tracks plus `.uirecsession`, either as a folder or ZIP archive, in
whichever destination format was requested. Real-device compatibility tests
remain outstanding, and only FLAC has been confirmed against the observed
Ui24R schema.

---

## Phase 6

WebAssembly

Deliverables:

- browser processing
- browser export

---

## Phase 7

Web UI

Deliverables:

- drag and drop
- track mapping editor
- ZIP generation

Status: in progress for the static preview.

Implemented:

- Static browser page compatible with GitHub Pages
- Local `.uirecsession` JSON inspection
- Local file selection without upload
- Drag-and-drop file addition
- Multiple audio-file selection
- Editable track names and channel mappings
- Read-only filename display matching the `files` field
- Numeric mapping editor with a fixed `i.` prefix, exported as `i.N`
- Browser `.uirecsession` download
- Docker preview on host port 8080
- Browser-side audio metadata: sample rate, total duration, and file
  extension are computed from original audio files via the Web Audio API
  (`decodeAudioData`); destination conversion is deferred until ZIP export
  and uses the selected WAV encoder or shipped Rust/WASM adapter
- USB prep guidance shown in the web UI and CLI help: format as FAT32,
  create a root `Multitrack` folder, and store each session in its own
  subfolder with the ZIP contents inside it
- Stereo-to-mono channel splitting: stereo files are decoded and, if the
  left/right channels differ, split into two mono tracks and converted to
  `<name> L.flac` and `<name> R.flac` when FLAC is selected; if both channels
  are identical, the file is kept as a single mono-equivalent track instead
- Non-blocking warnings shown in the UI when a file cannot be decoded or
  when a stereo file is split
- Browser-side ZIP session packaging: a store-only (uncompressed) ZIP
  writer implemented in plain JavaScript bundles the actual local audio
  files together with `.uirecsession` into a downloadable `session.zip`,
  converting files only during export and reporting progress in the status
  badge, without any external library or Node.js build step
- Destination-format selector: a "Destination format" dropdown lets the
  user pick the output container for raw audio files. `flac` is the default
  and is encoded locally by the shipped Rust/WASM adapter; `wav` is also
  genuinely implemented (every non-WAV file is decoded and re-encoded as
  canonical PCM WAV, not just renamed); `mp3` is encoded locally at 320 kbps
  through the Rust/WASM adapter
- Fixed: the exported configuration file is now named exactly
  `.uirecsession` (no basename) in both the direct JSON download and the
  ZIP package, matching the schema in
  `documentation/format_specifications/ui24r_session_format.md`; it was
  previously saved/packaged as `session.uirecsession`, which the Ui24R
  mixer would not recognize
- Fixed: removing a track from the editor now also removes its underlying
  audio file from the workspace, so a track excluded from the session is
  also excluded from the downloaded ZIP package instead of lingering as an
  orphaned file

The browser file picker deliberately avoids the generic `audio/*` accept type,
so Firefox Android does not offer microphone recording. No microphone
permission is requested. Recording may be considered later as a separate,
explicit feature.

Remaining:

- Real Ui24R validation of browser-generated FLAC sessions
  as-is, typically WAV, instead of FLAC)

---

## Phase 8

GitHub Pages deployment

Deliverables:

- public demo
- offline support

Status: in progress.

Implemented:

- GitHub Actions deployment workflow
- Static publication of `applications/web/`
- Deployment on pushes to `main`
- Manual workflow dispatch

Remaining:

- Enable GitHub Pages with `Source: GitHub Actions` in repository settings
- Confirm the public Pages URL after the first deployment

---

## Phase 9

Flutter application

Deliverables:

- Android
- iOS
- Windows
- Linux
- macOS
