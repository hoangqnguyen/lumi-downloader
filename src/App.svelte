<script lang="ts">
  import { onMount } from "svelte";
  import { initQueue } from "./lib/stores/queue.svelte";
  import { getTheme, toggleTheme } from "./lib/stores/settings.svelte";
  import { checkBinaries, checkYtdlpUpdate, updateYtdlp } from "./lib/tauri";
  import UrlInput from "./lib/components/UrlInput.svelte";
  import QueuePanel from "./lib/components/QueuePanel.svelte";
  import AdvancedPanel from "./lib/components/AdvancedPanel.svelte";
  import FolderPicker from "./lib/components/FolderPicker.svelte";
  import SetupScreen from "./lib/components/SetupScreen.svelte";
  import logo from "./assets/logo.png";

  let theme = $derived(getTheme());
  let ready = $state(false);
  let checking = $state(true);
  let updateAvailable = $state(false);
  let updateLatest = $state("");
  let updating = $state(false);
  let updateError = $state("");
  let updateDone = $state(false);

  $effect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  });

  onMount(async () => {
    document.documentElement.setAttribute("data-theme", getTheme());
    const hasBinaries = await checkBinaries();
    if (hasBinaries) {
      await initQueue();
      ready = true;
      // Check for yt-dlp updates in the background
      checkForUpdate();
    }
    checking = false;
  });

  async function checkForUpdate() {
    try {
      const info = await checkYtdlpUpdate();
      updateAvailable = info.update_available;
      updateLatest = info.latest;
    } catch {
      // Silently ignore — not critical
    }
  }

  async function doUpdate() {
    updating = true;
    updateError = "";
    try {
      await updateYtdlp();
      updateAvailable = false;
      updateDone = true;
      setTimeout(() => { updateDone = false; }, 3000);
    } catch (e) {
      updateError = String(e);
    } finally {
      updating = false;
    }
  }

  function onSetupComplete() {
    ready = true;
    initQueue();
    checkForUpdate();
  }
</script>

{#if checking}
  <div class="loading"></div>
{:else if !ready}
  <SetupScreen onComplete={onSetupComplete} />
{:else}
<div class="app">
  <header class="titlebar">
    <div class="logo">
      <img src={logo} alt="Logo" width="24" height="24" />
      <span class="app-name">Lumi Downloader</span>
    </div>
    {#if updateAvailable}
      <button
        class="update-btn shake"
        onclick={doUpdate}
        disabled={updating}
        title="Update to {updateLatest}"
      >
        {#if updating}
          <svg class="spin" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
          </svg>
          <span>Updating...</span>
        {:else}
          <span class="update-dot"></span>
          <span>New version available</span>
        {/if}
      </button>
    {/if}
    {#if updateDone}
      <span class="update-success">Updated!</span>
    {/if}
    {#if updateError}
      <span class="update-error" title={updateError}>Update failed</span>
    {/if}
    <button class="theme-toggle btn-ghost" onclick={toggleTheme} title="Toggle theme" aria-label="Toggle dark/light mode">
      {#if theme === "dark"}
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="4"/>
          <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>
        </svg>
      {/if}
    </button>
  </header>

  <main class="content">
    <div class="top-section">
      <UrlInput />
      <FolderPicker />
      <AdvancedPanel />
    </div>

    <div class="divider"></div>

    <QueuePanel />
  </main>
</div>
{/if}

<style>
  .app {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    overflow: hidden;
  }

  .titlebar {
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    -webkit-app-region: drag;
    user-select: none;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    -webkit-app-region: no-drag;
  }

  .theme-toggle {
    -webkit-app-region: no-drag;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    color: var(--text-dim);
  }

  .app-name {
    font-size: 14px;
    font-weight: 700;
    color: var(--text);
    letter-spacing: -0.01em;
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 16px;
    gap: 12px;
    overflow: hidden;
    min-height: 0;
  }

  .top-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex-shrink: 0;
  }

  .divider {
    height: 1px;
    background: var(--border);
    flex-shrink: 0;
    margin: 0 -4px;
  }

  .loading {
    height: 100vh;
    background: var(--bg);
  }

  .update-btn {
    -webkit-app-region: no-drag;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    font-size: 11px;
    font-weight: 600;
    color: #fff;
    background: var(--orange);
    border: none;
    border-radius: 999px;
    cursor: pointer;
    transition: background 0.15s, box-shadow 0.15s;
    box-shadow: 0 0 0 0 rgba(249, 115, 22, 0);
  }

  .update-btn.shake {
    animation: shake 0.6s ease-in-out 0.5s, glow 2s ease-in-out 1.1s infinite;
  }

  .update-btn:hover:not(:disabled) {
    background: var(--orange-dim);
    box-shadow: 0 0 12px rgba(249, 115, 22, 0.4);
  }

  .update-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
    animation: none;
  }

  .update-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #fff;
    flex-shrink: 0;
    animation: pulse-dot 1.5s ease-in-out infinite;
  }

  .update-success {
    font-size: 11px;
    font-weight: 600;
    color: #22c55e;
    -webkit-app-region: no-drag;
    animation: fade-in-out 3s ease forwards;
  }

  .update-error {
    font-size: 10px;
    color: var(--red);
    -webkit-app-region: no-drag;
  }

  .spin {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    15% { transform: translateX(-3px) rotate(-2deg); }
    30% { transform: translateX(3px) rotate(2deg); }
    45% { transform: translateX(-2px) rotate(-1deg); }
    60% { transform: translateX(2px) rotate(1deg); }
    75% { transform: translateX(-1px); }
  }

  @keyframes glow {
    0%, 100% { box-shadow: 0 0 0 0 rgba(249, 115, 22, 0); }
    50% { box-shadow: 0 0 10px rgba(249, 115, 22, 0.35); }
  }

  @keyframes pulse-dot {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  @keyframes fade-in-out {
    0% { opacity: 0; transform: translateY(-2px); }
    15% { opacity: 1; transform: translateY(0); }
    75% { opacity: 1; }
    100% { opacity: 0; }
  }
</style>
