import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PlaylistEntry, JobEvent } from "./types";

export function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  return tauriInvoke<T>(cmd, args);
}

export async function getDefaultDownloadDir(): Promise<string> {
  return invoke<string>("get_default_download_dir");
}

export async function pickFolder(): Promise<string | null> {
  return invoke<string | null>("pick_folder");
}

export async function openFolder(path: string): Promise<void> {
  return invoke<void>("open_folder", { path });
}

export async function expandPlaylist(url: string, cookiesBrowser: string = ""): Promise<PlaylistEntry[]> {
  return invoke<PlaylistEntry[]>("expand_playlist", { url, cookiesBrowser });
}

export async function startDownload(params: {
  jobId: string;
  url: string;
  outputDir: string;
  audioOnly: boolean;
  resolution: string;
  cookiesBrowser: string;
}): Promise<void> {
  return invoke<void>("start_download", {
    jobId: params.jobId,
    url: params.url,
    outputDir: params.outputDir,
    audioOnly: params.audioOnly,
    resolution: params.resolution,
    cookiesBrowser: params.cookiesBrowser,
  });
}

export async function cancelDownload(jobId: string): Promise<void> {
  return invoke<void>("cancel_download", { jobId });
}

export async function openFile(path: string): Promise<void> {
  return invoke<void>("open_file", { path });
}

export async function revealFile(path: string): Promise<void> {
  return invoke<void>("reveal_file", { path });
}

export async function listenJobEvents(
  handler: (event: JobEvent) => void
): Promise<() => void> {
  return listen<JobEvent>("job-event", (e) => handler(e.payload));
}
