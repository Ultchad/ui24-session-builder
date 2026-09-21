const input = document.querySelector("#file-input");
const status = document.querySelector("#status");
const result = document.querySelector("#result-content");
const dropzone = document.querySelector("#dropzone");
const resultsPanel = document.querySelector(".results");
let wasmBridge = null;
const workspace = {
  session: null,
  files: [],
  originalFiles: [],
  fileGroups: [],
  stereoGroups: [],
  trackMetadata: new Map(),
  warnings: [],
  outputFormat: "flac"
};

function setResultsState(isEmpty) {
  if (!resultsPanel) return;
  resultsPanel.classList.toggle("is-empty", isEmpty);
  resultsPanel.hidden = isEmpty;
}

setResultsState(true);

function resetToEmptyState() {
  workspace.session = null;
  workspace.files = [];
  workspace.originalFiles = [];
  workspace.fileGroups = [];
  workspace.stereoGroups = [];
  workspace.trackMetadata = new Map();
  workspace.warnings = [];
  setResultsState(true);
  if (result) {
    result.innerHTML = '<div class="empty-state">Select or drop files to build a local session.</div>';
  }
  setStatus("Waiting for a file");
}

function setStatus(message, busy = false) {
  if (!status) return;
  status.textContent = message;
  status.classList.toggle("is-busy", busy);
  status.setAttribute("aria-busy", String(busy));
}

input.addEventListener("change", () => handleFiles(input.files));
dropzone.addEventListener("dragover", (event) => {
  event.preventDefault();
  dropzone.classList.add("is-dragging");
});
dropzone.addEventListener("dragleave", () => dropzone.classList.remove("is-dragging"));
dropzone.addEventListener("drop", (event) => {
  event.preventDefault();
  dropzone.classList.remove("is-dragging");
  handleFiles(event.dataTransfer.files);
});

async function handleFiles(fileList) {
  const files = Array.from(fileList);
  if (!files.length) return;
  setStatus("Reading locally", true);
  setResultsState(false);
  try {
    const sessionFile = files.find((file) => isSessionFile(file.name));
    if (sessionFile) {
      workspace.session = JSON.parse(await sessionFile.text());
      workspace.files = [];
      workspace.originalFiles = [];
      workspace.trackMetadata = new Map();
      workspace.warnings = [];
      renderSession(workspace.session, sessionFile.name);
    } else {
      workspace.session = null;
      workspace.warnings = [];
      const audioFiles = files.filter(isAudioFile);
      workspace.originalFiles = [...audioFiles];
      workspace.fileGroups = [];
      workspace.stereoGroups = [];
      setStatus("Analyzing audio", true);
      const processed = await processAudioInputs(audioFiles);
      workspace.files = processed;
      renderAudioFiles();
    }
    setStatus("Ready");
  } catch (error) {
    setStatus("Invalid file");
    result.innerHTML = `<div class="error">${escapeHtml(error.message)}</div>`;
  }
}

function renderSession(session, filename) {
  setResultsState(false);
  result.classList.remove("empty-state");
  const files = Array.isArray(session.files) ? session.files : [];
  const mappings = Array.isArray(session.mapping) ? session.mapping : [];
  const names = Array.isArray(session.names) ? session.names : files;
  if (typeof session.sampleRate !== "number" || !Array.isArray(session.mapping)) {
    throw new Error("This JSON does not look like a Ui24R session.");
  }
  workspace.session = { ...session, files: [...files], names: [...names], mapping: [...mappings] };
  result.innerHTML = `
    <div class="summary">
      ${metric("File", filename)}
      ${metric("Tracks", files.length)}
      ${metric("Sample rate", `${session.sampleRate} Hz`)}
      ${metric("Duration", `${session.lengthSeconds ?? "?"} s`)}
    </div>
    ${editableTracks()}
    ${actions()}`;
  bindEditor();
}

function renderAudioFiles() {
  setResultsState(false);
  result.classList.remove("empty-state");
  const files = workspace.files;
  const summary = computeAudioSummary();
  const extensionLabel = `.${workspace.outputFormat}`;
  result.innerHTML = `
    <div class="summary">
      ${metric("Files", files.length)}
      ${metric("Sample rate", summary.sampleRateMismatch ? "Mismatched" : `${summary.sampleRate} Hz`)}
      ${metric("Duration", `${summary.durationSeconds} s`)}
      ${metric("Extension", extensionLabel)}
    </div>
    ${formatSelector()}
    ${workspace.warnings.length ? `<div class="warnings">${workspace.warnings.map((warning) => `<p>${escapeHtml(warning)}</p>`).join("")}</div>` : ""}
    ${editableTracks()}
    ${actions()}`;
  bindEditor();
}

