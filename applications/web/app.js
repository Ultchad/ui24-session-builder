const input = document.querySelector("#file-input");
const status = document.querySelector("#status");
const result = document.querySelector("#result-content");

input.addEventListener("change", async () => {
  const file = input.files[0];
  if (!file) return;
  status.textContent = "Reading locally";
  try {
    if (file.name.toLowerCase().endsWith(".uirecsession") || file.name.toLowerCase().endsWith(".json")) {
      renderSession(JSON.parse(await file.text()), file.name);
    } else {
      renderAudio(file);
    }
    status.textContent = "Ready";
  } catch (error) {
    status.textContent = "Invalid file";
    result.innerHTML = `<div class="error">${escapeHtml(error.message)}</div>`;
  }
});

function renderSession(session, filename) {
  const files = Array.isArray(session.files) ? session.files : [];
  const mappings = Array.isArray(session.mapping) ? session.mapping : [];
  const names = Array.isArray(session.names) ? session.names : files;
  if (typeof session.sampleRate !== "number" || !Array.isArray(session.mapping)) {
    throw new Error("This JSON does not look like a Ui24R session.");
  }
  result.innerHTML = `
    <div class="summary">
      ${metric("File", filename)}
      ${metric("Tracks", files.length)}
      ${metric("Sample rate", `${session.sampleRate} Hz`)}
      ${metric("Duration", `${session.lengthSeconds ?? "?"} s`)}
    </div>
    <div class="track-list">
      ${files.map((name, index) => `
        <div class="track">
          <span class="track-index">${String(index + 1).padStart(2, "0")}</span>
          <span class="track-name">${escapeHtml(names[index] ?? name)}</span>
          <span class="track-channel">${escapeHtml(mappings[index] ?? "-")}</span>
        </div>`).join("")}
    </div>`;
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
