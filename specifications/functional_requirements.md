# Functional Requirements

## Audio Import

The system shall import audio files from local storage.

Supported formats:

- WAV
- FLAC
- AIFF

Future formats may be added.

---

## Audio Analysis

The system shall extract:

- sample rate
- bit depth
- channel count
- duration

---

## Session Creation

The system shall generate:

- FLAC files
- .uirecsession file
- valid folder structure

---

## Validation

The system shall prevent export if:

- no tracks exist
- more than 22 tracks exist
- unsupported formats are detected
- channel conflicts exist

---

## Export

The system shall support:

- folder export
- ZIP export
