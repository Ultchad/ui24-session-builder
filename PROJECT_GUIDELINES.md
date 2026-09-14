# UI24 Session Builder

## Project Mission

UI24 Session Builder is an open-source application designed to create playback sessions compatible with the Soundcraft Ui24R digital mixer.

The application must allow users to:

- Import audio files.
- Analyze audio metadata.
- Convert source audio files to FLAC.
- Generate a valid Ui24R session.
- Export the session to a folder or ZIP archive.
- Validate compatibility before export.

The application must work entirely offline.

No cloud service, server component, external API, analytics service, telemetry system or account management system may be required.

All processing must occur locally on the user's device.

---

# Core Principles

## Simplicity

Prefer simple and readable code over clever code.

## Maintainability

The project should remain understandable by:

- audio engineers
- system administrators
- hobby developers
- AI coding assistants

## Explicitness

Everything should be clearly named.

Avoid abbreviations unless they are industry standards.

## Documentation First

No implementation should be added without documentation.

Documentation is the source of truth.

---

# Technology Stack

## Core Library

Language:

Rust

Responsibilities:

- audio metadata extraction
- session model
- validation
- FLAC generation
- Ui24R export generation

The core library contains all business logic.

---

## CLI

Language:

Rust

Framework:

Clap

Purpose:

- testing
- automation
- power users
- CI validation

---

## Web

Technology:

- React
- TypeScript
- Vite
- Rust WebAssembly

Requirements:

- Fully client-side
- No backend
- Compatible with GitHub Pages
- Offline capable

---

## Mobile/Desktop

Technology:

Flutter

Bridge:

flutter_rust_bridge

The Flutter application must consume the same Rust business library.

---

# Design Rules

## No Business Logic In User Interfaces

Business logic belongs only inside Rust libraries.

UI layers must remain thin.

---

## No Duplicate Logic

A function must have a single implementation.

Example:

Audio metadata extraction must exist only once in the Rust core.

---

## No Magic Values

Bad:

```rust
if track_count > 22
```

Good:

```rust
const MAX_TRACK_COUNT: usize = 22;
```

---

# Source Code Language

All source code must use English.

Examples:

Good:

SessionTrack

Bad:

PisteAudio

---

# Comment Language

All comments must use English.

Example:

```rust
// Verify all channel assignments are unique.
```

---

# Documentation Language

Internal documentation:

English

User documentation:

English and French

Spanish translation will be added later.

---

# Localization

All user-visible text must be translatable.

Never hardcode interface strings.

Bad:

```typescript
<button>Create Session</button>
```

Good:

```typescript
<button>{t("session.create")}</button>
```

Directory structure:

```text
locales/

├─ en.json
├─ fr.json
└─ es.json
```

Initially:

- English completed
- French placeholders
- Spanish placeholders

---

# Audio Rules

The application must never modify source files.

Input files are read-only.

Generated files must be written into a separate output location.

---

# Error Handling

No silent failures.

Every error message should explain:

- what happened
- why it happened
- possible corrective actions

---

# Logging

Supported levels:

- ERROR
- WARN
- INFO
- DEBUG
- TRACE

Sensitive user data must never be logged.

---

# Maximum Ui24R Capacity

Known limit:

22 tracks

The value must be controlled through a dedicated constant.

---

# Reverse Engineering Requirements

The Soundcraft-generated session format is considered the reference implementation.

Whenever a new format detail is discovered:

1. Update documentation.
2. Add tests.
3. Update implementation.

Never implement assumptions without documentation.

---

# Definition Of Done

A feature is complete only if:

- implemented
- tested
- documented
- reviewed

All four conditions are mandatory.