function formatSelector() {
  const selected = workspace.outputFormat;
  const flacEnabled = true;
  return `<label class="format-select">
    <span>Destination format</span>
    <select id="output-format">
      <option value="flac" ${selected === "flac" ? "selected" : ""} ${flacEnabled ? "" : "disabled"}>FLAC (WASM)</option>
      <option value="wav" ${selected === "wav" ? "selected" : ""}>WAV</option>
      <option value="mp3" ${selected === "mp3" ? "selected" : ""}>MP3 (320 kbps)</option>
    </select>
  </label>
  <p class="privacy">Displayed extensions are the selected export projection; conversion runs when the ZIP is created. FLAC, WAV, and MP3 at 320 kbps are encoded locally.</p>`;
}

function editableTracks() {
  const tracks = workspace.session ? workspace.session.files : workspace.files.map(projectedFileName);
  const names = workspace.session?.names ?? tracks.map(stripExtension);
  const mappings = workspace.session?.mapping ?? tracks.map((_, index) => `i.${index}`);
  return `<div class="track-list" role="table" aria-label="Session tracks">
    <div class="track track-header" role="row">
      <span role="columnheader">#</span>
      <span role="columnheader">Filename</span>
      <span role="columnheader">Track name</span>
      <span role="columnheader">Mapping</span>
      <span role="columnheader">Action</span>
    </div>
    ${tracks.map((name, index) => `
    <div class="track" data-index="${index}">
      <span class="track-index">${String(index + 1).padStart(2, "0")}</span>
      <input class="track-filename" value="${escapeAttribute(name)}" aria-label="Track ${index + 1} filename" readonly>
      <input class="track-name" data-field="name" value="${escapeAttribute(names[index] ?? name)}" aria-label="Track ${index + 1} name">
      <label class="mapping-input"><span aria-hidden="true">i.</span><input class="track-channel" data-field="mapping" type="number" min="0" max="21" step="1" value="${escapeAttribute(mappingNumber(mappings[index], index))}" aria-label="Track ${index + 1} mapping number"></label>
      <div class="track-actions">
        ${stereoActionForIndex(index)}
        <button class="remove-track" type="button" title="Remove track" data-remove="${index}">Remove</button>
      </div>
    </div>`).join("")}</div>`;
}

function stereoActionForIndex(index) {
  if (workspace.session) return "";
  const file = workspace.files[index];
  const groupIndex = workspace.stereoGroups.findIndex((group) => group.files.includes(file));
  if (groupIndex < 0) return "";
  const group = workspace.stereoGroups[groupIndex];
  const label = group.mode === "downmix" ? "Split stereo" : "Downmix mono";
  return `<button class="stereo-action" type="button" title="${label} ${escapeAttribute(group.sourceName)}" data-stereo-action="${groupIndex}">${label}</button>`;
}

function actions() {
  const canPackage = workspace.files.length > 0;
  return `<div class="actions">
    <button id="download-session" type="button">Download .uirecsession</button>
    ${canPackage ? '<button id="download-package" type="button">Download session .zip</button>' : ""}
    <button id="clear-workspace" class="secondary" type="button">Clear</button>
  </div>`;
}

