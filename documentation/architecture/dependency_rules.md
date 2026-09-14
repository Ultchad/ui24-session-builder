# Dependency Rules

## Purpose

This document defines allowed and forbidden dependencies within the project.

Following these rules prevents architectural drift and keeps the project maintainable over time.

---

# Dependency Direction

Dependencies must always flow downward.

```text
Applications
    |
    v
ui24_core
    |
    v
Supporting Libraries


A lower layer must never depend on a higher layer.

Allowed Dependencies
Applications → Core

Allowed:

applications/cli
    -> ui24_core

applications/web
    -> ui24_core (WASM)

applications/flutter
    -> ui24_core


Applications consume business services.

Core → Supporting Libraries

Allowed:

ui24_core
    -> audio_processing

ui24_core
    -> session_generator

ui24_core
    -> localization

ui24_core
    -> file_system


Core orchestrates lower-level services.

Supporting Libraries

Allowed:

session_generator
    -> file_system

audio_processing
    -> file_system


Only when necessary.

Dependencies must remain minimal.

Forbidden Dependencies
Core → Applications

Forbidden:

ui24_core
    -> React

ui24_core
    -> Flutter

ui24_core
    -> CLI code


Business logic must remain platform independent.

Supporting Libraries → Applications

Forbidden:

audio_processing
    -> React

session_generator
    -> Flutter

localization
    -> CLI


Libraries must never depend on user interfaces.

Cross Application Dependencies

Forbidden:

web
    -> flutter

flutter
    -> cli

cli
    -> web


Applications are independent.

External Dependency Policy

Before adding a dependency, evaluate:

Is it actively maintained?
Is it widely used?
Is it cross-platform?
Is it compatible with open-source licensing?
Is it really necessary?

Prefer fewer dependencies.

Dependency Injection

Business logic should depend on abstractions rather than implementations whenever practical.

Example:

AudioFileReader (trait/interface)
        |
        +--> NativeFileReader
        +--> BrowserFileReader


This improves testing and platform portability.

File Ownership Rules

Each file should have a single responsibility.

Good examples:

session_exporter.rs
audio_metadata.rs
channel_assignment.rs


Avoid generic names.

Bad examples:

utils.rs
helpers.rs
common.rs
misc.rs
manager.rs


Unless there is a very strong justification.

Circular Dependencies

Circular dependencies are forbidden.

Bad:

ui24_core
    -> audio_processing

audio_processing
    -> ui24_core


Good:

ui24_core
    -> audio_processing


Only one direction.

Architecture Validation

When introducing new code, contributors must verify:

Dependency direction remains correct.
No duplicated business logic is introduced.
No UI code leaks into libraries.
No platform-specific logic leaks into core services.
The change remains compatible with all supported platforms.

If a feature violates these rules, the design must be revised before implementation.
