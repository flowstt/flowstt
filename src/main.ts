import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { initTheme, getResolvedTheme, onThemeChange } from "./theme";

import logoLight from "./assets/flowstt-landscape-light.svg";
import logoDark from "./assets/flowstt-landscape.svg";

// Startup timing
const JS_MODULE_LOAD_TIME = performance.now();
function startupLog(msg: string) {
  invoke("startup_log", { message: msg });
}
startupLog(`JS module evaluated at ${JS_MODULE_LOAD_TIME.toFixed(0)}ms after page origin`);

function isDebugConsoleHotkey(e: KeyboardEvent): boolean {
  const isIKey = e.code === "KeyI" || e.key === "i" || e.key === "I";
  const isCtrlShift = e.ctrlKey && e.shiftKey && !e.altKey && !e.metaKey;
  const isMetaAlt = e.metaKey && e.altKey && !e.ctrlKey && !e.shiftKey;
  return isIKey && (isCtrlShift || isMetaAlt);
}

interface ModelStatus {
  available: boolean;
  path: string;
}

// CaptureStatus matches backend TranscribeStatus
interface CaptureStatus {
  capturing: boolean;
  in_speech: boolean;
  queue_depth: number;
  error: string | null;
  source1_id: string | null;
  source2_id: string | null;
  transcription_mode: TranscriptionMode;
}

// Transcription mode matching backend
type TranscriptionMode = "automatic" | "push_to_talk";

// History entry from the service
interface HistoryEntry {
  id: string;
  text: string;
  timestamp: string;
  wav_path: string | null;
}

// Enriched transcription result payload
interface TranscriptionCompletePayload {
  id: string | null;
  text: string;
  timestamp: string | null;
  audio_path: string | null;
}

// DOM elements
let historyContainer: HTMLElement | null;
let modelWarning: HTMLElement | null;
let modelPathEl: HTMLElement | null;
let downloadModelBtn: HTMLButtonElement | null;
let downloadStatusEl: HTMLElement | null;
let closeBtn: HTMLButtonElement | null;

// Event listeners
let transcriptionCompleteUnlisten: UnlistenFn | null = null;
let captureStateChangedUnlisten: UnlistenFn | null = null;
let historyEntryDeletedUnlisten: UnlistenFn | null = null;
let autoModeToggledUnlisten: UnlistenFn | null = null;
let pttHotkeysChangedUnlisten: UnlistenFn | null = null;

async function checkModelStatus() {
  try {
    const status = await invoke<ModelStatus>("check_model_status");

    if (!status.available && modelWarning && modelPathEl) {
      modelWarning.classList.remove("hidden");
      modelPathEl.textContent = `Model location: ${status.path}`;
    } else if (status.available && modelWarning) {
      modelWarning.classList.add("hidden");
    }
  } catch (error) {
    console.error("Failed to check model status:", error);
  }
}

async function downloadModel() {
  if (!downloadModelBtn || !downloadStatusEl) return;

  downloadModelBtn.disabled = true;
  downloadStatusEl.textContent = "Downloading model... This may take a few minutes.";
  downloadStatusEl.className = "download-status loading";

  try {
    await invoke("download_model");
    downloadStatusEl.textContent = "Download complete!";
    downloadStatusEl.className = "download-status success";

    setTimeout(() => {
      checkModelStatus();
    }, 1500);
  } catch (error) {
    console.error("Download error:", error);
    downloadStatusEl.textContent = `Download failed: ${error}`;
    downloadStatusEl.className = "download-status error";
    downloadModelBtn.disabled = false;
  }
}

// ============== Event Listeners ==============

