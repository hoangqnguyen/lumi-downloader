<script lang="ts">
  import {
    getAudioOnly,
    getResolution,
    getMaxConcurrent,
    getCookiesBrowser,
    setAudioOnly,
    setResolution,
    setMaxConcurrent,
    setCookiesBrowser,
  } from "../stores/settings.svelte";

  let audioOnly = $derived(getAudioOnly());
  let resolution = $derived(getResolution());
  let maxConcurrent = $derived(getMaxConcurrent());
  let cookiesBrowser = $derived(getCookiesBrowser());
  import type { Resolution, CookiesBrowser } from "../types";

  const resolutions: { value: Resolution; label: string }[] = [
    { value: "best", label: "Best quality" },
    { value: "1080", label: "1080p" },
    { value: "720", label: "720p" },
    { value: "480", label: "480p" },
    { value: "360", label: "360p" },
  ];

  const browsers: { value: CookiesBrowser; label: string }[] = [
    { value: "", label: "None (disabled)" },
    { value: "chrome", label: "Chrome" },
    { value: "firefox", label: "Firefox" },
    { value: "safari", label: "Safari" },
    { value: "edge", label: "Edge" },
    { value: "brave", label: "Brave" },
    { value: "opera", label: "Opera" },
  ];
</script>

<details class="advanced-panel">
  <summary class="summary">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.07 4.93a10 10 0 0 1 0 14.14M4.93 4.93a10 10 0 0 0 0 14.14"/>
    </svg>
    Advanced options
    <svg class="chevron" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
      <polyline points="6 9 12 15 18 9"/>
    </svg>
  </summary>

  <div class="content">
    <div class="row">
      <label class="toggle-label" for="audio-only">
        <span class="label-text">
          <span class="label-title">Audio only</span>
          <span class="label-sub">Extracts MP3 from video</span>
        </span>
        <div class="toggle" class:on={audioOnly}>
          <input
            id="audio-only"
            type="checkbox"
            checked={audioOnly}
            onchange={(e) => setAudioOnly((e.target as HTMLInputElement).checked)}
          />
          <div class="track"><div class="thumb"></div></div>
        </div>
      </label>
    </div>

    <div class="row" class:disabled={audioOnly}>
      <label class="field" for="resolution">
        <span class="field-label">Resolution</span>
        <select
          id="resolution"
          value={resolution}
          onchange={(e) => setResolution((e.target as HTMLSelectElement).value as Resolution)}
          disabled={audioOnly}
        >
          {#each resolutions as r}
            <option value={r.value}>{r.label}</option>
          {/each}
        </select>
      </label>
    </div>

    <div class="row">
      <div class="field">
        <span class="field-label">Parallel downloads</span>
        <div class="slider-row">
          <input
            type="range"
            min="1"
            max="5"
            value={maxConcurrent}
            onchange={(e) => setMaxConcurrent(Number((e.target as HTMLInputElement).value))}
          />
          <span class="slider-val">{maxConcurrent}</span>
        </div>
      </div>
    </div>

    <div class="row">
      <label class="field" for="cookies-browser">
        <span class="field-label">Browser cookies</span>
        <span class="field-hint">Use cookies from a browser to bypass bot detection</span>
        <select
          id="cookies-browser"
          value={cookiesBrowser}
          onchange={(e) => setCookiesBrowser((e.target as HTMLSelectElement).value as CookiesBrowser)}
        >
          {#each browsers as b}
            <option value={b.value}>{b.label}</option>
          {/each}
        </select>
      </label>
    </div>
  </div>
</details>

<style>
  .advanced-panel {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .summary {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    cursor: pointer;
    user-select: none;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-dim);
    letter-spacing: 0.03em;
    list-style: none;
    transition: color 0.15s;
  }

  .summary::-webkit-details-marker {
    display: none;
  }

  .summary:hover {
    color: var(--text);
  }

  .chevron {
    margin-left: auto;
    transition: transform 0.2s ease;
  }

  details[open] .chevron {
    transform: rotate(180deg);
  }

  .content {
    padding: 4px 14px 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    border-top: 1px solid var(--border);
  }

  .row {
    transition: opacity 0.2s;
  }

  .row.disabled {
    opacity: 0.4;
    pointer-events: none;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    cursor: pointer;
  }

  .label-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .label-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
  }

  .label-sub {
    font-size: 11px;
    color: var(--text-muted);
  }

  .toggle input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .track {
    width: 40px;
    height: 22px;
    background: var(--border);
    border-radius: 999px;
    position: relative;
    transition: background 0.2s;
    cursor: pointer;
  }

  .toggle.on .track {
    background: var(--orange);
  }

  .thumb {
    width: 16px;
    height: 16px;
    background: #fff;
    border-radius: 50%;
    position: absolute;
    top: 3px;
    left: 3px;
    transition: transform 0.2s;
    box-shadow: 0 1px 3px rgba(0,0,0,0.3);
  }

  .toggle.on .thumb {
    transform: translateX(18px);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-dim);
  }

  .field-hint {
    font-size: 11px;
    color: var(--text-muted);
  }

  select {
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    background: var(--bg-input);
    border: 1px solid var(--border);
    color: var(--text);
    font-size: 13px;
    width: 100%;
  }

  select:focus {
    border-color: var(--orange);
  }

  .slider-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  input[type="range"] {
    flex: 1;
    height: 4px;
    appearance: none;
    background: var(--border);
    border-radius: 999px;
    outline: none;
    border: none;
    cursor: pointer;
  }

  input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--orange);
    cursor: pointer;
    box-shadow: 0 1px 3px rgba(0,0,0,0.3);
  }

  .slider-val {
    width: 20px;
    text-align: center;
    font-size: 13px;
    font-weight: 600;
    color: var(--orange);
    font-family: var(--font-mono);
  }
</style>
