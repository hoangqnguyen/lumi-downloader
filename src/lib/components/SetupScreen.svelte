<script lang="ts">
  import { setupBinaries, listenSetupProgress } from "../tauri";
  import logo from "../../assets/logo.png";

  let { onComplete }: { onComplete: () => void } = $props();

  let status = $state("Preparing...");
  let error = $state<string | null>(null);
  let installing = $state(false);

  async function startSetup() {
    installing = true;
    error = null;

    const unlisten = await listenSetupProgress((msg) => {
      status = msg;
    });

    try {
      await setupBinaries();
      unlisten();
      onComplete();
    } catch (e) {
      unlisten();
      error = String(e);
      installing = false;
    }
  }
</script>

<div class="setup">
  <div class="setup-card">
    <img src={logo} alt="Lumi" width="48" height="48" />
    <h2>Welcome to Lumi</h2>
    <p class="subtitle">
      Lumi needs to download a few components before it can start.
    </p>

    {#if error}
      <div class="error">
        <p>{error}</p>
        <button class="btn" onclick={startSetup}>Retry</button>
      </div>
    {:else if installing}
      <div class="progress">
        <div class="spinner"></div>
        <p class="status">{status}</p>
      </div>
    {:else}
      <button class="btn btn-primary" onclick={startSetup}>Set up</button>
      <p class="hint">Downloads yt-dlp (~40 MB) and ffmpeg (~80 MB)</p>
    {/if}
  </div>
</div>

<style>
  .setup {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg);
  }

  .setup-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    max-width: 320px;
    text-align: center;
  }

  h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    color: var(--text);
  }

  .subtitle {
    margin: 0;
    font-size: 13px;
    color: var(--text-dim);
    line-height: 1.5;
  }

  .btn-primary {
    background: var(--accent);
    color: #fff;
    border: none;
    padding: 10px 32px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    margin-top: 8px;
  }

  .btn-primary:hover {
    filter: brightness(1.1);
  }

  .progress {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    margin-top: 8px;
  }

  .status {
    margin: 0;
    font-size: 13px;
    color: var(--text-dim);
  }

  .hint {
    margin: 0;
    font-size: 11px;
    color: var(--text-dim);
    opacity: 0.7;
  }

  .error {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .error p {
    margin: 0;
    font-size: 12px;
    color: var(--error, #e55);
    max-width: 280px;
    word-break: break-word;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
