import { getCurrentWindow } from "@tauri-apps/api/window";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-shell";
import { initTheme, getResolvedTheme, onThemeChange } from "./theme";

import logoLight from "./assets/flowstt-portrait-light.svg";
import logoDark from "./assets/flowstt-portrait.svg";

const WEBSITE_URL = "https://flowstt.io";
const GITHUB_URL = "https://github.com/flowstt/flowstt";
const LICENSE_URL = "https://github.com/flowstt/flowstt/blob/master/LICENSE";

// ── Update info types ─────────────────────────────────────────────────────────

interface UpdateInfo {
    available: boolean;
    version?: string;
    date?: string;
    notes?: string;
}

interface DownloadProgressPayload {
    chunkLength: number;
    contentLength: number | null;
}

// ── Update UI state ───────────────────────────────────────────────────────────

let downloadedBytes = 0;
let totalBytes = 0;

function showUpdateStatus(section: string) {
    const ids = ["update-checking", "update-current", "update-error", "update-available"];
    for (const id of ids) {
        const el = document.getElementById(id);
        if (el) el.style.display = id === section ? "" : "none";
    }
    const statusEl = document.getElementById("update-status");
    if (statusEl) statusEl.style.display = "";
}

function displayUpdateAvailable(info: UpdateInfo) {
    const versionEl = document.getElementById("update-version");
    if (versionEl) versionEl.textContent = `Update available: v${info.version ?? "unknown"}`;
    const notesEl = document.getElementById("update-notes");
    if (notesEl) notesEl.textContent = info.notes ?? "";
    showUpdateStatus("update-available");
    const installBtn = document.getElementById("install-update-btn") as HTMLButtonElement | null;
    if (installBtn) installBtn.disabled = false;
    const progressWrap = document.getElementById("update-progress-wrap");
    if (progressWrap) progressWrap.style.display = "none";
}

async function runUpdateCheck(userInitiated: boolean) {
    const checkBtn = document.getElementById("check-updates-btn") as HTMLButtonElement | null;
    if (checkBtn) checkBtn.disabled = true;
    showUpdateStatus("update-checking");

    try {
        const result = await invoke<UpdateInfo>("check_for_updates");
        if (result.available) {
            displayUpdateAvailable(result);
        } else {
            if (userInitiated) {
                showUpdateStatus("update-current");
            } else {
                // Background-discovered: hide status if already showing "checking"
                const statusEl = document.getElementById("update-status");
                if (statusEl) statusEl.style.display = "none";
            }
        }
    } catch (e) {
        const errorEl = document.getElementById("update-error");
        if (errorEl) {
            errorEl.textContent = `Update check failed: ${e instanceof Error ? e.message : String(e)}`;
        }
        showUpdateStatus("update-error");
    } finally {
        if (checkBtn) checkBtn.disabled = false;
    }
}

function isDebugConsoleHotkey(e: KeyboardEvent): boolean {
    const isIKey = e.code === "KeyI" || e.key === "i" || e.key === "I";
    const isCtrlShift = e.ctrlKey && e.shiftKey && !e.altKey && !e.metaKey;
    const isMetaAlt = e.metaKey && e.altKey && !e.ctrlKey && !e.shiftKey;
    return isIKey && (isCtrlShift || isMetaAlt);
}

/**
 * Open an external URL in the default browser.
 */
function openExternal(url: string) {
    void open(url).catch((error) => {
        console.error("Failed to open external link:", error);
    });
}

document.addEventListener("DOMContentLoaded", async () => {
    // Initialize theme before first paint
    await initTheme();

    // Disable default context menu
    document.addEventListener("contextmenu", (e) => {
        e.preventDefault();
    });

    // Suppress all default keyboard behaviour in this decorationless window.
    // See main.ts for detailed explanation of why this is needed.
    const suppressKeyHandler = (e: KeyboardEvent) => {
        if (isDebugConsoleHotkey(e)) return;
        if (e.key === "F4" && e.altKey) return;
        const tag = (e.target as HTMLElement)?.tagName;
        if (tag === "SELECT" || tag === "INPUT" || tag === "BUTTON") return;
        e.preventDefault();
    };
    document.addEventListener("keydown", suppressKeyHandler);
    document.addEventListener("keyup", suppressKeyHandler);

    // Set version
    try {
        const version = await getVersion();
        const versionEl = document.getElementById("about-version");
        if (versionEl) {
            versionEl.textContent = `Version ${version}`;
        }
    } catch (e) {
        console.error("Failed to get version:", e);
    }

    // Swap logo image based on theme
    const aboutLogo = document.querySelector<HTMLImageElement>(".about-logo");
    if (aboutLogo) {
        const updateLogo = (theme: string) => {
            aboutLogo.src = theme === "light" ? logoLight : logoDark;
        };
        updateLogo(getResolvedTheme());
        onThemeChange(updateLogo);
    }

    // Close button - use destroy() like main window does
    const closeBtn = document.getElementById("close-btn");
    if (closeBtn) {
        closeBtn.addEventListener("click", async (e) => {
            e.preventDefault();
            e.stopPropagation();
            const win = getCurrentWindow();
            await win.destroy();
        });
    }

    // External links
    document.getElementById("link-website")?.addEventListener("click", (e) => {
        e.preventDefault();
        openExternal(WEBSITE_URL);
    });

    document.getElementById("link-github")?.addEventListener("click", (e) => {
        e.preventDefault();
        openExternal(GITHUB_URL);
    });

    document.getElementById("link-license")?.addEventListener("click", (e) => {
        e.preventDefault();
        openExternal(LICENSE_URL);
    });

    // ── Update UI ─────────────────────────────────────────────────────────────

    // "Check for Updates" button
    document.getElementById("check-updates-btn")?.addEventListener("click", () => {
        void runUpdateCheck(true);
    });

    // "Install & Relaunch" button
    const installBtn = document.getElementById("install-update-btn") as HTMLButtonElement | null;
    if (installBtn) {
        installBtn.addEventListener("click", async () => {
            installBtn.disabled = true;
            const progressWrap = document.getElementById("update-progress-wrap");
            if (progressWrap) progressWrap.style.display = "";
            downloadedBytes = 0;
            totalBytes = 0;
            try {
                await invoke("install_update");
                // App relaunches; code below is only reached on error
            } catch (e) {
                const errorEl = document.getElementById("update-error");
                if (errorEl) {
                    errorEl.textContent = `Install failed: ${e instanceof Error ? e.message : String(e)}`;
                }
                showUpdateStatus("update-error");
                installBtn.disabled = false;
            }
        });
    }

    // Download progress events from Rust
    await listen<DownloadProgressPayload>("update-download-progress", (event) => {
        const { chunkLength, contentLength } = event.payload;
        if (contentLength && contentLength > 0) {
            totalBytes = contentLength;
        }
        downloadedBytes += chunkLength;

        const progressEl = document.getElementById("update-progress") as HTMLProgressElement | null;
        const labelEl = document.getElementById("update-progress-label");
        if (progressEl && totalBytes > 0) {
            const pct = Math.min(100, Math.round((downloadedBytes / totalBytes) * 100));
            progressEl.value = pct;
            if (labelEl) labelEl.textContent = `${pct}%`;
        }
    });

    // Background update-available event (from startup check or tray menu trigger)
    await listen<UpdateInfo>("update-available", (event) => {
        displayUpdateAvailable(event.payload);
    });

    // Tray "Check for Updates" triggers a check and surfaces the result here
    await listen("trigger-update-check", () => {
        void runUpdateCheck(true);
    });
});
