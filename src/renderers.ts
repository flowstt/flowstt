// Mini waveform renderer for the header area.
// Shows a real-time waveform while PTT recording is active.

// Ring buffer for storing waveform samples
export class RingBuffer {
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
      return this.buffer.slice(0, this.writeIndex);
    }
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

export class MiniWaveformRenderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private animationId: number | null = null;
  private ringBuffer: RingBuffer;
  private isActive: boolean = false;

  constructor(canvas: HTMLCanvasElement, bufferSize: number = 512) {
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

  // Attenuation from center (0.5) to edges (0 or 1)
  // Magnifies amplitude at center, tapers to 0 at edges
  private attenuation(t: number): number {
    const distFromCenter = Math.abs(t - 0.5) * 2;
    return Math.cos(distFromCenter * Math.PI / 2) * 2;
  }

  private draw(): void {
    const dpr = window.devicePixelRatio || 1;
    const width = this.canvas.width / dpr;
    const height = this.canvas.height / dpr;
    const samples = this.ringBuffer.getSamples();

    this.ctx.clearRect(0, 0, width, height);

    if (samples.length === 0) {
      this.drawIdleLine(width, height);
      return;
    }

    const waveformColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-color")
      .trim() || "#3b82f6";
    const glowColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-glow")
      .trim() || "rgba(59, 130, 246, 0.5)";

    const centerY = height / 2;
    const maxAmplitude = height / 2 - 2;
    const pointCount = samples.length;

    // Find peak amplitude after attenuation to prevent clipping
    let peakAttenuated = 0;
    for (let i = 0; i < pointCount; i++) {
      const t = i / (pointCount - 1);
      const att = this.attenuation(t);
      const sample = Math.abs(samples[i] || 0);
      peakAttenuated = Math.max(peakAttenuated, sample * att);
    }

    const scale = peakAttenuated > 1 ? 1 / peakAttenuated : 1;

    // Calculate points with attenuation applied
    const points: { x: number; y: number }[] = [];
    for (let i = 0; i < pointCount; i++) {
      const t = i / (pointCount - 1);
      const att = this.attenuation(t);
      const sample = samples[i] || 0;
      const x = t * width;
      const clampedSample = Math.max(-1, Math.min(1, sample));
      const y = centerY - clampedSample * maxAmplitude * att * scale;
      points.push({ x, y });
    }

    // Build smooth path using Catmull-Rom spline interpolation
    this.ctx.beginPath();
    if (points.length > 0) {
      this.ctx.moveTo(points[0].x, points[0].y);

      if (points.length === 1) {
        this.ctx.lineTo(points[0].x, points[0].y);
      } else if (points.length === 2) {
        this.ctx.lineTo(points[1].x, points[1].y);
      } else {
        for (let i = 0; i < points.length - 1; i++) {
          const p0 = points[Math.max(0, i - 1)];
          const p1 = points[i];
          const p2 = points[i + 1];
          const p3 = points[Math.min(points.length - 1, i + 2)];

          const tension = 0.5;
          const cp1x = p1.x + (p2.x - p0.x) * tension / 3;
          const cp1y = p1.y + (p2.y - p0.y) * tension / 3;
          const cp2x = p2.x - (p3.x - p1.x) * tension / 3;
          const cp2y = p2.y - (p3.y - p1.y) * tension / 3;

          this.ctx.bezierCurveTo(cp1x, cp1y, cp2x, cp2y, p2.x, p2.y);
        }
      }
    }

    // Glow layer
    this.ctx.save();
    this.ctx.strokeStyle = glowColor;
    this.ctx.lineWidth = 4;
    this.ctx.filter = "blur(3px)";
    this.ctx.stroke();
    this.ctx.restore();

    // Main waveform line
    this.ctx.strokeStyle = waveformColor;
    this.ctx.lineWidth = 1;
    this.ctx.stroke();
  }

  drawIdle(): void {
    const dpr = window.devicePixelRatio || 1;
    const width = this.canvas.width / dpr;
    const height = this.canvas.height / dpr;
    this.ctx.clearRect(0, 0, width, height);
    this.drawIdleLine(width, height);
  }

  private drawIdleLine(width: number, height: number): void {
    const centerY = height / 2;
    const waveformColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-color")
      .trim() || "#3b82f6";
    const glowColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--waveform-glow")
      .trim() || "rgba(59, 130, 246, 0.5)";

    this.ctx.beginPath();
    this.ctx.moveTo(0, centerY);
    this.ctx.lineTo(width, centerY);

    this.ctx.save();
    this.ctx.strokeStyle = glowColor;
    this.ctx.lineWidth = 4;
    this.ctx.filter = "blur(3px)";
    this.ctx.stroke();
    this.ctx.restore();

    this.ctx.strokeStyle = waveformColor;
    this.ctx.lineWidth = 1;
    this.ctx.stroke();
  }

  resize(): void {
    this.setupCanvas();
  }
}
