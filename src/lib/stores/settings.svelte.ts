import type { AppSettings, Resolution, CookiesBrowser, Theme } from "../types";
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
let cookiesBrowser = $state<CookiesBrowser>(saved.cookiesBrowser ?? "");
let theme = $state<Theme>(saved.theme ?? "dark");

if (!saved.outputDir) {
  getDefaultDownloadDir().then((dir) => {
    outputDir = dir;
    persist();
  });
}

function persist() {
  saveToStorage({ outputDir, audioOnly, resolution, maxConcurrent, cookiesBrowser, theme });
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

export function getCookiesBrowser(): CookiesBrowser {
  return cookiesBrowser;
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

export function setCookiesBrowser(v: CookiesBrowser) {
  cookiesBrowser = v;
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
  return { outputDir, audioOnly, resolution, maxConcurrent, cookiesBrowser, theme };
}