async function setupEventListeners() {
  // Transcription results
  if (!transcriptionCompleteUnlisten) {
    transcriptionCompleteUnlisten = await listen<TranscriptionCompletePayload>("transcription-complete", (event) => {
      const payload = event.payload;
      if (payload.id && payload.timestamp) {
        appendHistorySegment({
          id: payload.id,
          text: payload.text,
          timestamp: payload.timestamp,
          wav_path: payload.audio_path,
        });
      }
    });
  }

  // Capture state changes
  if (!captureStateChangedUnlisten) {
    captureStateChangedUnlisten = await listen<{capturing: boolean, error: string | null}>(
      "capture-state-changed",
      (event) => {

        if (event.payload.error) {
          console.error("[Capture] Error:", event.payload.error);
        }
      }
    );
  }

  // History entry deleted
  if (!historyEntryDeletedUnlisten) {
    historyEntryDeletedUnlisten = await listen<string>("history-entry-deleted", (event) => {
      removeHistorySegmentFromDOM(event.payload);
    });
  }

  // Auto mode toggled
  if (!autoModeToggledUnlisten) {
    autoModeToggledUnlisten = await listen<TranscriptionMode>("auto-mode-toggled", (event) => {
      const mode = event.payload;
      console.log(`[Main] Auto mode toggled to: ${mode}`);
    });
  }

  // PTT hotkeys changed
  if (!pttHotkeysChangedUnlisten) {
    pttHotkeysChangedUnlisten = await listen("ptt-hotkeys-changed", () => {
      // Nothing to update here since mini waveform is removed
    });
  }
}

function cleanupEventListeners() {
  transcriptionCompleteUnlisten?.();
  transcriptionCompleteUnlisten = null;

  captureStateChangedUnlisten?.();
  captureStateChangedUnlisten = null;

  historyEntryDeletedUnlisten?.();
  historyEntryDeletedUnlisten = null;

  autoModeToggledUnlisten?.();
  autoModeToggledUnlisten = null;

  pttHotkeysChangedUnlisten?.();
  pttHotkeysChangedUnlisten = null;
}

// ============== History Display ==============

let currentAudio: HTMLAudioElement | null = null;

function formatTimestamp(isoString: string): string {
  try {
    const date = new Date(isoString);
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  } catch {
    return "";
  }
}

function createSegmentElement(entry: HistoryEntry): HTMLElement {
  const row = document.createElement("div");
  row.className = "history-segment";
  row.dataset.id = entry.id;

  const ts = document.createElement("span");
  ts.className = "segment-timestamp";
  ts.textContent = formatTimestamp(entry.timestamp);
  row.appendChild(ts);

  const text = document.createElement("span");
  text.className = "segment-text";
  text.textContent = entry.text;
  row.appendChild(text);

  const actions = document.createElement("span");
  actions.className = "segment-actions";

  if (entry.wav_path) {
    const playBtn = document.createElement("button");
    playBtn.className = "segment-btn";
    playBtn.title = "Play audio";
    playBtn.innerHTML = "&#9654;";
    const wavPath = entry.wav_path;
    playBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      playSegmentAudio(wavPath, playBtn);
    });
    actions.appendChild(playBtn);
  }

  const copyBtn = document.createElement("button");
  copyBtn.className = "segment-btn";
  copyBtn.title = "Copy text";
  copyBtn.innerHTML = '<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="5" width="9" height="9" rx="1.5"/><path d="M5 11H3.5A1.5 1.5 0 0 1 2 9.5v-7A1.5 1.5 0 0 1 3.5 1h7A1.5 1.5 0 0 1 12 2.5V5"/></svg>';
  copyBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    navigator.clipboard.writeText(entry.text).then(() => {
      copyBtn.classList.add("copy-success");
      setTimeout(() => copyBtn.classList.remove("copy-success"), 1000);
    });
  });
  actions.appendChild(copyBtn);

  const deleteBtn = document.createElement("button");
  deleteBtn.className = "segment-btn";
  deleteBtn.title = "Delete";
  deleteBtn.innerHTML = "&#10005;";
  deleteBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    deleteHistoryEntry(entry.id, row);
  });
  actions.appendChild(deleteBtn);

  row.appendChild(actions);
  return row;
}

function appendHistorySegment(entry: HistoryEntry): void {
  if (!historyContainer) return;

  const emptyMsg = historyContainer.querySelector(".history-empty");
  if (emptyMsg) emptyMsg.remove();

  const el = createSegmentElement(entry);
  historyContainer.appendChild(el);
  historyContainer.scrollTop = historyContainer.scrollHeight;
}