function bindEditor() {
  result.querySelectorAll("[data-field]").forEach((field) => field.addEventListener("input", updateSessionFromEditor));
  result.querySelectorAll("[data-remove]").forEach((button) => button.addEventListener("click", () => {
    const index = Number(button.dataset.remove);
    const isRawAudio = workspace.files.length > 0;
    if (isRawAudio) {
      const [removed] = workspace.files.splice(index, 1);
      const containingGroup = workspace.fileGroups.find((fileGroup) => fileGroup.files.includes(removed));
      if (containingGroup) {
        containingGroup.files = containingGroup.files.filter((file) => file !== removed);
      }
      workspace.trackMetadata.delete(removed);
    }
    if (workspace.session) {
      workspace.session.files.splice(index, 1);
      workspace.session.names.splice(index, 1);
      workspace.session.mapping.splice(index, 1);
    }
    isRawAudio ? renderAudioFiles() : renderSession(workspace.session, "edited session");
  }));
  result.querySelector("#download-session").addEventListener("click", downloadSession);
  const packageButton = result.querySelector("#download-package");
  if (packageButton) packageButton.addEventListener("click", downloadSessionPackage);
  const formatField = result.querySelector("#output-format");
  if (formatField) formatField.addEventListener("change", async (event) => {
    workspace.outputFormat = event.target.value;
    renderAudioFiles();
    setStatus("Ready");
  });
  result.querySelectorAll("[data-stereo-action]").forEach((button) => button.addEventListener("click", () => {
    const group = workspace.stereoGroups[Number(button.dataset.stereoAction)];
    if (group) applyStereoMode(Number(button.dataset.stereoAction), group.mode === "downmix" ? "split" : "downmix");
  }));
  result.querySelector("#clear-workspace").addEventListener("click", () => {
    resetToEmptyState();
  });
}

function applyStereoMode(groupIndex, mode) {
  const group = workspace.stereoGroups[groupIndex];
  if (!group) return;
  group.mode = mode;
  group.files = mode === "downmix" ? [createDownmixFile(group)] : createSplitFiles(group);
  for (const file of group.files) {
    workspace.trackMetadata.set(file, {
      sampleRate: group.sampleRate,
      durationSamples: group.durationSamples,
      ext: ".wav"
    });
  }
  const fileGroup = workspace.fileGroups.find((candidate) => candidate.group === group);
  if (fileGroup) fileGroup.files = group.files;
  workspace.files = workspace.fileGroups.flatMap((candidate) => candidate.files);
  renderAudioFiles();
  setStatus("Ready");
}

function updateSessionFromEditor() {
  const names = [...result.querySelectorAll('[data-field="name"]')].map((field) => field.value);
  const mapping = [...result.querySelectorAll('[data-field="mapping"]')].map((field, index) => {
    const value = Number.parseInt(field.value, 10);
    if (!Number.isInteger(value) || value < 0 || value > 21) {
      field.value = String(index);
      return `i.${index}`;
    }
    return `i.${value}`;
  });
  if (workspace.files.length > 0) {
    // Raw-audio workspace: recompute files/ext/sampleRate/duration from the
    // current file list on every edit, so a removed track disappears from
    // the exported session and ZIP package instead of leaving stale data.
    const summary = computeAudioSummary();
    workspace.session = {
      complete: false,
      ext: `.${workspace.outputFormat}`,
      files: workspace.files.map((file) => stripExtension(projectedFileName(file))),
      names,
      mapping,
      sampleRate: summary.sampleRate,
      lengthSamples: summary.durationSamples,
      lengthSeconds: summary.durationSeconds
    };
  } else if (workspace.session) {
    workspace.session.names = names;
    workspace.session.mapping = mapping;
  }
}

