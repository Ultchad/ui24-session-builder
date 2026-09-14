const input = document.querySelector("#file-input");
const status = document.querySelector("#status");
const result = document.querySelector("#result-content");
const dropzone = document.querySelector("#dropzone");
const workspace = { session: null, files: [] };

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
  status.textContent = "Reading locally";
  try {
    const sessionFile = files.find((file) => isSessionFile(file.name));
    if (sessionFile) {
      workspace.session = JSON.parse(await sessionFile.text());
      workspace.files = [];
      renderSession(workspace.session, sessionFile.name);
    } else {
      workspace.session = null;
      workspace.files.push(...files.filter(isAudioFile));
      renderAudioFiles();
    }
    status.textContent = "Ready";
  } catch (error) {
    status.textContent = "Invalid file";
    result.innerHTML = `<div class="error">${escapeHtml(error.message)}</div>`;
  }
}

function renderSession(session, filename) {
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
  const files = workspace.files;
  result.innerHTML = `
    <div class="summary">
      ${metric("Files", files.length)}
      ${metric("Audio", files.map((file) => file.name).join(", "))}
      ${metric("Mode", "New session")}
      ${metric("Browser", "Local only")}
    </div>
    ${editableTracks()}
    ${actions()}`;
  bindEditor();
}

function editableTracks() {
  const tracks = workspace.session ? workspace.session.files : workspace.files.map((file) => file.name);
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
      <button class="remove-track" type="button" title="Remove track" data-remove="${index}">Remove</button>
    </div>`).join("")}</div>`;
}

function actions() {
  return `<div class="actions"><button id="download-session" type="button">Download .uirecsession</button><button id="clear-workspace" class="secondary" type="button">Clear</button></div>`;
}

function bindEditor() {
  result.querySelectorAll("[data-field]").forEach((field) => field.addEventListener("input", updateSessionFromEditor));
  result.querySelectorAll("[data-remove]").forEach((button) => button.addEventListener("click", () => {
    const index = Number(button.dataset.remove);
    if (workspace.session) {
      workspace.session.files.splice(index, 1);
      workspace.session.names.splice(index, 1);
      workspace.session.mapping.splice(index, 1);
    } else {
      workspace.files.splice(index, 1);
    }
    workspace.session ? renderSession(workspace.session, "edited session") : renderAudioFiles();
  }));
  result.querySelector("#download-session").addEventListener("click", downloadSession);
  result.querySelector("#clear-workspace").addEventListener("click", () => {
    workspace.session = null;
    workspace.files = [];
    result.innerHTML = '<div class="empty-state">Select or drop files to build a local session.</div>';
    status.textContent = "Waiting for a file";
  });
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
  if (workspace.session) {
    workspace.session.names = names;
    workspace.session.mapping = mapping;
  } else {
    workspace.session = {
      complete: false,
      ext: ".flac",
      files: workspace.files.map((file) => stripExtension(file.name)),
      names,
      mapping,
      sampleRate: 48000,
      lengthSamples: 0,
      lengthSeconds: 0
    };
  }
}

function downloadSession() {
  updateSessionFromEditor();
  const blob = new Blob([JSON.stringify(workspace.session, null, 2)], { type: "application/json" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = "session.uirecsession";
  link.click();
  URL.revokeObjectURL(link.href);
}

function isSessionFile(name) {
  return name.toLowerCase().endsWith(".uirecsession") || name.toLowerCase().endsWith(".json");
}

function isAudioFile(file) {
  return /\.(wav|flac|aif|aiff)$/i.test(file.name);
}

function stripExtension(name) {
  return name.replace(/\.[^.]+$/, "");
}

function mappingNumber(mapping, fallback) {
  const value = String(mapping ?? "").replace(/^i\./, "");
  return /^\d+$/.test(value) ? value : String(fallback);
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
