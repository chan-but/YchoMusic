<script lang="ts">
  import { onMount } from 'svelte';
  import { windowMode } from '@/stores/windowMode';
  import { setMiniMode, setFullscreenMode, setNormalMode, minimizeWindow, closeWindow } from '@/lib/window';

  async function handleMini() {
    await setMiniMode();
    windowMode.set('mini');
  }

  async function handleFullscreen() {
    await setFullscreenMode();
    windowMode.set('fullscreen');
  }

  async function handleMinimize() {
    await minimizeWindow();
  }

  async function handleClose() {
    await closeWindow();
  }

  onMount(() => {
    document.querySelectorAll('.wc-btn').forEach((el) => {
      el.setAttribute('data-tauri-drag-region', 'no-drag');
      (el as HTMLElement).style.webkitAppRegion = 'no-drag';
    });
  });
</script>

<div class="win-controls">
  <button type="button" class="wc-btn wc-mini" onclick={handleMini} aria-label="迷你播放器" title="迷你播放器">
    <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5">
      <rect x="2" y="5" width="12" height="6" rx="1.5" />
    </svg>
  </button>
  <button type="button" class="wc-btn wc-full" onclick={handleFullscreen} aria-label="全屏沉浸模式" title="全屏沉浸模式">
    <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
      <path d="M2 5V2.5A.5.5 0 0 1 2.5 2H5" />
      <path d="M11 2h2.5a.5.5 0 0 1 .5.5V5" />
      <path d="M14 11v2.5a.5.5 0 0 1-.5.5H11" />
      <path d="M5 14H2.5a.5.5 0 0 1-.5-.5V11" />
    </svg>
  </button>
  <button type="button" class="wc-btn wc-min" onclick={handleMinimize} aria-label="最小化" title="最小化">
    <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
      <line x1="3" y1="8" x2="13" y2="8" />
    </svg>
  </button>
  <button type="button" class="wc-btn wc-close" onclick={handleClose} aria-label="关闭" title="关闭">
    <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
      <line x1="4" y1="4" x2="12" y2="12" />
      <line x1="12" y1="4" x2="4" y2="12" />
    </svg>
  </button>
</div>

<style>
  .win-controls {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: auto;
  }

  .wc-btn {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
    -webkit-app-region: no-drag;
  }

  .wc-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-primary);
  }

  .wc-btn:active {
    transform: scale(0.92);
  }

  .wc-close:hover {
    background: rgba(232, 64, 69, 0.5);
    color: #fff;
  }
</style>