function downloadSession() {
  updateSessionFromEditor();
  const blob = new Blob([JSON.stringify(workspace.session, null, 2)], { type: "application/json" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = ".uirecsession";
  link.click();
  URL.revokeObjectURL(link.href);
}

// Bundles the session JSON with the actual local audio bytes into a ZIP,
// so a usable session package can be produced fully client-side.
async function downloadSessionPackage() {
  setStatus("Preparing export", true);
  try {
    updateSessionFromEditor();
    const exportFiles = await prepareExportFiles();
    const entries = [];
    for (const file of exportFiles) {
      entries.push({ name: file.name, data: new Uint8Array(await file.arrayBuffer()) });
    }
    entries.push({
      name: ".uirecsession",
      data: new TextEncoder().encode(JSON.stringify(workspace.session, null, 2))
    });
    const blob = buildZip(entries);
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = "session.zip";
    link.click();
    URL.revokeObjectURL(link.href);
    setStatus("Ready");
  } catch (error) {
    setStatus("Export failed");
    workspace.warnings.push(error.message);
    renderAudioFiles();
  }
}

async function prepareExportFiles() {
  const files = workspace.files;
  const exportFiles = [];
  for (let index = 0; index < files.length; index += 1) {
    setStatus(`Converting ${index + 1}/${files.length}`, true);
    const [exportFile] = await convertFileForExport(files[index]);
    exportFiles.push(exportFile);
  }
  return exportFiles;
}

async function convertFileForExport(file) {
  const targetExtension = `.${workspace.outputFormat}`;
  if (extensionOf(file.name) === targetExtension) return [file];
  const metadata = workspace.trackMetadata.get(file);
  if (workspace.outputFormat === "wav") {
    const decoded = await decodeForExport(file);
    const channels = [];
    for (let channel = 0; channel < decoded.buffer.numberOfChannels; channel += 1) {
      channels.push(decoded.buffer.getChannelData(channel));
    }
    return [new File([encodeWavPcm(channels, decoded.sampleRate)], `${stripExtension(file.name)}.wav`, { type: "audio/wav" })];
  }
  if (workspace.outputFormat === "flac") {
    const bridge = await ensureWasmBridge();
    if (!bridge || typeof bridge.convert_audio_to_flac_bytes !== "function") {
      throw new Error("The browser FLAC bridge could not be loaded.");
    }
    const bytes = bridge.convert_audio_to_flac_bytes(new Uint8Array(await file.arrayBuffer()), extensionOf(file.name));
    if (!bytes.length) throw new Error(`FLAC conversion failed for ${file.name}.`);
    return [new File([bytes], `${stripExtension(file.name)}.flac`, { type: "audio/flac" })];
  }
  if (workspace.outputFormat === "mp3") {
    const bridge = await ensureWasmBridge();
    if (!bridge || typeof bridge.convert_audio_to_mp3_bytes !== "function") {
      throw new Error("The browser MP3 bridge could not be loaded.");
    }
    const bytes = bridge.convert_audio_to_mp3_bytes(new Uint8Array(await file.arrayBuffer()), extensionOf(file.name));
    if (!bytes.length) throw new Error(`MP3 conversion failed for ${file.name}.`);
    return [new File([bytes], `${stripExtension(file.name)}.mp3`, { type: "audio/mpeg" })];
  }
  return [file];
}

async function decodeForExport(file) {
  const AudioContextClass = window.AudioContext || window.webkitAudioContext;
  if (!AudioContextClass) throw new Error(`Cannot convert ${file.name}: Web Audio is unavailable.`);
  const context = new AudioContextClass();
  try {
    const buffer = await context.decodeAudioData(await file.arrayBuffer());
    return { buffer, sampleRate: buffer.sampleRate };
  } finally {
    await context.close();
  }
}

// Minimal store-only (uncompressed) ZIP writer: no external dependency and
// no Node.js build step, matching the rest of this static Web app.
function buildZip(entries) {
  const encoder = new TextEncoder();
  const localParts = [];
  const centralParts = [];
  let offset = 0;

  for (const entry of entries) {
    const nameBytes = encoder.encode(entry.name);
    const crc = crc32(entry.data);
    const size = entry.data.length;

    const localHeader = new DataView(new ArrayBuffer(30));
    localHeader.setUint32(0, 0x04034b50, true);
    localHeader.setUint16(6, 0, true);
    localHeader.setUint16(8, 0, true);
    localHeader.setUint32(14, crc, true);
    localHeader.setUint32(18, size, true);
    localHeader.setUint32(22, size, true);
    localHeader.setUint16(26, nameBytes.length, true);
    localParts.push(new Uint8Array(localHeader.buffer), nameBytes, entry.data);

    const centralHeader = new DataView(new ArrayBuffer(46));
    centralHeader.setUint32(0, 0x02014b50, true);
    centralHeader.setUint16(6, 20, true);
    centralHeader.setUint32(16, crc, true);
    centralHeader.setUint32(20, size, true);
    centralHeader.setUint32(24, size, true);
    centralHeader.setUint16(28, nameBytes.length, true);
    centralHeader.setUint32(42, offset, true);
    centralParts.push(new Uint8Array(centralHeader.buffer), nameBytes);

    offset += localHeader.byteLength + nameBytes.length + size;
  }

  const centralStart = offset;
  const centralSize = centralParts.reduce((sum, part) => sum + part.length, 0);

  const endRecord = new DataView(new ArrayBuffer(22));
  endRecord.setUint32(0, 0x06054b50, true);
  endRecord.setUint16(8, entries.length, true);
  endRecord.setUint16(10, entries.length, true);
  endRecord.setUint32(12, centralSize, true);
  endRecord.setUint32(16, centralStart, true);

  return new Blob([...localParts, ...centralParts, new Uint8Array(endRecord.buffer)], { type: "application/zip" });
}

let crcTable;
function crc32(data) {
  if (!crcTable) {
    crcTable = new Uint32Array(256);
    for (let n = 0; n < 256; n += 1) {
      let value = n;
      for (let k = 0; k < 8; k += 1) value = value & 1 ? (0xedb88320 ^ (value >>> 1)) : value >>> 1;
      crcTable[n] = value >>> 0;
    }
  }
  let crc = 0xffffffff;
  for (let index = 0; index < data.length; index += 1) crc = crcTable[(crc ^ data[index]) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

function isSessionFile(name) {
  return name.toLowerCase().endsWith(".uirecsession") || name.toLowerCase().endsWith(".json");
}

function isAudioFile(file) {
  return /\.(wav|flac|aif|aiff|mp3)$/i.test(file.name);
}

function stripExtension(name) {
  return name.replace(/\.[^.]+$/, "");
}

function projectedFileName(file) {
  return `${stripExtension(file.name)}.${workspace.outputFormat}`;
}

function mappingNumber(mapping, fallback) {
  const value = String(mapping ?? "").replace(/^i\./, "");
  return /^\d+$/.test(value) ? value : String(fallback);
}

function extensionOf(name) {
  const match = /\.[^.]+$/.exec(name);
  return match ? match[0].toLowerCase() : "";
}

async function ensureWasmBridge() {
  if (wasmBridge) return wasmBridge;
  try {
    const module = await import("./wasm/ui24_audio_processing.js");
    await module.default();
    wasmBridge = module;
    return module;
  } catch (error) {
    console.warn("WASM FLAC bridge unavailable; the original file will be kept.", error);
    return null;
  }
}

// Decodes each file with the Web Audio API to get sample rate and duration.
// Stereo files can be split into two mono WAV files or downmixed to one mono
// WAV track. Output conversion is intentionally deferred until export.
async function processAudioInputs(files) {
  if (!files.length) return [];
  const AudioContextClass = window.AudioContext || window.webkitAudioContext;
  if (!AudioContextClass) {
    for (const file of files) {
      workspace.trackMetadata.set(file, { sampleRate: null, durationSamples: 0, ext: extensionOf(file.name) });
    }
    workspace.warnings.push("This browser cannot decode audio; duration and channel checks are unavailable.");
    return files;
  }

  const context = new AudioContextClass();
  const output = [];
  workspace.fileGroups = [];
  workspace.stereoGroups = [];
  for (const file of files) {
    const extension = extensionOf(file.name);
    let buffer;
    try {
      buffer = await context.decodeAudioData(await file.arrayBuffer());
    } catch {
      workspace.trackMetadata.set(file, { sampleRate: null, durationSamples: 0, ext: extension });
      workspace.warnings.push(`Could not decode ${file.name} in the browser; duration is unavailable.`);
      workspace.fileGroups.push({ files: [file], stereo: false });
      output.push(file);
      continue;
    }

    const { sampleRate, length: durationSamples, numberOfChannels } = buffer;
    if (numberOfChannels >= 2) {
      const left = buffer.getChannelData(0);
      const right = buffer.getChannelData(1);
      const group = {
        sourceName: file.name,
        baseName: stripExtension(file.name),
        sourceFile: file,
        left,
        right,
        sampleRate,
        durationSamples,
        identical: channelsAreIdentical(left, right),
        mode: channelsAreIdentical(left, right) ? "downmix" : "split",
        files: []
      };
      group.files = group.mode === "downmix" ? [createDownmixFile(group)] : createSplitFiles(group);
      workspace.stereoGroups.push(group);
      workspace.fileGroups.push({ files: group.files, stereo: true, group });
      for (const stereoFile of group.files) {
        workspace.trackMetadata.set(stereoFile, { sampleRate, durationSamples, ext: ".wav" });
      }
      if (!group.identical) {
        workspace.warnings.push(`${file.name} has different left/right channels; choose split or downmix in stereo handling.`);
      } else {
        workspace.warnings.push(`${file.name} has identical left/right channels; downmix mono is selected by default.`);
      }
      output.push(...group.files);
      continue;
    }

    workspace.trackMetadata.set(file, { sampleRate, durationSamples, ext: extension });
    workspace.fileGroups.push({ files: [file], stereo: false });
    output.push(file);
  }
  await context.close();
  return output;
}

function createSplitFiles(group) {
  return [
    new File([encodeWavPcm([group.left], group.sampleRate)], `${group.baseName} L.wav`, { type: "audio/wav" }),
    new File([encodeWavPcm([group.right], group.sampleRate)], `${group.baseName} R.wav`, { type: "audio/wav" })
  ];
}

function createDownmixFile(group) {
  const mono = new Float32Array(group.left.length);
  for (let index = 0; index < mono.length; index += 1) {
    mono[index] = (group.left[index] + group.right[index]) / 2;
  }
  return new File([encodeWavPcm([mono], group.sampleRate)], `${group.baseName} mono.wav`, { type: "audio/wav" });
}

function channelsAreIdentical(left, right, epsilon = 1e-4) {
  if (left.length !== right.length) return false;
  for (let index = 0; index < left.length; index += 1) {
    if (Math.abs(left[index] - right[index]) > epsilon) return false;
  }
  return true;
}

// Encodes one or more interleaved Float32Array PCM channels as a canonical
// 16-bit WAV file. Used both for stereo-split halves (one channel each) and
// for whole-file WAV normalization (mono or multiple channels).
function encodeWavPcm(channels, sampleRate) {
  const channelCount = channels.length;
  const frameCount = channels[0].length;
  const dataSize = frameCount * channelCount * 2;
  const buffer = new ArrayBuffer(44 + dataSize);
  const view = new DataView(buffer);
  writeWavString(view, 0, "RIFF");
  view.setUint32(4, 36 + dataSize, true);
  writeWavString(view, 8, "WAVE");
  writeWavString(view, 12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, channelCount, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * channelCount * 2, true);
  view.setUint16(32, channelCount * 2, true);
  view.setUint16(34, 16, true);
  writeWavString(view, 36, "data");
  view.setUint32(40, dataSize, true);
  let offset = 44;
  for (let frame = 0; frame < frameCount; frame += 1) {
    for (let channel = 0; channel < channelCount; channel += 1) {
      const clamped = Math.max(-1, Math.min(1, channels[channel][frame]));
      view.setInt16(offset, clamped < 0 ? clamped * 0x8000 : clamped * 0x7fff, true);
      offset += 2;
    }
  }
  return new Blob([buffer], { type: "audio/wav" });
}

function writeWavString(view, offset, text) {
  for (let index = 0; index < text.length; index += 1) view.setUint8(offset + index, text.charCodeAt(index));
}

function computeAudioSummary() {
  const entries = workspace.files.map(
    (file) => workspace.trackMetadata.get(file) ?? { sampleRate: null, durationSamples: 0, ext: extensionOf(file.name) }
  );
  const sampleRates = new Set(entries.map((entry) => entry.sampleRate).filter((rate) => rate != null));
  const extensions = new Set(entries.map((entry) => entry.ext).filter(Boolean));
  const sampleRate = sampleRates.size === 1 ? [...sampleRates][0] : 48000;
  const durationSamples = entries.reduce((max, entry) => Math.max(max, entry.durationSamples), 0);
  return {
    sampleRate,
    durationSamples,
    durationSeconds: sampleRate ? Math.round(durationSamples / sampleRate) : 0,
    extensions,
    sampleRateMismatch: sampleRates.size > 1
  };
}

function renderAudio(file) {
  const extension = file.name.includes(".") ? file.name.split(".").pop().toUpperCase() : "UNKNOWN";
  result.innerHTML = `
    <div class="summary">
      ${metric("File", file.name)}
      ${metric("Format", extension)}
      ${metric("Size", formatBytes(file.size))}
      ${metric("Browser", "Local only")}
    </div>
    <p class="privacy">Browser metadata decoding will use the shared Rust/WASM engine when the WebAssembly adapter is added.</p>`;
}

function metric(label, value) {
  return `<div class="metric"><small>${escapeHtml(label)}</small><strong>${escapeHtml(String(value))}</strong></div>`;
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  return `${(bytes / 1024).toFixed(1)} KB`;
}

function escapeHtml(value) {
  return value.replace(/[&<>'"]/g, (character) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;"
  })[character]);
}

function escapeAttribute(value) {
  return escapeHtml(String(value));
}
