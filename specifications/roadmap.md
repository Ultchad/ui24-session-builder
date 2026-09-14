# Development Roadmap

## Current Status

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1 | Partially complete | Preliminary format documentation exists; sample corpus and complete validation tooling remain outstanding. |
| Phase 2 | Complete | Rust workspace, domain model, validation engine, and WAV/FLAC/AIFF metadata extraction are implemented and tested. |
| Phase 3 | Complete | In-memory conversion, generic output, benchmark, file output, and generated-fixture tests are implemented. |
| Phase 4 | Next | Session configuration and folder generation are not implemented yet. |
| Phase 5 | In progress | Initial CLI audio analysis and conversion commands are implemented. |
| Phase 6 | Planned | WebAssembly bindings are not implemented yet. |
| Phase 7 | In progress | Static browser editing, drag-and-drop, multi-file selection, and `.uirecsession` download are available; shared Rust/WASM processing is still pending. |
| Phases 8-9 | Planned | No production implementation yet. |

The project currently has no session generator, ZIP exporter, WebAssembly processing layer, or session-level export workflow.

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
- WAV, FLAC, and AIFF metadata extraction through Symphonia
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
- WAV, FLAC, and AIFF decoding before FLAC encoding
- Generic `std::io::Write` output adapter
- Native file output adapter through a destination path
- Repeatable PCM-to-FLAC benchmark
- Validation of sample rate, channel count, bit depth, frame alignment, and sample ranges
- Unit tests for successful encoding and invalid parameters
- WAV-to-FLAC and FLAC-to-FLAC round-trip tests
- AIFF-to-FLAC conversion test
- WAV, FLAC, and AIFF metadata preservation tests

Remaining:

- External WAV, FLAC, and AIFF fixture corpus for compatibility hardening

---

## Phase 4

Session generation

Deliverables:

- configuration generator
- folder generator
- export validation

---

## Phase 5

CLI

Deliverables:

```bash
ui24-session-builder create
```

Status: in progress.

Implemented for development and testing:

- `analyze <input>` for WAV, FLAC, and AIFF metadata
- `convert <input> <output>` for FLAC conversion

The `create` session workflow remains blocked on Phase 4 session generation.

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

Remaining:

- Rust/WASM audio processing
- Browser-side FLAC export

---

## Phase 8

GitHub Pages deployment

Deliverables:

- public demo
- offline support

---

## Phase 9

Flutter application

Deliverables:

- Android
- iOS
- Windows
- Linux
- macOS
