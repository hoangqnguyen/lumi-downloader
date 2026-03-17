<script lang="ts">
  import {
    getAudioOnly,
    getResolution,
    getTranscript,
    getMaxConcurrent,
    getAutoRetry,
    getAutoRetryMaxAttempts,
    getAutoRetryDelaySec,
    getCookieBrowser,
    setAudioOnly,
    setResolution,
    setTranscript,
    setMaxConcurrent,
    setAutoRetry,
    setAutoRetryMaxAttempts,
    setAutoRetryDelaySec,
    setCookieBrowser,
  } from "../stores/settings.svelte";

  let audioOnly = $derived(getAudioOnly());
  let resolution = $derived(getResolution());
  let transcriptMode = $derived(getTranscript());
  let maxConcurrent = $derived(getMaxConcurrent());
  let autoRetry = $derived(getAutoRetry());
  let autoRetryMaxAttempts = $derived(getAutoRetryMaxAttempts());
  let autoRetryDelaySec = $derived(getAutoRetryDelaySec());
  let cookieBrowser = $derived(getCookieBrowser());
  import type { Resolution, TranscriptMode, CookieBrowser } from "../types";

  const cookieBrowsers: { value: CookieBrowser; label: string }[] = [
    { value: "none", label: "Off" },
    { value: "firefox", label: "Firefox" },
  ];

  const transcriptModes: { value: TranscriptMode; label: string }[] = [
    { value: "none", label: "None" },
    { value: "include", label: "Include" },
    { value: "only", label: "Only" },
  ];

  const resolutions: { value: Resolution; label: string }[] = [
    { value: "best", label: "Best" },
    { value: "1080", label: "1080p" },
    { value: "720", label: "720p" },
    { value: "480", label: "480p" },
    { value: "360", label: "360p" },
  ];

  const formats: { value: boolean; label: string }[] = [
    { value: false, label: "MP4" },
    { value: true, label: "MP3" },
  ];

  let tipText = $state("");
  let tipStyle = $state("");

  function showTip(e: MouseEvent) {
    const el = e.currentTarget as HTMLElement;
    const text = el.getAttribute("data-tip");
    if (!text) return;
    const rect = el.getBoundingClientRect();
    tipText = text;
    tipStyle = `top:${rect.bottom + 6}px;left:${rect.left}px`;
  }

  function hideTip() {
    tipText = "";
  }
</script>

