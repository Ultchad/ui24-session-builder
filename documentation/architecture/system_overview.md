# System Overview

## Purpose

UI24 Session Builder is an open-source application that generates playback sessions compatible with the Soundcraft Ui24R digital mixer.

The application is designed to be:

- Cross-platform
- Offline-first
- Easy to maintain
- Easy to understand
- Open-source friendly

All business logic is shared across every platform.

---

# System Components

The project is composed of four major layers:

1. User Applications
2. Business Logic
3. Supporting Libraries
4. File System Layer

---

# High Level Architecture

```text
+----------------------+
|      CLI App         |
+----------------------+

+----------------------+
|      Web App         |
+----------------------+

+----------------------+
|    Flutter App       |
+----------------------+
            |
            v
+----------------------+
|      ui24_core       |
+----------------------+
            |
            v
+----------------------+     +----------------------+
|  audio_processing    |     |  session_generator   |
+----------------------+     +----------------------+
            |
            v
+----------------------+
|     localization     |
+----------------------+
            |
            v
+----------------------+
|     file_system      |
+----------------------+

Applications Layer

The Applications Layer contains all user interfaces.

Applications must never contain business rules.

Applications may:

Display information
Collect user input
Display errors
Display progress

Applications must not:

Analyze audio
Generate sessions
Convert files
Implement validation rules

Those responsibilities belong to the business layer.

CLI Application

Location:

applications/cli


Responsibilities:

Batch processing
Testing
Automation
Power users
Web Application

Location:

applications/web


Technology:

React
TypeScript
Vite
Rust WebAssembly

Responsibilities:

Browser-based interface
File selection
Session export
Drag and drop workflow

Requirements:

Fully offline
No backend
Compatible with GitHub Pages
Flutter Application

Location:

applications/flutter


Responsibilities:

Mobile user interfaces
Desktop user interfaces

Supported targets:

Android
iOS
Windows
Linux
macOS

Communication with Rust occurs through:

flutter_rust_bridge

Business Layer

The business layer contains all project logic.

Location:

libraries/ui24_core


Responsibilities:

Session creation
Session validation
Export workflow management
Domain models

The business layer must remain platform independent.

Supporting Libraries

Supporting libraries provide specialized functionality.

audio_processing

Responsibilities:

Audio metadata extraction
FLAC conversion
Audio validation
Audio format abstraction

Contains no Ui24R-specific logic.

session_generator

Responsibilities:

Generate .uirecsession files
Generate session folder structures
Serialize configuration data

Contains no UI code.

localization

Responsibilities:

Translation loading
Language selection
Translation lookup

All user-facing strings must pass through this layer.

file_system

Responsibilities:

File creation
File reading
File writing
ZIP archive generation

This layer isolates platform-specific storage operations.

Domain Model

The core domain is centered around a playback session.

Session
├── Metadata
├── Tracks
├── Routing
└── ValidationReport

SessionMetadata

Contains:

Session name
Sample rate
Duration in seconds
Duration in samples
SessionTrack

Contains:

Display name
File name
Audio format
Audio metadata
Channel assignment
ChannelAssignment

Observed values:

i.0
i.1
i.2
...
i.21


Known maximum:

22 tracks

Design Goals

The architecture must allow:

Adding new interfaces without changing business logic
Adding new audio formats without changing user interfaces
Supporting future platforms
Long-term maintenance
Easy onboarding of contributors
Efficient AI-assisted development
Success Criteria

The architecture is successful when:

Business logic exists only once.
All platforms produce identical outputs.
The generated session is accepted by a Soundcraft Ui24R.
New contributors can quickly understand the codebase.
