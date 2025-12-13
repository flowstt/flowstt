import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface AudioDevice {
  id: string;
  name: string;
}

interface ModelStatus {
  available: boolean;
  path: string;
}

interface AudioSamplesPayload {
  samples: number[];
}

// Ring buffer for storing waveform samples
class RingBuffer {
  private buffer: Float32Array;
  private writeIndex: number = 0;
  private filled: boolean = false;

  constructor(capacity: number) {
    this.buffer = new Float32Array(capacity);
  }

  push(samples: number[]): void {
    for (const sample of samples) {
      this.buffer[this.writeIndex] = sample;
      this.writeIndex = (this.writeIndex + 1) % this.buffer.length;
      if (this.writeIndex === 0) {
        this.filled = true;
      }
    }
  }

  // Get samples in order (oldest to newest)
  getSamples(): Float32Array {
    if (!this.filled) {
      // Return only the filled portion
      return this.buffer.slice(0, this.writeIndex);
    }
    // Return samples in chronological order
    const result = new Float32Array(this.buffer.length);
    const secondPart = this.buffer.slice(this.writeIndex);
    const firstPart = this.buffer.slice(0, this.writeIndex);
    result.set(secondPart, 0);
    result.set(firstPart, secondPart.length);
    return result;
  }

  clear(): void {
    this.buffer.fill(0);
    this.writeIndex = 0;
    this.filled = false;
  }

  get length(): number {
    return this.filled ? this.buffer.length : this.writeIndex;
  }
}

