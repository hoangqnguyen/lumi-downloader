<script lang="ts">
  import { getJobs, clearCompleted, clearAll, retryAllFailed, getActiveCount, getPendingCount, getDoneCount, getErrorCount } from "../stores/queue.svelte";
  import QueueItem from "./QueueItem.svelte";
  import type { DownloadJob } from "../types";

  type Filter = "all" | "active" | "pending" | "done" | "error";
  let filter = $state<Filter>("all");

  let allJobs = $derived(getJobs());
  let isEmpty = $derived(allJobs.length === 0);
  let activeCount = $derived(getActiveCount());
  let pendingCount = $derived(getPendingCount());
  let doneCount = $derived(getDoneCount());
  let errorCount = $derived(getErrorCount());

  let jobs = $derived.by(() => {
    switch (filter) {
      case "active": return allJobs.filter((j: DownloadJob) => j.status === "downloading" || j.status === "waiting");
      case "pending": return allJobs.filter((j: DownloadJob) => j.status === "pending");
      case "done": return allJobs.filter((j: DownloadJob) => j.status === "done");
      case "error": return allJobs.filter((j: DownloadJob) => j.status === "error");
      default: return allJobs;
    }
  });

  function toggleFilter(f: Filter) {
    filter = filter === f ? "all" : f;
  }
</script>

<div class="queue-panel">
  {#if isEmpty}
    <div class="empty-state">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <p>No downloads yet</p>
      <span>Paste YT URLs above to get started</span>
    </div>
  {:else}
    <div class="queue-header">
      <div class="stats">
        <button class="stat-filter" class:selected={filter === "all"} onclick={() => filter = "all"}>
          {allJobs.length} all
        </button>
        {#if activeCount > 0}
          <button class="stat-filter active" class:selected={filter === "active"} onclick={() => toggleFilter("active")}>
            {activeCount} active
          </button>
        {/if}
        {#if pendingCount > 0}
          <button class="stat-filter pending" class:selected={filter === "pending"} onclick={() => toggleFilter("pending")}>
            {pendingCount} pending
          </button>
        {/if}
        {#if doneCount > 0}
          <button class="stat-filter done" class:selected={filter === "done"} onclick={() => toggleFilter("done")}>
            {doneCount} done
          </button>
        {/if}
        {#if errorCount > 0}
          <button class="stat-filter error" class:selected={filter === "error"} onclick={() => toggleFilter("error")}>
            {errorCount} failed
          </button>
        {/if}
      </div>
      <div class="queue-actions">
        {#if errorCount > 0}
          <button class="btn-ghost small retry-all" onclick={retryAllFailed}>Retry failed</button>
        {/if}
        {#if doneCount > 0}
          <button class="btn-ghost small" onclick={clearCompleted}>Clear done</button>
        {/if}
        <button class="btn-ghost small" onclick={clearAll}>Clear all</button>
      </div>
    </div>
    <div class="queue-list">
      {#each jobs as job (job.id)}
        <QueueItem {job} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .queue-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--text-muted);
    padding: 40px 0;
  }

  .empty-state p {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-dim);
  }

  .empty-state span {
    font-size: 12px;
  }

  .queue-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 0 8px;
    flex-shrink: 0;
  }

  .stats {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .stat-filter {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
    background: none;
    border: 1px solid transparent;
    border-radius: 4px;
    padding: 2px 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .stat-filter:hover {
    background: var(--bg-hover, rgba(255, 255, 255, 0.06));
  }

  .stat-filter.selected {
    border-color: var(--text-muted);
    background: var(--bg-hover, rgba(255, 255, 255, 0.06));
  }

  .stat-filter.active {
    color: var(--orange);
  }
  .stat-filter.active.selected {
    border-color: var(--orange);
  }

  .stat-filter.done {
    color: var(--green);
  }
  .stat-filter.done.selected {
    border-color: var(--green);
  }

  .stat-filter.error {
    color: var(--red);
  }
  .stat-filter.error.selected {
    border-color: var(--red);
  }

  .queue-actions {
    display: flex;
    gap: 4px;
  }

  .small {
    font-size: 11px;
    padding: 4px 8px;
  }

  .retry-all {
    color: var(--orange);
  }

  .queue-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow-y: auto;
    padding-bottom: 8px;
    flex: 1;
  }
</style>
