<script lang="ts">
  import type { DownloadJob } from "../types";
  import { cancelJob, retryJob, removeJob } from "../stores/queue.svelte";
  import { openFile, revealFile } from "../tauri";

  let { job }: { job: DownloadJob } = $props();

  const statusLabel: Record<string, string> = {
    pending: "Pending",
    waiting: "Waiting",
    downloading: "Downloading",
    merging: "Merging",
    done: "Done",
    error: "Error",
    cancelled: "Cancelled",
  };

  const statusColor: Record<string, string> = {
    pending: "var(--text-muted)",
    waiting: "var(--blue)",
    downloading: "var(--orange)",
    merging: "var(--yellow)",
    done: "var(--green)",
    error: "var(--red)",
    cancelled: "var(--text-muted)",
  };

  let isActive = $derived(
    job.status === "downloading" || job.status === "waiting" || job.status === "merging"
  );

  let showProgress = $derived(
    job.status === "downloading" || job.status === "merging" || job.status === "done"
  );

  function formatUrl(url: string) {
    try {
      return new URL(url).hostname.replace("www.", "");
    } catch {
      return url.slice(0, 30);
    }
  }
</script>

<div class="item" class:done={job.status === "done"} class:error={job.status === "error"}>
  <div class="item-header">
    <div class="title-row">
      <span class="title truncate">{job.title}</span>
      <span class="status-badge" style="color: {statusColor[job.status]}">
        {statusLabel[job.status] ?? job.status}
      </span>
    </div>
    <div class="meta-row">
      <span class="domain">{formatUrl(job.url)}</span>
      {#if job.status === "downloading" && job.size}
        <span class="meta">{job.size}</span>
      {/if}
      {#if job.status === "downloading" && job.speed}
        <span class="meta">{job.speed}</span>
      {/if}
      {#if job.status === "downloading" && job.eta && job.eta !== "Unknown"}
        <span class="meta">ETA {job.eta}</span>
      {/if}
    </div>
  </div>

  {#if showProgress}
    <div class="progress-bar">
      <div
        class="progress-fill"
        style="width: {job.percent}%; background: {statusColor[job.status]}"
        class:pulse={job.status === "merging"}
      ></div>
    </div>
  {/if}

  {#if job.status === "error" && job.error}
    <p class="error-msg truncate">
      {job.error}
      {#if job.retryCount > 0}
        <span class="retry-count">(attempt {job.retryCount + 1})</span>
      {/if}
    </p>
  {/if}

  <div class="actions">
    {#if isActive}
      <button class="btn-ghost action-btn" onclick={() => cancelJob(job.id)} title="Cancel">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    {:else if job.status === "error" || job.status === "cancelled"}
      <button class="btn-ghost action-btn" onclick={() => retryJob(job.id)} title="Retry">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="1 4 1 10 7 10"/>
          <path d="M3.51 15a9 9 0 1 0 .49-3.96"/>
        </svg>
      </button>
      <button class="btn-ghost action-btn" onclick={() => removeJob(job.id)} title="Remove">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6l-1 14H6L5 6"/>
          <path d="M10 11v6M14 11v6"/>
        </svg>
      </button>
    {:else if job.status === "done"}
      <button class="btn-ghost action-btn" onclick={() => removeJob(job.id)} title="Dismiss">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    {/if}
  </div>

  {#if job.status === "done" && job.filePath}
    <div class="done-actions">
      <button class="done-btn play" onclick={() => openFile(job.filePath!)} title="Play">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="5 3 19 12 5 21 5 3"/>
        </svg>
        Play
      </button>
      <button class="done-btn find" onclick={() => revealFile(job.filePath!)} title="Show in folder">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        Find
      </button>
    </div>
  {/if}
</div>

<style>
  .item {
    padding: 10px 12px;
    border-radius: var(--radius);
    background: var(--bg-card);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: border-color 0.2s;
    position: relative;
  }

  .item:hover {
    border-color: var(--border-focus);
  }

  .item.done {
    opacity: 0.85;
  }

  .item.error {
    border-color: rgba(239, 68, 68, 0.3);
  }

  .item-header {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding-right: 60px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
    flex: 1;
    min-width: 0;
  }

  .status-badge {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    flex-shrink: 0;
  }

  .meta-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .domain {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .meta {
    font-size: 11px;
    color: var(--text-dim);
    font-family: var(--font-mono);
  }

  .meta::before {
    content: "·";
    margin-right: 8px;
    color: var(--text-muted);
  }

  .progress-bar {
    height: 3px;
    background: var(--border);
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    border-radius: 999px;
    transition: width 0.4s ease;
  }

  .pulse {
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .error-msg {
    font-size: 11px;
    color: var(--red);
    font-family: var(--font-mono);
  }

  .retry-count {
    color: var(--text-muted);
    font-weight: 500;
  }

  .actions {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .item:hover .actions {
    opacity: 1;
  }

  .action-btn {
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border-radius: 6px;
  }

  .done-actions {
    display: flex;
    gap: 6px;
  }

  .done-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 500;
    border: 1px solid var(--border);
    background: transparent;
    transition: all 0.15s;
  }

  .done-btn.play {
    color: var(--orange);
    border-color: rgba(255, 107, 0, 0.3);
  }

  .done-btn.play:hover {
    background: var(--orange-subtle);
    border-color: var(--orange);
  }

  .done-btn.find {
    color: var(--text-dim);
  }

  .done-btn.find:hover {
    background: var(--bg-hover);
    color: var(--text);
    border-color: var(--border-focus);
  }
</style>