// Waveform renderer using Canvas
class WaveformRenderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private animationId: number | null = null;
  private ringBuffer: RingBuffer;
  private isActive: boolean = false;

  constructor(canvas: HTMLCanvasElement, bufferSize: number = 8192) {
    this.canvas = canvas;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Could not get canvas 2D context");
    }
    this.ctx = ctx;
    this.ringBuffer = new RingBuffer(bufferSize);
    this.setupCanvas();
  }

  private setupCanvas(): void {
    // Handle high DPI displays
    const dpr = window.devicePixelRatio || 1;
    const rect = this.canvas.getBoundingClientRect();
    this.canvas.width = rect.width * dpr;
    this.canvas.height = rect.height * dpr;
    this.ctx.scale(dpr, dpr);
  }

  pushSamples(samples: number[]): void {
    this.ringBuffer.push(samples);
  }

  start(): void {
    if (this.isActive) return;
    this.isActive = true;
    this.animate();
  }

  stop(): void {
    this.isActive = false;
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId);
      this.animationId = null;
    }
  }

  get active(): boolean {
    return this.isActive;
  }

  clear(): void {
    this.ringBuffer.clear();
    this.drawIdle();
  }

  private animate = (): void => {
    if (!this.isActive) return;
    this.draw();
    this.animationId = requestAnimationFrame(this.animate);
  };

  private draw(): void {
    const width = this.canvas.getBoundingClientRect().width;
    const height = this.canvas.getBoundingClientRect().height;
    const samples = this.ringBuffer.getSamples();

    // Clear canvas
    this.ctx.fillStyle = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-bg")
      .trim() || "#1e293b";
    this.ctx.fillRect(0, 0, width, height);

    // Draw grid
    this.drawGrid(width, height);

    // Get drawable area (excluding axis labels)
    const area = this.getDrawableArea();

    if (samples.length === 0) {
      this.drawCenterLine(area);
      return;
    }

    // Get colors
    const waveformColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-color")
      .trim() || "#3b82f6";
    const glowColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-glow")
      .trim() || "rgba(59, 130, 246, 0.5)";

    const centerY = area.y + area.height / 2;
    const amplitude = (area.height / 2 - 4) * 1.5; // Increased amplitude scale

    // Downsample if we have more samples than pixels
    const step = Math.max(1, Math.floor(samples.length / area.width));
    const pointCount = Math.min(samples.length, Math.floor(area.width));

    // Build the path once
    this.ctx.beginPath();
    for (let i = 0; i < pointCount; i++) {
      const sampleIndex = Math.floor(i * step);
      const sample = samples[sampleIndex] || 0;
      const x = area.x + (i / pointCount) * area.width;
      // Clamp the sample to prevent drawing outside canvas
      const clampedSample = Math.max(-1, Math.min(1, sample));
      const y = centerY - clampedSample * amplitude;

      if (i === 0) {
        this.ctx.moveTo(x, y);
      } else {
        this.ctx.lineTo(x, y);
      }
    }

    // Draw glow layer (thicker, blurred)
    this.ctx.save();
    this.ctx.strokeStyle = glowColor;
    this.ctx.lineWidth = 6;
    this.ctx.filter = "blur(4px)";
    this.ctx.stroke();
    this.ctx.restore();

    // Draw main waveform line
    this.ctx.strokeStyle = waveformColor;
    this.ctx.lineWidth = 2;
    this.ctx.stroke();
  }

  private drawGrid(width: number, height: number): void {
    const gridColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-grid")
      .trim() || "rgba(255, 255, 255, 0.08)";
    const textColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-text")
      .trim() || "rgba(255, 255, 255, 0.5)";
    
    const leftMargin = 32; // Space for Y-axis labels
    const bottomMargin = 18; // Space for X-axis labels
    const graphWidth = width - leftMargin;
    const graphHeight = height - bottomMargin;
    
    this.ctx.strokeStyle = gridColor;
    this.ctx.lineWidth = 1;

    // Horizontal grid lines (amplitude levels) - tighter spacing
    const horizontalLines = 8;
    for (let i = 0; i <= horizontalLines; i++) {
      const y = (graphHeight / horizontalLines) * i;
      this.ctx.beginPath();
      this.ctx.moveTo(leftMargin, y);
      this.ctx.lineTo(width, y);
      this.ctx.stroke();
    }

    // Vertical grid lines (time divisions) - tighter spacing
    const verticalLines = 16;
    for (let i = 0; i <= verticalLines; i++) {
      const x = leftMargin + (graphWidth / verticalLines) * i;
      this.ctx.beginPath();
      this.ctx.moveTo(x, 0);
      this.ctx.lineTo(x, graphHeight);
      this.ctx.stroke();
    }

    // Draw Y-axis labels (amplitude)
    this.ctx.fillStyle = textColor;
    this.ctx.font = "10px system-ui, sans-serif";
    this.ctx.textAlign = "right";
    this.ctx.textBaseline = "middle";
    
    const yLabels = ["1.0", "0.5", "0", "-0.5", "-1.0"];
    const yPositions = [0, 0.25, 0.5, 0.75, 1];
    for (let i = 0; i < yLabels.length; i++) {
      const y = yPositions[i] * graphHeight;
      this.ctx.fillText(yLabels[i], leftMargin - 4, y);
    }

    // Draw X-axis labels (time in seconds)
    this.ctx.textAlign = "center";
    this.ctx.textBaseline = "top";
    
    // Assuming ~0.5 seconds of visible audio in buffer
    const timeLabels = ["0.0s", "0.1s", "0.2s", "0.3s", "0.4s", "0.5s"];
    for (let i = 0; i < timeLabels.length; i++) {
      const x = leftMargin + (graphWidth / (timeLabels.length - 1)) * i;
      this.ctx.fillText(timeLabels[i], x, graphHeight + 4);
    }
  }

  // Get the drawable area dimensions (excluding margins)
  private getDrawableArea(): { x: number; y: number; width: number; height: number } {
    const width = this.canvas.getBoundingClientRect().width;
    const height = this.canvas.getBoundingClientRect().height;
    const leftMargin = 32;
    const bottomMargin = 18;
    return {
      x: leftMargin,
      y: 0,
      width: width - leftMargin,
      height: height - bottomMargin
    };
  }

  drawIdle(): void {
    const width = this.canvas.getBoundingClientRect().width;
    const height = this.canvas.getBoundingClientRect().height;

    this.ctx.fillStyle = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-bg")
      .trim() || "#1e293b";
    this.ctx.fillRect(0, 0, width, height);
    this.drawGrid(width, height);
    const area = this.getDrawableArea();
    this.drawCenterLine(area);
  }

  private drawCenterLine(area: { x: number; y: number; width: number; height: number }): void {
    const lineColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-line")
      .trim() || "#475569";
    this.ctx.strokeStyle = lineColor;
    this.ctx.lineWidth = 1;
    this.ctx.beginPath();
    const centerY = area.y + area.height / 2;
    this.ctx.moveTo(area.x, centerY);
    this.ctx.lineTo(area.x + area.width, centerY);
    this.ctx.stroke();
  }
}

let deviceSelect: HTMLSelectElement | null;
let recordBtn: HTMLButtonElement | null;
let monitorToggle: HTMLInputElement | null;
let processingToggle: HTMLInputElement | null;
let statusEl: HTMLElement | null;
let resultEl: HTMLElement | null;
let modelWarning: HTMLElement | null;
let modelPathEl: HTMLElement | null;
let downloadModelBtn: HTMLButtonElement | null;
let downloadStatusEl: HTMLElement | null;
let waveformCanvas: HTMLCanvasElement | null;
let closeBtn: HTMLButtonElement | null;

let isRecording = false;
let isMonitoring = false;
let isProcessingEnabled = false;
let wasMonitoringBeforeRecording = false;
let waveformRenderer: WaveformRenderer | null = null;
let audioSamplesUnlisten: UnlistenFn | null = null;
let transcriptionCompleteUnlisten: UnlistenFn | null = null;
let transcriptionErrorUnlisten: UnlistenFn | null = null;