function removeHistorySegmentFromDOM(id: string): void {
  if (!historyContainer) return;
  const el = historyContainer.querySelector(`[data-id="${id}"]`);
  if (el) el.remove();

  if (historyContainer.children.length === 0) {
    showEmptyState();
  }
}

function showEmptyState(): void {
  if (!historyContainer) return;
  if (historyContainer.querySelector(".history-empty")) return;
  const msg = document.createElement("div");
  msg.className = "history-empty";
  msg.textContent = "No transcriptions yet. Start speaking to begin.";
  historyContainer.appendChild(msg);
}

async function loadHistory(): Promise<void> {
  if (!historyContainer) return;

  try {
    const entries = await invoke<HistoryEntry[]>("get_history");
    historyContainer.innerHTML = "";

    if (entries.length === 0) {
      showEmptyState();
      return;
    }

    for (const entry of entries) {
      const el = createSegmentElement(entry);
      historyContainer.appendChild(el);
    }

    historyContainer.scrollTop = historyContainer.scrollHeight;
  } catch (error) {
    console.error("Failed to load history:", error);
  }
}

async function deleteHistoryEntry(id: string, rowEl: HTMLElement): Promise<void> {
  try {
    await invoke("delete_history_entry", { id });
    rowEl.remove();
    if (historyContainer && historyContainer.children.length === 0) {
      showEmptyState();
    }
  } catch (error) {
    console.error("Failed to delete history entry:", error);
  }
}

function playSegmentAudio(wavPath: string, btn: HTMLButtonElement): void {
  if (currentAudio) {
    currentAudio.pause();
    currentAudio = null;
    document.querySelectorAll(".segment-btn.playing").forEach(b => b.classList.remove("playing"));
  }

  const assetUrl = convertFileSrc(wavPath);
  const audio = new Audio(assetUrl);
  currentAudio = audio;
  btn.classList.add("playing");

  audio.addEventListener("ended", () => {
    btn.classList.remove("playing");
    currentAudio = null;
  });

  audio.addEventListener("error", () => {
    btn.classList.remove("playing");
    currentAudio = null;
    console.error("Failed to play audio:", wavPath);
  });

  audio.play().catch((e) => {
    btn.classList.remove("playing");
    currentAudio = null;
    console.error("Audio playback error:", e);
  });
}

// ============== Window Management ==============

async function openAboutWindow() {
  const existing = await WebviewWindow.getByLabel("about");
  if (existing) {
    const isVisible = await existing.isVisible();
    if (isVisible) {
      await existing.setFocus();
    } else {
      await existing.show();
      await existing.setFocus();
    }
    return;
  }

  const aboutWindow = new WebviewWindow("about", {
    url: "about.html",
    title: "About FlowSTT",
    width: 400,
    height: 460,
    resizable: false,
    maximizable: false,
    minimizable: false,
    decorations: false,
    transparent: false,
    shadow: true,
    skipTaskbar: true,
    center: true,
  });

  aboutWindow.once("tauri://error", (e) => {
    console.error("Failed to create about window:", e.payload);
  });
}

async function openConfigWindow() {
  const existing = await WebviewWindow.getByLabel("config");
  if (existing) {
    const isVisible = await existing.isVisible();
    if (isVisible) {
      await existing.setFocus();
    } else {
      await existing.show();
      await existing.setFocus();
    }
    return;
  }

  const configWindow = new WebviewWindow("config", {
    url: "config.html",
    title: "FlowSTT Settings",
    width: 480,
    height: 529,
    resizable: false,
    maximizable: false,
    minimizable: false,
    decorations: false,
    transparent: true,
    shadow: true,
    skipTaskbar: true,
    center: true,
  });

  configWindow.once("tauri://error", (e) => {
    console.error("Failed to create config window:", e.payload);
  });
}

// ============== Initialization ==============

