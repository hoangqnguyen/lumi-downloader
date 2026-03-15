import type { DownloadJob, JobEvent } from "../types";
import { startDownload, cancelDownload, listenJobEvents } from "../tauri";
import { getSettings } from "./settings.svelte";

let jobs = $state<DownloadJob[]>([]);
let initialized = false;

export function getJobs(): DownloadJob[] {
  return jobs;
}

export function getActiveCount(): number {
  return jobs.filter((j) => j.status === "downloading" || j.status === "waiting").length;
}

export function getPendingCount(): number {
  return jobs.filter((j) => j.status === "pending").length;
}

export function getDoneCount(): number {
  return jobs.filter((j) => j.status === "done").length;
}

export function getErrorCount(): number {
  return jobs.filter((j) => j.status === "error").length;
}

export function addJobs(entries: Array<{ url: string; title: string }>) {
  const now = Date.now();
  const newJobs: DownloadJob[] = entries.map((e, i) => ({
    id: `${now}-${i}-${Math.random().toString(36).slice(2)}`,
    url: e.url,
    title: e.title || e.url,
    status: "pending",
    percent: 0,
    speed: "",
    eta: "",
    size: "",
    addedAt: now + i,
    retryCount: 0,
  }));
  jobs = [...jobs, ...newJobs];
  scheduleNext();
}

export function updateJob(id: string, patch: Partial<DownloadJob>) {
  const idx = jobs.findIndex((j) => j.id === id);
  if (idx >= 0) {
    jobs[idx] = { ...jobs[idx], ...patch };
  }
}

export function removeJob(id: string) {
  jobs = jobs.filter((j) => j.id !== id);
}

export function clearCompleted() {
  jobs = jobs.filter((j) => j.status !== "done" && j.status !== "cancelled");
}

export function clearAll() {
  const active = jobs.filter(
    (j) => j.status === "downloading" || j.status === "waiting"
  );
  for (const job of active) {
    cancelDownload(job.id).catch(() => {});
  }
  jobs = [];
}

export async function cancelJob(id: string) {
  await cancelDownload(id).catch(() => {});
  updateJob(id, { status: "cancelled", speed: "", eta: "" });
  scheduleNext();
}

export async function retryJob(id: string) {
  const job = jobs.find((j) => j.id === id);
  updateJob(id, {
    status: "pending",
    percent: 0,
    speed: "",
    eta: "",
    error: undefined,
    retryCount: (job?.retryCount ?? 0) + 1,
  });
  scheduleNext();
}

export function retryAllFailed() {
  const failed = jobs.filter((j) => j.status === "error");
  for (const job of failed) {
    updateJob(job.id, {
      status: "pending",
      percent: 0,
      speed: "",
      eta: "",
      error: undefined,
      retryCount: job.retryCount + 1,
    });
  }
  scheduleNext();
}

export function scheduleNext() {
  const settings = getSettings();
  const maxConcurrent = settings.maxConcurrent;
  const currently = jobs.filter(
    (j) => j.status === "downloading" || j.status === "waiting"
  ).length;

  const slotsAvailable = maxConcurrent - currently;
  if (slotsAvailable <= 0) return;

  const pending = jobs.filter((j) => j.status === "pending");
  const toStart = pending.slice(0, slotsAvailable);

  for (const job of toStart) {
    updateJob(job.id, { status: "waiting" });
    const s = getSettings();
    startDownload({
      jobId: job.id,
      url: job.url,
      outputDir: s.outputDir,
      audioOnly: s.audioOnly,
      resolution: s.resolution,
    }).catch((err) => {
      updateJob(job.id, { status: "error", error: String(err) });
      scheduleNext();
    });
  }
}

export function handleJobEvent(event: JobEvent) {
  const { job_id, kind } = event;
  const data = event.data as Record<string, unknown> | undefined;

  // Ignore late-arriving events for cancelled jobs
  const existing = jobs.find((j) => j.id === job_id);
  if (!existing || existing.status === "cancelled") return;

  switch (kind) {
    case "Progress": {
      const p = data as {
        percent: number;
        size: string;
        speed: string;
        eta: string;
      };
      updateJob(job_id, {
        status: "downloading",
        percent: p.percent,
        size: p.size,
        speed: p.speed,
        eta: p.eta,
      });
      break;
    }
    case "Title": {
      const t = data as { title: string };
      updateJob(job_id, { title: t.title });
      break;
    }
    case "Merging": {
      updateJob(job_id, { status: "merging", percent: 100, speed: "", eta: "" });
      break;
    }
    case "FilePath": {
      const f = data as { path: string };
      updateJob(job_id, { filePath: f.path });
      break;
    }
    case "Done": {
      const d = data as { success: boolean; error?: string };
      if (d.success) {
        updateJob(job_id, { status: "done", percent: 100, speed: "", eta: "" });
      } else {
        const settings = getSettings();
        const job = jobs.find((j) => j.id === job_id);
        if (settings.autoRetry && job && job.retryCount < settings.autoRetryMaxAttempts) {
          updateJob(job_id, {
            status: "pending",
            error: undefined,
            percent: 0,
            speed: "",
            eta: "",
            retryCount: job.retryCount + 1,
          });
          setTimeout(() => scheduleNext(), settings.autoRetryDelaySec * 1000);
        } else {
          updateJob(job_id, {
            status: "error",
            error: d.error || "Unknown error",
            speed: "",
            eta: "",
          });
        }
      }
      scheduleNext();
      break;
    }
  }
}

export async function initQueue() {
  if (initialized) return;
  initialized = true;
  await listenJobEvents(handleJobEvent);
}
