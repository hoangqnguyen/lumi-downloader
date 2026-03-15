import type { AppSettings, Resolution, Theme } from "../types";
import { getDefaultDownloadDir } from "../tauri";

const STORAGE_KEY = "ytdl-settings";

function loadFromStorage(): Partial<AppSettings> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function saveToStorage(s: AppSettings) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
  } catch {}
}

const saved = loadFromStorage();

let outputDir = $state<string>(saved.outputDir ?? "");
let audioOnly = $state<boolean>(saved.audioOnly ?? false);
let resolution = $state<Resolution>(saved.resolution ?? "best");
let maxConcurrent = $state<number>(saved.maxConcurrent ?? 3);
let theme = $state<Theme>(saved.theme ?? "dark");
let autoRetry = $state<boolean>(saved.autoRetry ?? false);
let autoRetryMaxAttempts = $state<number>(saved.autoRetryMaxAttempts ?? 3);
let autoRetryDelaySec = $state<number>(saved.autoRetryDelaySec ?? 10);

if (!saved.outputDir) {
  getDefaultDownloadDir().then((dir) => {
    outputDir = dir;
    persist();
  });
}

function persist() {
  saveToStorage({ outputDir, audioOnly, resolution, maxConcurrent, theme, autoRetry, autoRetryMaxAttempts, autoRetryDelaySec });
}

export function getOutputDir(): string {
  return outputDir;
}

export function getAudioOnly(): boolean {
  return audioOnly;
}

export function getResolution(): Resolution {
  return resolution;
}

export function getMaxConcurrent(): number {
  return maxConcurrent;
}

export function setOutputDir(v: string) {
  outputDir = v;
  persist();
}

export function setAudioOnly(v: boolean) {
  audioOnly = v;
  persist();
}

export function setResolution(v: Resolution) {
  resolution = v;
  persist();
}

export function setMaxConcurrent(v: number) {
  maxConcurrent = v;
  persist();
}

export function getAutoRetry(): boolean {
  return autoRetry;
}

export function getAutoRetryMaxAttempts(): number {
  return autoRetryMaxAttempts;
}

export function getAutoRetryDelaySec(): number {
  return autoRetryDelaySec;
}

export function setAutoRetry(v: boolean) {
  autoRetry = v;
  persist();
}

export function setAutoRetryMaxAttempts(v: number) {
  autoRetryMaxAttempts = v;
  persist();
}

export function setAutoRetryDelaySec(v: number) {
  autoRetryDelaySec = v;
  persist();
}

export function getTheme(): Theme {
  return theme;
}

export function setTheme(v: Theme) {
  theme = v;
  persist();
}

export function toggleTheme() {
  theme = theme === "dark" ? "light" : "dark";
  persist();
}

export function getSettings(): AppSettings {
  return { outputDir, audioOnly, resolution, maxConcurrent, theme, autoRetry, autoRetryMaxAttempts, autoRetryDelaySec };
}
