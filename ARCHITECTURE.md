# Architecture Overview

## Purpose

This document describes the technical architecture of UI24 Session Builder.

Its goal is to provide a long-term reference for contributors and AI coding assistants.

---

# Design Objectives

The architecture must provide:

- Cross-platform compatibility
- Shared business logic
- Offline operation
- High maintainability
- Clear separation of responsibilities
- Easy testing
- Long-term scalability

---

# Architectural Principles

## Single Source Of Truth

Business logic must exist only once.

Implementations must not be duplicated across platforms.

---

## Offline First

The application must fully operate without:

- Internet connection
- Cloud service
- External APIs
- User accounts

---

## Platform Independence

The business layer must not depend on:

- Flutter
- React
- Browser APIs
- Operating system APIs

Platform-specific code belongs only to platform applications.

---

## Documentation First

Documentation drives implementation.

If functionality is not documented, it should not be implemented.

---

# System Overview

```text
┌─────────────────────────────────────────┐
│              User Interface             │
└─────────────────────────────────────────┘

          ┌─────────┬─────────┬─────────┐
          │         │         │         │

          ▼         ▼         ▼

      CLI App    Web App   Flutter App

          │         │         │

          └─────────┴─────────┘

                    │

                    ▼

        UI24 Core Business Library

                    │

    ┌───────────────┼───────────────┐
    │               │               │

    ▼               ▼               ▼

Audio Processing  Session Logic  Localization

                    │

                    ▼

             File System Layer
```

---

# Repository Structure

```text
ui24-session-builder/

├── applications/
│
│   ├── cli/
│   ├── web/
│   └── flutter/
│
├── libraries/
│
│   ├── ui24_core/
│   ├── audio_processing/
│   ├── session_generator/
│   ├── localization/
│   └── file_system/
│
├── documentation/
│
├── specifications/
│
├── tests/
│
└── tools/
```

---

# Library Responsibilities

## ui24_core

Primary business layer.

Responsibilities:

- Session creation
- Session validation
- Workflow orchestration
- Shared domain models

Contains no UI code.

---

## audio_processing

Responsibilities:

- Metadata extraction
- Audio validation
- FLAC conversion
- Audio format abstraction

Contains no Ui24R-specific logic.

---

## session_generator

Responsibilities:

- Ui24R export generation
- Configuration serialization
- Session folder structure generation

Contains no UI code.

---

## localization

Responsibilities:

- Translation loading
- Language selection
- Placeholder handling

---

## file_system

Responsibilities:

- File creation
- File reading
- File writing
- Archive generation

---

# Applications

## CLI

Location:

```text
applications/cli
```

Responsibilities:

- Testing
- Automation
- Command-line workflows

Example:

```bash
ui24-session-builder create \
    --input ./tracks \
    --output ./session
```

---

## Web

Location:

```text
applications/web
```

Technology:

- React
- TypeScript
- Vite
- Rust WebAssembly

Responsibilities:

- Browser interface
- File selection
- Session export

No backend allowed.

Compatible with GitHub Pages.

---

## Flutter

Location:

```text
applications/flutter
```

Responsibilities:

- Mobile user interfaces
- Desktop user interfaces

Uses:

```text
flutter_rust_bridge
```

to reach business functions.

---

# Domain Model

## Session

```text
Session
├── Metadata
├── Tracks
├── Routing
└── Validation Report
```

---

## SessionMetadata

```text
Session Name
Sample Rate
Duration Seconds
Duration Samples
```

---

## SessionTrack

```text
Display Name
File Name
Audio Format
Metadata
Channel Assignment
```

---

## ChannelAssignment

Current observed values:

```text
i.0
i.1
i.2
...
i.21
```

Maximum known channel count:

```text
22
```

---

# Localization Architecture

User-visible text must never be hardcoded.

Example:

Bad:

```typescript
"Create Session"
```

Good:

```typescript
t("session.create")
```

Translation files:

```text
locales/

├── en.json
├── fr.json
└── es.json
```

---

# Testing Architecture

## Unit Tests

Required for:

- Metadata extraction
- Session validation
- Configuration generation
- Audio conversion

---

## Integration Tests

Required for:

- Folder generation
- Session export
- ZIP export

---

## Regression Tests

Every bug fix must include at least one regression test.

---

# Dependency Rules

Allowed:

```text
Application
    ↓
Core Library
    ↓
Supporting Libraries
```

Not Allowed:

```text
Core Library
    ↓
Application
```

Applications depend on libraries.

Libraries never depend on applications.

---

# Future Enhancements

Potential future capabilities:

- Batch session generation
- Session templates
- Drag and drop ordering
- Automatic channel assignment
- Session comparison

All future features must remain compatible with the existing architecture.

---

# Success Criteria

The architecture is successful when:

- The same business logic is used everywhere.
- A Ui24R accepts generated sessions.
- New platforms can be added without modifying core logic.
- Contributors can understand the system quickly.
- AI coding assistants can navigate the codebase without ambiguity.
