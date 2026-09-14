# Development Roadmap

## Current Status

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1 | Partially complete | Preliminary format documentation exists; sample corpus and complete validation tooling remain outstanding. |
| Phase 2 | Complete | Rust workspace, domain model, validation engine, and WAV/FLAC/AIFF metadata extraction are implemented and tested. |
| Phase 3 | In progress | In-memory conversion, generic output, benchmark, and file output are implemented; fixture breadth remains. |
| Phases 4-9 | Planned | No production implementation yet. |

The project currently has no UI, CLI, session generator, ZIP exporter, or session-level export workflow.

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

Status: in progress.

Implemented:

- Pure-Rust PCM-to-FLAC encoding in memory
- WAV, FLAC, and AIFF decoding before FLAC encoding
- Generic `std::io::Write` output adapter
- Native file output adapter through a destination path
- Repeatable PCM-to-FLAC benchmark
- Validation of sample rate, channel count, bit depth, frame alignment, and sample ranges
- Unit tests for successful encoding and invalid parameters
- WAV-to-FLAC and FLAC-to-FLAC round-trip tests

Remaining:

- Broader fixture coverage for WAV, FLAC, and AIFF conversion

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
