<script lang="ts">
  import { onMount } from "svelte";
  import { initQueue } from "./lib/stores/queue.svelte";
  import { getTheme, toggleTheme } from "./lib/stores/settings.svelte";
  import { checkBinaries } from "./lib/tauri";
  import UrlInput from "./lib/components/UrlInput.svelte";
  import QueuePanel from "./lib/components/QueuePanel.svelte";
  import AdvancedPanel from "./lib/components/AdvancedPanel.svelte";
  import FolderPicker from "./lib/components/FolderPicker.svelte";
  import SetupScreen from "./lib/components/SetupScreen.svelte";
  import logo from "./assets/logo.png";

  let theme = $derived(getTheme());
  let ready = $state(false);
  let checking = $state(true);

  $effect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  });

  onMount(async () => {
    document.documentElement.setAttribute("data-theme", getTheme());
    const hasBinaries = await checkBinaries();
    if (hasBinaries) {
      await initQueue();
      ready = true;
    }
    checking = false;
  });

  function onSetupComplete() {
    ready = true;
    initQueue();
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
    height: 48px;
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
</style>
