# Testing Strategy

This document describes the testing conventions already in effect across the
Rust workspace, the CLI, and the static Web application.

## Test ownership

- **Unit tests** live beside the code they cover, in a `#[cfg(test)] mod
  tests` block at the bottom of the same source file (see
  `libraries/audio_processing/src/flac_encoder.rs`,
  `libraries/audio_processing/src/symphonia_reader.rs`,
  `libraries/ui24_core/src/validation.rs`). They cover a single function or
  type in isolation, including invalid-input and error-path cases.
- **Integration tests** live in `tests/integration/tests/*.rs` and in each
  library's own `tests/` directory when present (for example
  `libraries/session_generator`). They exercise the public API of one or
  more crates together, matching real call patterns (build a `Session`,
  validate it, generate a configuration or folder).
- **CLI tests** are currently manual, run against the fixtures in `tmp/`
  (see "Manual verification" below). Automated CLI integration tests can be
  added under `applications/cli/tests/` when CLI behavior stabilizes.

## What every test must do

- Use real, deterministic input data (synthetic PCM buffers, hand-built WAV
  or AIFF byte layouts, or checked-in fixtures). Do not depend on
  network access or system audio devices.
- Assert on the exact typed error variant when testing a failure path, not
  just `is_err()`, unless the error message itself is derived from a
  third-party library and cannot be matched precisely (for example
  Symphonia decode errors).
- Name tests as a full sentence describing the behavior under test, e.g.
  `rejects_more_than_maximum_tracks`, `writes_converted_wav_to_a_file`,
  `rejects_truncated_mp3_sources_without_panicking`.

## Fixture policy

- Small, synthetic fixtures (a few PCM samples encoded as a minimal WAV or
  AIFF byte buffer) are generated inline in the test module and are
  preferred whenever they are sufficient.
- Larger or real-world audio files (`tmp/wav/`, `tmp/mp3/`) are used for
  local manual verification and are excluded from version control via
  `.gitignore` (`tmp/`). Any automated test that depends on such a file must
  guard the test body with an existence check and return early when the
  file is absent, so the test suite still passes in a clean checkout or CI:

```rust
let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp/mp3/Cri_wilhelm.mp3");
if !fixture.is_file() {
    return;
}
```

  This pattern is used by
  `flac_encoder::tests::rejects_malformed_mp3_fixture_without_panicking` and
  `session_generator::tests::validates_official_ui24r_fixture_schema`.

## Required checks before every commit

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
git diff --check
```

For changes under `applications/web`:

```bash
docker run --rm -v "$PWD/applications/web:/app:ro" node:22-alpine node --check /app/app.js
docker compose -f docker-compose.web.yml up -d --build
```

## Regression tests

Every bug fix includes a regression test that fails before the fix and
passes after it, per `CONTRIBUTING.md`. Example: the malformed-MP3 decoder
panic was reproduced with the real `Cri_wilhelm.mp3` fixture before adding
`std::panic::catch_unwind`, and the same fixture is now asserted to return a
controlled error.

## Manual verification (outside the automated suite)

Some behavior cannot be verified by automated tests alone, because it
depends on real hardware, a real mobile browser, or the proprietary Ui24R
mixer. These checks are tracked in `specifications/roadmap.md` under
"Remaining" items and must be performed manually before a release:

- CLI `create`/`convert`/`analyze` against real multi-track WAV/MP3
  recordings on a development machine.
- The static Web UI in an actual desktop and mobile browser (drag-and-drop,
  file picker, downloads).
- Loading a CLI- or Web-generated session onto a real Ui24R mixer over its
  supported transport (SD card or USB) to confirm the mixer accepts the
  session and maps channels correctly.

See the project's manual test checklist shared with the user for the exact
steps expected for each of these environments.
