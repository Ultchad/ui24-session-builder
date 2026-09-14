# UiRecSession Schema

## File Name

```text
.uirecsession
```

## Format

```text
JSON
```

## Schema

```json
{
  "complete": true,
  "ext": ".flac",
  "files": [
    "Track Name"
  ],
  "lengthSamples": 16320000,
  "lengthSeconds": 340,
  "mapping": [
    "i.0"
  ],
  "names": [
    "Track Name"
  ],
  "sampleRate": 48000
}
```

## Field Requirements

### complete

Required: Yes

Type: Boolean

Description:

Indicates whether the session is complete.

Known values:

```json
true
```

---

### ext

Required: Yes

Type: String

Description:

File extension shared by all audio tracks.

Known values:

```json
".flac"
```

---

### files

Required: Yes

Type:

```json
string[]
```

Description:

Track filenames without extension.

Order appears important.

Array length must match:

- names
- mapping

---

### names

Required: Yes

Type:

```json
string[]
```

Description:

Display names shown by the Ui24R.

Currently identical to files.

Keep both fields independent.

---

### mapping

Required: Yes

Type:

```json
string[]
```

Description:

Track routing assignment.

Observed values:

```json
"i.0"
"i.1"
...
"i.21"
```

Maximum observed value:

```json
"i.21"
```

Corresponding to:

22 tracks

---

### lengthSamples

Required: Yes

Type:

```json
integer
```

Description:

Session length expressed in samples.

Formula:

```text
lengthSamples = sampleRate × lengthSeconds
```

---

### lengthSeconds

Required: Yes

Type:

```json
integer
```

Description:

Session duration in seconds.

---

### sampleRate

Required: Yes

Type:

```json
integer
```

Known values:

```json
48000
```

Additional rates require validation.