async function loadDevices() {
  try {
    const devices = await invoke<AudioDevice[]>("list_audio_devices");

    if (deviceSelect) {
      deviceSelect.innerHTML = "";

      if (devices.length === 0) {
        deviceSelect.innerHTML =
          '<option value="">No audio devices found</option>';
        return;
      }

      devices.forEach((device) => {
        const option = document.createElement("option");
        option.value = device.id;
        option.textContent = device.name;
        deviceSelect?.appendChild(option);
      });

      if (recordBtn) {
        recordBtn.disabled = false;
      }
      if (monitorToggle) {
        monitorToggle.disabled = false;
      }
      if (processingToggle) {
        processingToggle.disabled = false;
      }
    }
  } catch (error) {
    console.error("Failed to load devices:", error);
    if (deviceSelect) {
      deviceSelect.innerHTML = `<option value="">Error loading devices</option>`;
    }
    setStatus(`Error: ${error}`, "error");
  }
}

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
    
    // Hide warning after successful download
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

function setStatus(message: string, type: "normal" | "loading" | "error" = "normal") {
  if (statusEl) {
    statusEl.textContent = message;
    statusEl.className = "status";
    if (type !== "normal") {
      statusEl.classList.add(type);
    }
  }
}

async function setupAudioListener() {
  if (audioSamplesUnlisten) return;

  audioSamplesUnlisten = await listen<AudioSamplesPayload>("audio-samples", (event) => {
    if (waveformRenderer) {
      waveformRenderer.pushSamples(event.payload.samples);
    }
  });
}

async function cleanupAudioListener() {
  if (audioSamplesUnlisten) {
    audioSamplesUnlisten();
    audioSamplesUnlisten = null;
  }
}

async function setupTranscriptionListeners() {
  if (transcriptionCompleteUnlisten) return;

  transcriptionCompleteUnlisten = await listen<string>("transcription-complete", (event) => {
    if (resultEl) {
      resultEl.textContent = event.payload;
    }
    if (isMonitoring) {
      setStatus("Monitoring...", "loading");
    } else {
      setStatus("Transcription complete");
    }
  });

  transcriptionErrorUnlisten = await listen<string>("transcription-error", (event) => {
    console.error("Transcription error:", event.payload);
    setStatus(`Transcription error: ${event.payload}`, "error");
  });
}

// Cleanup function for transcription listeners (called on app cleanup if needed)
export function cleanupTranscriptionListeners() {
  if (transcriptionCompleteUnlisten) {
    transcriptionCompleteUnlisten();
    transcriptionCompleteUnlisten = null;
  }
  if (transcriptionErrorUnlisten) {
    transcriptionErrorUnlisten();
    transcriptionErrorUnlisten = null;
  }
}

async function toggleProcessing() {
  if (!processingToggle) return;

  const newState = processingToggle.checked;
  try {
    await invoke("set_processing_enabled", { enabled: newState });
    isProcessingEnabled = newState;
    console.log(`Voice processing ${isProcessingEnabled ? "enabled" : "disabled"}`);
  } catch (error) {
    console.error("Toggle processing error:", error);
    // Revert toggle on error
    processingToggle.checked = !newState;
  }
}

async function toggleMonitor() {
  if (!deviceSelect || !monitorToggle) return;

  if (isMonitoring) {
    // Stop monitoring
    try {
      await invoke("stop_monitor");
      isMonitoring = false;
      monitorToggle.checked = false;
      setStatus("");
      
      waveformRenderer?.stop();
      waveformRenderer?.clear();
      await cleanupAudioListener();
    } catch (error) {
      console.error("Stop monitor error:", error);
      setStatus(`Error: ${error}`, "error");
      monitorToggle.checked = true; // Revert toggle on error
    }
  } else {
    // Start monitoring
    const deviceId = deviceSelect.value;
    if (!deviceId) {
      setStatus("Please select an audio device", "error");
      monitorToggle.checked = false; // Revert toggle
      return;
    }

    try {
      await setupAudioListener();
      await invoke("start_monitor", { deviceId });
      isMonitoring = true;
      monitorToggle.checked = true;
      setStatus("Monitoring...", "loading");
      
      waveformRenderer?.clear();
      waveformRenderer?.start();
    } catch (error) {
      console.error("Start monitor error:", error);
      setStatus(`Error: ${error}`, "error");
      monitorToggle.checked = false; // Revert toggle on error
      await cleanupAudioListener();
    }
  }
}

