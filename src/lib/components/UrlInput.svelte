<script lang="ts">
  import { parseInput } from "../parseUrls";
  import { expandPlaylist } from "../tauri";
  import { addJobs } from "../stores/queue.svelte";
  import type { ParsedUrl } from "../parseUrls";

  let value = $state("");
  let loading = $state(false);
  let error = $state("");
  let textarea: HTMLTextAreaElement | null = $state(null);
  const isMac = navigator.userAgent.includes("Mac");
  const submitHint = isMac ? "⌘+Return to submit" : "Ctrl+Enter to submit";

  export function focus() {
    textarea?.focus();
  }

  async function submit() {
    const trimmed = value.trim();
    if (!trimmed) return;

    loading = true;
    error = "";

    try {
      const parsed = parseInput(trimmed);
      if (parsed.length === 0) {
        error = "No valid URLs found. Paste a YouTube or TikTok link.";
        return;
      }

      const jobs: Array<{ url: string; title: string }> = [];

      for (const item of parsed) {
        if (item.type === "video") {
          jobs.push({ url: item.url, title: extractTitle(item.url) });
        } else if (item.type === "playlist") {
          try {
            const entries = await expandPlaylist(item.url);
            for (const e of entries) {
              jobs.push({ url: e.url, title: e.title });
            }
          } catch (e) {
            error = `Failed to expand playlist: ${e}`;
          }
        } else if (item.type === "video_in_playlist") {
          // Show dialog to ask — for now just add the single video
          jobs.push({ url: item.videoUrl, title: extractTitle(item.videoUrl) });
        }
      }

      if (jobs.length > 0) {
        addJobs(jobs);
        value = "";
      }
    } finally {
      loading = false;
    }
  }

  function extractTitle(url: string): string {
    // Best effort — real title will come from yt-dlp during download
    try {
      const u = new URL(url);
      if (u.hostname.includes("tiktok.com") || u.hostname === "vm.tiktok.com") {
        const videoId = u.pathname.match(/\/video\/(\d+)/)?.[1];
        return videoId ? `TikTok (${videoId})` : "TikTok video";
      }
      const v = u.searchParams.get("v");
      return v ? `Video (${v})` : url;
    } catch {
      return url;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      submit();
    }
  }

</script>

<div class="url-input">
  <div class="input-label">
    <span>Add URLs</span>
    <span class="hint">Paste links, CSV, or newline-separated · {submitHint}</span>
  </div>
  <div class="input-wrap" class:loading>
    <textarea
      bind:this={textarea}
      bind:value
      onkeydown={onKeydown}
placeholder="https://youtube.com/watch?v=... or channel/playlist/TikTok link"
      rows={3}
      disabled={loading}
    ></textarea>
    <button
      class="submit-btn"
      onclick={submit}
      disabled={loading || !value.trim()}
      aria-label="Add to queue"
    >
      {#if loading}
        <svg class="spin" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
        </svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <polyline points="19 12 12 19 5 12"/>
        </svg>
      {/if}
    </button>
  </div>
  {#if error}
    <p class="error">{error}</p>
  {/if}
</div>

<style>
  .url-input {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .input-label {
    display: flex;
    align-items: baseline;
    gap: 10px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-dim);
  }

  .hint {
    font-weight: 400;
    text-transform: none;
    letter-spacing: 0;
    color: var(--text-muted);
    font-size: 11px;
  }

  .input-wrap {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    position: relative;
  }

  textarea {
    flex: 1;
    padding: 10px 12px;
    border-radius: var(--radius);
    background: var(--bg-input);
    border: 1px solid var(--border);
    color: var(--text);
    resize: none;
    line-height: 1.6;
    font-size: 13px;
    min-height: 72px;
    transition: border-color 0.15s;
  }

  textarea::placeholder {
    color: var(--text-muted);
  }

  textarea:focus {
    border-color: var(--blue);
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.25);
  }

  .submit-btn {
    width: 38px;
    height: 38px;
    border-radius: var(--radius-sm);
    background: var(--orange);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .submit-btn:hover:not(:disabled) {
    background: var(--orange-dim);
  }

  .submit-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .spin {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading textarea {
    opacity: 0.6;
  }

  .error {
    font-size: 12px;
    color: var(--red);
    padding: 6px 10px;
    background: rgba(239, 68, 68, 0.1);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(239, 68, 68, 0.2);
  }
</style>
