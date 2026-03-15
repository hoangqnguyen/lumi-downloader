export type JobStatus =
  | "pending"
  | "waiting"
  | "downloading"
  | "merging"
  | "done"
  | "error"
  | "cancelled";

export type Resolution = "best" | "1080" | "720" | "480" | "360";

export interface DownloadJob {
  id: string;
  url: string;
  title: string;
  status: JobStatus;
  percent: number;
  speed: string;
  eta: string;
  size: string;
  error?: string;
  filePath?: string;
  addedAt: number;
}

export type CookiesBrowser = "" | "chrome" | "firefox" | "safari" | "edge" | "brave" | "opera";

export interface AppSettings {
  outputDir: string;
  audioOnly: boolean;
  resolution: Resolution;
  maxConcurrent: number;
  cookiesBrowser: CookiesBrowser;
}

export interface ProgressEventData {
  percent: number;
  size: string;
  speed: string;
  eta: string;
}

export type JobEventPayload =
  | { kind: "Progress"; data: ProgressEventData }
  | { kind: "Merging" }
  | { kind: "Done"; data: { success: boolean; error?: string } };

export interface JobEvent {
  job_id: string;
  kind: string;
  data?: unknown;
}

export interface PlaylistEntry {
  url: string;
  title: string;
  id: string;
  duration?: number;
  thumbnail?: string;
}
