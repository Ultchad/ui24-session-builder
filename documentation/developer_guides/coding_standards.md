# Coding Standards

These standards describe the conventions already enforced across the Rust
workspace, the CLI, and the static Web application. They follow the
principles in `PROJECT_GUIDELINES.md` (simplicity, maintainability,
explicitness, no magic values, English source and comments).

## Rust

### Edition and toolchain

- Rust edition `2021` across every crate.
- Stable channel only, pinned by `rust-toolchain.toml` with the `rust-src`,
  `rustfmt`, `clippy`, and `rust-analyzer` components.
- No `unsafe` code. None is currently used anywhere in the workspace, and new
  `unsafe` blocks require a documented, reviewed justification before merge.

### Formatting and linting

Every change must pass, from the repository root:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
```

Clippy warnings are treated as errors (`-D warnings`); do not silence a lint
with `#[allow(...)]` unless the lint is a false positive, and explain why in a
comment next to the attribute.

### Naming and API design

- Names are explicit and unabbreviated (`SessionTrack`, not `TrkS`), matching
  the "No abbreviations" rule in `PROJECT_GUIDELINES.md`.
- Numeric limits and fixed constants are named, not inlined
  (`MAX_TRACK_COUNT`, `MAX_INDEX`), per the "No Magic Values" rule.
- Public crate APIs return `Result<T, E>` with a crate-local, typed error enum
  (for example `AudioProcessingError`, `AudioConversionError`,
  `FlacEncodingError`, `SessionGenerationError`). Do not use `anyhow` or
  string-only errors in library crates; reserve free-form error strings for
  CLI-level reporting only.
- Error enums implement `Display` and `std::error::Error`, and derive
  `Clone, Debug, Eq, PartialEq` where the payload allows it, so tests can
  assert on exact error values.

### Dependency direction

- `audio_processing` never depends on `ui24_core`.
- `ui24_core` contains pure domain and validation logic, no file-system or
  platform APIs.
- `session_generator` depends on `ui24_core` and `audio_processing` only
  through their public APIs.
- Applications (`applications/cli`, `applications/web`) depend on the
  libraries, never the other way around.

See `documentation/architecture/dependency_rules.md` for the enforced graph.

### Robustness

- Decoding third-party or user-supplied audio data must never panic the
  process. Wrap third-party decoder calls that are known to panic on
  malformed input in `std::panic::catch_unwind` and convert the panic into a
  typed error, as done in `SymphoniaMetadataReader` and `FlacEncoder`.
- Validate inputs at the boundary (file parsing, CLI argument parsing) and
  keep internal domain functions free of redundant defensive checks for
  states that are already excluded by the type system.

### Comments and documentation

- Comments are written in English and explain *why*, not *what* the next
  line already shows.
- Public items that are not self-explanatory from their name get a short
  `///` doc comment; avoid multi-paragraph doc comments where one line is
  enough.

## Web application (`applications/web`)

- Plain HTML, CSS, and JavaScript only; no bundler, package manager, or
  Node.js build step, so the site can be published to GitHub Pages by
  copying the three static files.
- No external JavaScript libraries. Features such as ZIP packaging and audio
  decoding are implemented directly with browser-native APIs (Web Audio API,
  `Blob`, `DataView`) to keep the dependency-free, offline-first constraint
  from `PROJECT_GUIDELINES.md`.
- Every change to `app.js` or `styles.css` bumps the cache-busting query
  string (`?v=N`) referenced in `index.html`.
- Validate JavaScript syntax before committing:

```bash
docker run --rm -v "$PWD/applications/web:/app:ro" node:22-alpine node --check /app/app.js
```

## Commit and branch conventions

- Active development happens on the `dev` branch. Do not merge `dev` into
  `main` unless explicitly requested.
- Commit messages use an imperative summary line followed by a short bullet
  list of the concrete changes.
- Every commit that changes behavior updates the relevant documentation
  (`README.md`, `specifications/roadmap.md`) in the same commit.