window.addEventListener("DOMContentLoaded", () => {
  startupLog(`DOMContentLoaded fired at ${performance.now().toFixed(0)}ms`);

  document.addEventListener("contextmenu", (e) => {
    e.preventDefault();
  });

  const suppressKeyHandler = (e: KeyboardEvent) => {
    if (isDebugConsoleHotkey(e)) return;
    if (e.key === "F4" && e.altKey) return;

    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === "SELECT" || tag === "INPUT" || tag === "BUTTON") {
      return;
    }

    e.preventDefault();
  };
  document.addEventListener("keydown", suppressKeyHandler);
  document.addEventListener("keyup", suppressKeyHandler);

  // Get DOM elements
  historyContainer = document.querySelector("#history-container");
  modelWarning = document.querySelector("#model-warning");
  modelPathEl = document.querySelector("#model-path");
  downloadModelBtn = document.querySelector("#download-model-btn");
  downloadStatusEl = document.querySelector("#download-status");
  closeBtn = document.querySelector("#close-btn");

  // Swap logo image based on theme
  const appLogo = document.querySelector<HTMLImageElement>(".app-logo");
  if (appLogo) {
    const updateLogo = (theme: string) => {
      appLogo.src = theme === "light" ? logoLight : logoDark;
    };
    updateLogo(getResolvedTheme());
    onThemeChange(updateLogo);
  }

  // Set up event handlers
  downloadModelBtn?.addEventListener("click", downloadModel);
  document.querySelector("#about-btn")?.addEventListener("click", () => openAboutWindow());
  document.querySelector("#config-btn")?.addEventListener("click", () => openConfigWindow());
  closeBtn?.addEventListener("click", async (e) => {
    e.preventDefault();
    e.stopPropagation();
    const mainWindow = getCurrentWindow();
    await mainWindow.hide();
  });

  window.addEventListener("beforeunload", () => {
    cleanupEventListeners();
  });

  document.addEventListener("visibilitychange", async () => {
    if (!document.hidden) {
      checkModelStatus();
      try {
        await invoke<CaptureStatus>("get_status");
      } catch {
        // Ignore - service may not be ready
      }
    }
  });

  initializeApp();
});

async function initializeApp() {
  const t0 = performance.now();
  const elapsed = () => `${(performance.now() - t0).toFixed(0)}ms`;

  startupLog(`initializeApp started at ${performance.now().toFixed(0)}ms`);

  await initTheme();
  startupLog(`initTheme done (+${elapsed()})`);

  await setupEventListeners();
  startupLog(`setupEventListeners done (+${elapsed()})`);

  try {
    await invoke("connect_events");
    startupLog(`connect_events done (+${elapsed()})`);
  } catch (error) {
    startupLog(`connect_events FAILED (+${elapsed()}): ${error}`);
    console.error(`Connection error: ${error}`);
    try {
      const setupActive = await invoke<boolean>("needs_setup");
      if (!setupActive) {
        const mainWindow = getCurrentWindow();
        await mainWindow.show();
        await mainWindow.setFocus();
      }
    } catch {
      const mainWindow = getCurrentWindow();
      await mainWindow.show();
      await mainWindow.setFocus();
    }
    return;
  }

  try {
    const status = await invoke<CaptureStatus>("get_status");
    startupLog(`get_status done (+${elapsed()})`);

    if (status.error) {
      console.error(`Service error: ${status.error}`);
    }
  } catch (error) {
    startupLog(`get_status FAILED (+${elapsed()}): ${error}`);
  }

  const setupActive = await invoke<boolean>("needs_setup");

  if (!setupActive) {
    checkModelStatus();

    await loadHistory();
    startupLog(`loadHistory done (+${elapsed()})`);

    const mainWindow = getCurrentWindow();
    await mainWindow.show();
    await mainWindow.setFocus();
    startupLog(`window shown - startup complete (+${elapsed()})`);
  } else {
    startupLog(`setup wizard active - main window stays hidden (+${elapsed()})`);

    await listen("setup-complete", async () => {
      startupLog("setup-complete received - refreshing state");
      await checkModelStatus();
      try {
        await invoke<CaptureStatus>("get_status");
      } catch {
        // Ignore
      }
      await loadHistory();
    });
  }
}
