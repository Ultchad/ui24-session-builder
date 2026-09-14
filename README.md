# UI24 Session Builder

Open-source cross-platform session generator for the Soundcraft Ui24R.

## Overview

UI24 Session Builder is a fully offline application that creates playback sessions compatible with the Soundcraft Ui24R digital mixer.

The project is intended to replace the Windows-only "UI Session Maker" application while providing support for:

- Linux
- Windows
- macOS
- Web browsers
- Android
- iOS

The application works entirely on the user's device.

No cloud services, accounts, subscriptions, telemetry, analytics, or external servers are required.

---

# Features

## Audio Import

Import supported audio files:

- WAV
- FLAC
- AIFF

Additional formats may be supported in future versions.

---

## Audio Analysis

Automatically analyze:

- Sample rate
- Bit depth
- Channel count
- Duration

---

## FLAC Conversion

Convert imported audio files to FLAC while preserving source characteristics whenever possible.

---

## Ui24R Session Generation

Generate:

- FLAC audio files
- Ui24R session configuration
- Valid folder structure

---

## Session Validation

Verify:

- Track count
- Sample rate consistency
- Channel assignments
- File integrity

before exporting.

---

## Export Options

Export as:

- Folder
- ZIP archive

---

# Project Goals

- Replace the official Windows-only tool
- Work on all major platforms
- Preserve compatibility with Ui24R
- Provide a modern user interface
- Remain usable offline
- Be easy to maintain
- Be friendly to open-source contributors

---

# Supported Platforms

| Platform | Status |
|-----------|---------|
| Linux | Planned |
| Windows | Planned |
| macOS | Planned |
| Web Browser | Planned |
| Android | Planned |
| iOS | Planned |

---

# Technology Stack

## Core Engine

Rust

Responsibilities:

- Audio processing
- Metadata extraction
- Session generation
- Validation

---

## Command Line Interface

Rust + Clap

---

## Web Application

- React
- TypeScript
- Vite
- WebAssembly (Rust)

---

## Desktop and Mobile

- Flutter
- flutter_rust_bridge

---

# Project Structure

```text
ui24-session-builder/

├── applications/
├── libraries/
├── documentation/
├── specifications/
├── tests/
└── tools/
```

See:

```text
ARCHITECTURE.md
```

for complete technical details.

---

# Documentation

Main documents:

- PROJECT_GUIDELINES.md
- ARCHITECTURE.md
- documentation/format_specifications/ui24r_session_format.md
- specifications/functional_requirements.md
- specifications/roadmap.md

---

# Current Reverse Engineering Status

The Ui24R session format has been partially documented from official UI Session Maker exports.

Current known characteristics:

- Session configuration stored in `.uirecsession`
- JSON format
- FLAC audio files
- Maximum 22 tracks

See:

```text
documentation/format_specifications/ui24r_session_format.md
```

---

# Development Philosophy

The project follows:

- Documentation First
- Offline First
- Single Source Of Truth
- Shared Business Logic
- Clean Architecture

Business logic is implemented only once inside the Rust core library.

Platform-specific applications merely expose the functionality.

---

# Localization

UI text will support:

- English
- French
- Spanish (future)

All user-facing strings must be translatable.

---

# Contributing

Please read:

```text
CONTRIBUTING.md
```

before submitting pull requests.

---

# License

Apache License Version 2.0