{#if tipText}
  <div class="floating-tip" style={tipStyle}>{tipText}</div>
{/if}

<details class="advanced-panel">
  <summary class="summary">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.07 4.93a10 10 0 0 1 0 14.14M4.93 4.93a10 10 0 0 0 0 14.14"/>
    </svg>
    Advanced options
    <svg class="chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
      <polyline points="6 9 12 15 18 9"/>
    </svg>
  </summary>

  <div class="content">
    <div class="grid">

      <span class="label">Format <!-- svelte-ignore a11y_no_static_element_interactions --><span class="tip" data-tip="MP4 = video + audio. MP3 = audio only" onmouseenter={showTip} onmouseleave={hideTip}>?</span></span>
      <div class="control">
        <div class="radio-group compact-2">
          {#each formats as f}
            <label class="radio-option" class:selected={audioOnly === f.value}>
              <input type="radio" name="format" value={f.label} checked={audioOnly === f.value} onchange={() => setAudioOnly(f.value)} />
              <span>{f.label}</span>
            </label>
          {/each}
        </div>
      </div>

      <span class="label" class:dim={audioOnly}>Resolution <!-- svelte-ignore a11y_no_static_element_interactions --><span class="tip" data-tip="Maximum video quality" onmouseenter={showTip} onmouseleave={hideTip}>?</span></span>
      <div class="control" class:disabled={audioOnly}>
        <div class="radio-group">
          {#each resolutions as r}
            <label class="radio-option" class:selected={resolution === r.value}>
              <input type="radio" name="resolution" value={r.value} checked={resolution === r.value} onchange={() => setResolution(r.value)} disabled={audioOnly} />
              <span>{r.label}</span>
            </label>
          {/each}
        </div>
      </div>

      <span class="label">Transcript <!-- svelte-ignore a11y_no_static_element_interactions --><span class="tip" data-tip="None = no subtitles. Include = subtitles alongside media. Only = subtitles only" onmouseenter={showTip} onmouseleave={hideTip}>?</span></span>
      <div class="control">
        <div class="radio-group compact">
          {#each transcriptModes as t}
            <label class="radio-option" class:selected={transcriptMode === t.value}>
              <input type="radio" name="transcript" value={t.value} checked={transcriptMode === t.value} onchange={() => setTranscript(t.value)} />
              <span>{t.label}</span>
            </label>
          {/each}
        </div>
      </div>

      <span class="label">Cookies <!-- svelte-ignore a11y_no_static_element_interactions --><span class="tip" data-tip="Use Firefox cookies to bypass YouTube bot detection. Requires Firefox with YouTube logged in" onmouseenter={showTip} onmouseleave={hideTip}>?</span></span>
      <div class="control">
        <div class="radio-group compact-2">
          {#each cookieBrowsers as b}
            <label class="radio-option" class:selected={cookieBrowser === b.value}>
              <input type="radio" name="cookie-browser" value={b.value} checked={cookieBrowser === b.value} onchange={() => setCookieBrowser(b.value)} />
              <span>{b.label}</span>
            </label>
          {/each}
        </div>
      </div>

      <span class="label">Parallel <!-- svelte-ignore a11y_no_static_element_interactions --><span class="tip" data-tip="Number of simultaneous downloads" onmouseenter={showTip} onmouseleave={hideTip}>?</span></span>
      <div class="control slider-row">
        <input type="range" min="1" max="5" value={maxConcurrent} onchange={(e) => setMaxConcurrent(Number((e.target as HTMLInputElement).value))} />
        <span class="slider-val">{maxConcurrent}</span>
      </div>

      <span class="label">Auto-retry <!-- svelte-ignore a11y_no_static_element_interactions --><span class="tip" data-tip="Automatically retry failed downloads after a delay" onmouseenter={showTip} onmouseleave={hideTip}>?</span></span>
      <div class="control">
        <div class="retry-row">
          <label class="pill" class:on={autoRetry}>
            <input type="checkbox" checked={autoRetry} onchange={(e) => setAutoRetry((e.target as HTMLInputElement).checked)} />
            {autoRetry ? "On" : "Off"}
          </label>
          {#if autoRetry}
            <span class="retry-sep"></span>
            <span class="retry-label">Retries</span>
            <input type="range" min="1" max="10" value={autoRetryMaxAttempts} onchange={(e) => setAutoRetryMaxAttempts(Number((e.target as HTMLInputElement).value))} />
            <span class="slider-val">{autoRetryMaxAttempts}</span>
            <span class="retry-label">Delay</span>
            <input type="range" min="5" max="60" step="5" value={autoRetryDelaySec} onchange={(e) => setAutoRetryDelaySec(Number((e.target as HTMLInputElement).value))} />
            <span class="slider-val">{autoRetryDelaySec}s</span>
          {/if}
        </div>
      </div>

    </div>
  </div>
</details>

<style>
  .advanced-panel {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .summary {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    cursor: pointer;
    user-select: none;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-dim);
    letter-spacing: 0.03em;
    list-style: none;
    transition: color 0.15s;
  }

  .summary::-webkit-details-marker { display: none; }
  .summary:hover { color: var(--text); }

  .chevron {
    margin-left: auto;
    transition: transform 0.2s ease;
  }

  details[open] .chevron {
    transform: rotate(180deg);
  }

  .content {
    padding: 6px 12px 10px;
    border-top: 1px solid var(--border);
  }

  .grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 7px 14px;
    align-items: center;
  }

  .label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
  }

  .label.dim {
    opacity: 0.35;
  }

  .tip {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 11px;
    height: 11px;
    font-size: 8px;
    font-weight: 600;
    color: var(--text-muted);
    opacity: 0.3;
    border: 1px solid currentColor;
    border-radius: 50%;
    cursor: help;
    vertical-align: middle;
    margin-left: 1px;
    user-select: none;
    transition: opacity 0.15s;
  }

  .tip:hover {
    opacity: 0.7;
  }

  .floating-tip {
    position: fixed;
    z-index: 99999;
    background: #111;
    border: 1px solid #333;
    border-radius: 5px;
    padding: 5px 10px;
    font-size: 10.5px;
    color: #bbb;
    max-width: 240px;
    line-height: 1.4;
    pointer-events: none;
    box-shadow: 0 4px 16px rgba(0,0,0,0.5);
    white-space: normal;
  }

  .control {
    min-width: 0;
  }

  .disabled {
    opacity: 0.35;
    pointer-events: none;
  }

  /* Pill toggle */
  .pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 3px 14px;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-dim);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s;
    user-select: none;
  }

  .pill input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .pill.on {
    background: var(--orange);
    border-color: var(--orange);
    color: #fff;
    font-weight: 600;
  }

  .pill:not(.on):hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  /* Radio group */
  .radio-group {
    display: flex;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    width: 100%;
  }

  .radio-group.compact {
    width: 60%;
    min-width: 160px;
  }

  .radio-group.compact-2 {
    width: 35%;
    min-width: 110px;
  }

  .radio-option {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    position: relative;
  }

  .radio-option + .radio-option {
    border-left: 1px solid var(--border);
  }

  .radio-option input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .radio-option span {
    display: block;
    width: 100%;
    text-align: center;
    padding: 3px 6px;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-dim);
    transition: all 0.15s;
  }

  .radio-option.selected span {
    background: var(--orange);
    color: #fff;
    font-weight: 600;
  }

  .radio-option:not(.selected):hover span {
    background: var(--bg-hover);
    color: var(--text);
  }

  /* Sliders */
  .slider-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .retry-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .retry-sep {
    width: 1px;
    height: 14px;
    background: var(--border);
    margin: 0 2px;
  }

  .retry-label {
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  input[type="range"] {
    flex: 1;
    height: 3px;
    appearance: none;
    background: var(--border);
    border-radius: 999px;
    outline: none;
    border: none;
    cursor: pointer;
    min-width: 0;
  }

  input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--orange);
    cursor: pointer;
    box-shadow: 0 1px 2px rgba(0,0,0,0.3);
  }

  .slider-val {
    min-width: 20px;
    text-align: right;
    font-size: 11px;
    font-weight: 600;
    color: var(--orange);
    font-family: var(--font-mono);
    flex-shrink: 0;
  }
</style>
