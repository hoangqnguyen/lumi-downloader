<script lang="ts">
  import { getJobs, clearCompleted, clearAll, retryAllFailed, getActiveCount, getPendingCount, getDoneCount, getErrorCount } from "../stores/queue.svelte";
  import QueueItem from "./QueueItem.svelte";

  let jobs = $derived(getJobs());
  let isEmpty = $derived(jobs.length === 0);
  let activeCount = $derived(getActiveCount());
  let pendingCount = $derived(getPendingCount());
  let doneCount = $derived(getDoneCount());
  let errorCount = $derived(getErrorCount());
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
        {#if activeCount > 0}
          <span class="stat active">{activeCount} active</span>
        {/if}
        {#if pendingCount > 0}
          <span class="stat">{pendingCount} pending</span>
        {/if}
        {#if doneCount > 0}
          <span class="stat done">{doneCount} done</span>
        {/if}
        {#if errorCount > 0}
          <span class="stat error">{errorCount} failed</span>
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

  .stat {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .stat.active {
    color: var(--orange);
  }

  .stat.done {
    color: var(--green);
  }

  .stat.error {
    color: var(--red);
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