async function toggleRecording() {
  if (!deviceSelect || !recordBtn) return;

  if (isRecording) {
    // Stop recording - this returns immediately, transcription happens in background
    try {
      // Pass whether to keep monitoring
      await invoke("stop_recording", { 
        keepMonitoring: wasMonitoringBeforeRecording 
      });
      
      isRecording = false;
      recordBtn.textContent = "Record";
      recordBtn.classList.remove("recording");
      
      // Re-enable monitor button
      if (monitorToggle) {
        monitorToggle.disabled = false;
      }

      // If monitoring was active before, keep it running
      if (wasMonitoringBeforeRecording) {
        // Monitoring continues, update status
        setStatus("Transcribing... (monitoring continues)", "loading");
        // isMonitoring stays true, waveform keeps running
      } else {
        // Stop visualization since we weren't monitoring before
        isMonitoring = false;
        waveformRenderer?.stop();
        waveformRenderer?.clear();
        await cleanupAudioListener();
        setStatus("Transcribing...", "loading");
      }

      if (resultEl) {
        resultEl.textContent = "Processing audio...";
      }
      
      wasMonitoringBeforeRecording = false;
    } catch (error) {
      console.error("Stop recording error:", error);
      setStatus(`Error: ${error}`, "error");
      isRecording = false;
      recordBtn.textContent = "Record";
      recordBtn.classList.remove("recording");
      if (monitorToggle) {
        monitorToggle.disabled = false;
      }
      // On error, stop everything
      waveformRenderer?.stop();
      waveformRenderer?.clear();
      await cleanupAudioListener();
      isMonitoring = false;
      wasMonitoringBeforeRecording = false;
      if (monitorToggle) {
        monitorToggle.checked = false;
      }
    }
  } else {
    // Start recording
    const deviceId = deviceSelect.value;
    if (!deviceId) {
      setStatus("Please select an audio device", "error");
      return;
    }

    // Remember if monitoring was active before recording
    wasMonitoringBeforeRecording = isMonitoring;

    try {
      // Setup listeners if not already
      await setupAudioListener();
      await setupTranscriptionListeners();
      
      await invoke("start_recording", { deviceId });
      isRecording = true;
      isMonitoring = true; // Recording enables monitoring for visualization
      recordBtn.textContent = "Stop";
      recordBtn.classList.add("recording");
      setStatus("Recording...", "loading");
      
      // Disable monitor button during recording (can't toggle it)
      if (monitorToggle) {
        monitorToggle.disabled = true;
      }

      // Start waveform if not already running
      if (!waveformRenderer?.active) {
        waveformRenderer?.clear();
      }
      waveformRenderer?.start();

      if (resultEl) {
        resultEl.textContent = "Recording in progress...";
      }
    } catch (error) {
      console.error("Start recording error:", error);
      setStatus(`Error: ${error}`, "error");
      wasMonitoringBeforeRecording = false;
      // Don't clean up listener if monitoring was already active
      if (!isMonitoring) {
        await cleanupAudioListener();
      }
    }
  }
}

window.addEventListener("DOMContentLoaded", () => {
  deviceSelect = document.querySelector("#device-select");
  recordBtn = document.querySelector("#record-btn");
  monitorToggle = document.querySelector("#monitor-toggle");
  processingToggle = document.querySelector("#processing-toggle");
  statusEl = document.querySelector("#status");
  resultEl = document.querySelector("#transcription-result");
  modelWarning = document.querySelector("#model-warning");
  modelPathEl = document.querySelector("#model-path");
  downloadModelBtn = document.querySelector("#download-model-btn");
  downloadStatusEl = document.querySelector("#download-status");
  waveformCanvas = document.querySelector("#waveform-canvas");

  // Initialize waveform renderer
  if (waveformCanvas) {
    waveformRenderer = new WaveformRenderer(waveformCanvas);
    waveformRenderer.drawIdle();
    
    // Handle window resize
    window.addEventListener("resize", () => {
      if (waveformCanvas && waveformRenderer) {
        const dpr = window.devicePixelRatio || 1;
        const rect = waveformCanvas.getBoundingClientRect();
        waveformCanvas.width = rect.width * dpr;
        waveformCanvas.height = rect.height * dpr;
        const ctx = waveformCanvas.getContext("2d");
        if (ctx) {
          ctx.scale(dpr, dpr);
        }
      }
    });
  }

  // Setup transcription listeners early
  setupTranscriptionListeners();

  closeBtn = document.querySelector("#close-btn");

  recordBtn?.addEventListener("click", toggleRecording);
  monitorToggle?.addEventListener("change", toggleMonitor);
  processingToggle?.addEventListener("change", toggleProcessing);
  downloadModelBtn?.addEventListener("click", downloadModel);
  closeBtn?.addEventListener("click", async (e) => {
    e.preventDefault();
    e.stopPropagation();
    const window = getCurrentWindow();
    await window.destroy();
  });

  loadDevices();
  checkModelStatus();
});
