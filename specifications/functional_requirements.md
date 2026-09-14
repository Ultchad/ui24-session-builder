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

## Ui24R Session Generation

The system shall generate:

- FLAC audio files
- One `.uirecsession` file

The generated `.uirecsession` file shall conform to the documented Ui24R session schema.

Reference:

```text
documentation/format_specifications/ui24r_session_format.md
```
