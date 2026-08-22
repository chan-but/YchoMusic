<script lang="ts">
  import { playerState, play, pause, prev, next, seek, setVolume, setMode, startPositionUpdate, stopPositionUpdate, togglePlayPause, getCurrentPlayerState } from '@/stores/player';
  import { tracks } from '@/stores/library';
  import { settings, setSetting } from '@/stores/settings';
  import { windowMode } from '@/stores/windowMode';
  import { setNormalMode } from '@/lib/window';
  import type { PlayMode } from '@/types';
  import { onMount, onDestroy } from 'svelte';

  let showControls = $state(true);
  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  let isHoveringControls = $state(false);
  let isHoveringExit = $state(false);
  let showLyricsSettings = $state(false);

  function updateLyricsSetting(key: string, value: string) {
    setSetting(key, value);
  }

  let currentTrack = $derived.by(() => {
    const id = $playerState.current_track_id;
    if (!id) return undefined;
    return $tracks.find(t => t.id === id);
  });

  let currentCoverUrl = $derived.by(() => {
    const t = currentTrack;
    if (!t?.cover_blob) return '';
    return `data:image/jpeg;base64,${t.cover_blob}`;
  });

  let currentState = $derived.by(() => $playerState.state);
  let currentTrackId = $derived.by(() => $playerState.current_track_id);

  onMount(() => {
    startPositionUpdate();
    showControls = true;
    scheduleHide();
  });

  onDestroy(() => {
    if (hideTimer) clearTimeout(hideTimer);
  });

  function scheduleHide() {
    if (hideTimer) clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (!isHoveringControls && !isHoveringExit) showControls = false;
    }, 4000);
  }

  function onMouseMove() {
    showControls = true;
    scheduleHide();
  }

  function onExitHover() {
    isHoveringExit = true;
    showControls = true;
    scheduleHide();
  }

  function onExitLeave() {
    isHoveringExit = false;
    scheduleHide();
  }

  function formatTime(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  const modeIcons: Record<string, string> = {
    RepeatList: 'repeat-list',
    RepeatOne: 'repeat-one',
    Shuffle: 'shuffle',
  };

  function getModeSvg(mode: string): string {
    const icon = modeIcons[mode] || 'repeat-list';
    if (icon === 'shuffle') {
      return '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/></svg>';
    } else if (icon === 'repeat-one') {
      return '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/><text x="12" y="16" text-anchor="middle" font-size="7" fill="currentColor" stroke="none">1</text></svg>';
    } else {
      return '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/></svg>';
    }
  }

  function cycleMode() {
    const modes = ['repeat_list', 'shuffle', 'repeat_one'];
    const state = getCurrentPlayerState();
    const cur = state.mode === 'Shuffle' ? 'shuffle' : state.mode === 'RepeatOne' ? 'repeat_one' : 'repeat_list';
    const currentIdx = modes.indexOf(cur);
    setMode(modes[(currentIdx + 1) % modes.length]);
  }

  async function exitFullscreen() {
    try {
      await setNormalMode();
    } catch {}
    windowMode.set('normal');
  }

  function onProgressClick(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const pct = (e.clientX - rect.left) / rect.width;
    const state = getCurrentPlayerState();
    if (state.duration > 0) {
      seek(pct * state.duration);
    }
  }

  function onVolumeInput(e: Event) {
    const target = e.target as HTMLInputElement;
    setVolume(parseFloat(target.value));
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      exitFullscreen();
    }
    onMouseMove();
  }
</script>

<svelte:window onmousemove={onMouseMove} onkeydown={handleKeydown} />

<div class="fs-view" class:controls-hidden={!showControls}>
  <!-- Blurred background cover -->
  <div class="fs-bg">
    {#if currentCoverUrl}
      <img src={currentCoverUrl} alt="" class="fs-bg-img" />
    {/if}
  </div>
  <div class="fs-bg-overlay"></div>

  <!-- Always-visible top bar with exit button -->
  <div class="fs-topbar">
    <div class="fs-brand">
      <span class="fs-brand-mark">Y</span>
      <span class="fs-brand-name">YchoMusic</span>
    </div>
    <div class="fs-top-actions">
      <button
        type="button"
        class="fs-exit-btn"
        onclick={exitFullscreen}
        onmouseenter={onExitHover}
        onmouseleave={onExitLeave}
        aria-label="退出全屏"
        title="退出全屏 (Esc)"
      >
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 4v3a2 2 0 0 1-2 2H4" />
          <path d="M15 4v3a2 2 0 0 0 2 2h3" />
          <path d="M15 20v-3a2 2 0 0 0 2-2h3" />
          <path d="M9 20v-3a2 2 0 0 1-2-2H4" />
        </svg>
      </button>
    </div>
  </div>

  <!-- Center content -->
  <div class="fs-center">
    <div class="fs-cover-wrap">
      {#if currentCoverUrl}
        <img src={currentCoverUrl} alt="cover" class="fs-cover" />
      {:else}
        <div class="fs-cover-fallback"><span>♪</span></div>
      {/if}
    </div>

    <div class="fs-track-info">
      <h1 class="fs-title">{currentTrack?.title || '未在播放'}</h1>
      <p class="fs-artist">{currentTrack?.artist || ''}</p>
      <p class="fs-album">{currentTrack?.album || ''}</p>
    </div>
  </div>

  <!-- Bottom controls bar -->
  <div
    class="fs-controls"
    onmouseenter={() => { isHoveringControls = true; }}
    onmouseleave={() => { isHoveringControls = false; scheduleHide(); }}
  >
    <div class="fs-progress-row">
      <span class="fs-time">{formatTime($playerState.position)}</span>
      <div class="fs-progress" onclick={onProgressClick}>
        <div class="fs-progress-bar" style="width: {$playerState.duration > 0 ? ($playerState.position / $playerState.duration) * 100 : 0}%"></div>
      </div>
      <span class="fs-time">{formatTime($playerState.duration)}</span>
    </div>

    <div class="fs-buttons">
      <button type="button" class="fb-btn" onclick={() => prev()} aria-label="上一首">
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
      </button>
      {#if currentState === 'Playing'}
        <button type="button" class="fb-btn fb-play" onclick={() => pause()} aria-label="暂停">
          <svg viewBox="0 0 24 24" width="28" height="28" fill="currentColor"><path d="M6 5h4v14H6zm8 0h4v14h-4z"/></svg>
        </button>
      {:else if currentTrackId}
        <button type="button" class="fb-btn fb-play" onclick={() => togglePlayPause()} aria-label="播放">
          <svg viewBox="0 0 24 24" width="28" height="28" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
        </button>
      {/if}
      <button type="button" class="fb-btn" onclick={() => next()} aria-label="下一首">
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor"><path d="M16 6h2v12h-2zm-1.5 6L6 6v12z"/></svg>
      </button>
      <button type="button" class="fb-btn fb-mode" onclick={cycleMode} title={$playerState.mode}>
        {@html getModeSvg($playerState.mode)}
      </button>
      <div class="fb-volume">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M3 9v6h4l5 5V4L7 9H3z"/></svg>
        <input type="range" min="0" max="1" step="0.01" value={$playerState.volume} oninput={onVolumeInput} class="fs-volume" />
      </div>

      <!-- 字 (lyrics font settings) -->
      <div class="fs-lyrics-settings-wrap">
        <button
          type="button"
          class="fb-btn fs-lyrics-btn"
          onclick={() => (showLyricsSettings = !showLyricsSettings)}
          aria-label="歌词设置"
          title="歌词字体设置"
        >
          <span class="fs-lyrics-glyph">字</span>
        </button>
        {#if showLyricsSettings}
          <div class="fs-lyrics-popup" onclick={(e) => e.stopPropagation()}>
            <div class="fs-lyrics-header">
              <span>歌词字体设置</span>
              <button type="button" class="fs-lyrics-close" onclick={() => (showLyricsSettings = false)} aria-label="关闭">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                  <line x1="18" y1="6" x2="6" y2="18"/>
                  <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
              </button>
            </div>
            <div class="fs-lyrics-row">
              <label class="fs-lyrics-label">字体大小</label>
              <input
                type="range"
                min="12"
                max="32"
                step="1"
                value={$settings.lyrics_font_size || 16}
                class="fs-lyrics-slider"
                oninput={(e) => updateLyricsSetting('lyrics_font_size', (e.target as HTMLInputElement).value)}
              />
              <span class="fs-lyrics-val">{$settings.lyrics_font_size || 16}px</span>
            </div>
            <div class="fs-lyrics-row">
              <label class="fs-lyrics-label">行间距</label>
              <input
                type="range"
                min="1.5"
                max="4"
                step="0.1"
                value={$settings.lyrics_line_height || 2.2}
                class="fs-lyrics-slider"
                oninput={(e) => updateLyricsSetting('lyrics_line_height', (e.target as HTMLInputElement).value)}
              />
              <span class="fs-lyrics-val">{$settings.lyrics_line_height || 2.2}</span>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .fs-view {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: #06060a;
    overflow: hidden;
  }

  .fs-view.controls-hidden {
    cursor: none;
  }

  .fs-view.controls-hidden .fs-controls {
    opacity: 0;
    pointer-events: none;
    transform: translateY(20px);
  }

  .fs-view.controls-hidden .fs-track-info {
    opacity: 0.5;
  }

  /* Always-visible top bar */
  .fs-topbar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    transition: background 0.3s ease;
    pointer-events: auto;
  }

  .fs-brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .fs-brand-mark {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--color-accent), rgba(255, 107, 157, 0.8));
    color: #08080c;
    font-family: var(--font-display);
    font-weight: 700;
    font-size: 15px;
    box-shadow: 0 0 20px var(--color-accent-glow);
  }

  .fs-brand-name {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 0.95rem;
    color: rgba(255, 255, 255, 0.85);
    letter-spacing: -0.01em;
  }

  .fs-top-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .fs-exit-btn {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(0, 0, 0, 0.4);
    color: rgba(255, 255, 255, 0.85);
    cursor: pointer;
    transition: all 0.2s ease;
    backdrop-filter: blur(10px);
    -webkit-app-region: no-drag;
  }

  .fs-exit-btn:hover {
    background: rgba(232, 64, 69, 0.6);
    color: #fff;
    border-color: rgba(232, 64, 69, 0.8);
    box-shadow: 0 0 20px rgba(232, 64, 69, 0.4);
    transform: scale(1.05);
  }

  .fs-exit-btn:active {
    transform: scale(0.95);
  }

  /* Blurred background */
  .fs-bg {
    position: absolute;
    inset: 0;
    overflow: hidden;
  }

  .fs-bg-img {
    width: 110%;
    height: 110%;
    object-fit: cover;
    filter: blur(60px) saturate(1.5);
    transform: scale(1.1);
  }

  .fs-bg-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom, rgba(6, 6, 10, 0.6) 0%, rgba(6, 6, 10, 0.85) 100%);
  }

  /* Center content */
  .fs-center {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2rem;
    flex: 1;
    justify-content: center;
    transition: opacity 0.4s ease;
  }

  .fs-cover-wrap {
    width: 300px;
    height: 300px;
    border-radius: 16px;
    overflow: hidden;
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6), 0 0 60px var(--color-accent-glow);
  }

  .fs-cover {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .fs-cover-fallback {
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    background: linear-gradient(135deg, var(--color-accent), rgba(255, 107, 157, 0.4));
    color: #08080c;
    font-size: 4rem;
    font-weight: 700;
  }

  .fs-track-info {
    text-align: center;
    max-width: 600px;
    transition: opacity 0.4s ease;
  }

  .fs-title {
    font-family: var(--font-display);
    font-size: 1.6rem;
    font-weight: 700;
    color: #fff;
    margin: 0 0 0.3rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .fs-artist {
    font-size: 0.95rem;
    color: rgba(255, 255, 255, 0.6);
    margin: 0 0 0.1rem;
  }

  .fs-album {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.35);
    margin: 0;
  }

  /* Bottom controls */
  .fs-controls {
    position: relative;
    z-index: 2;
    width: 100%;
    max-width: 700px;
    padding: 1.5rem 2rem 2.5rem;
    transition: opacity 0.4s ease, transform 0.4s ease;
  }

  .fs-progress-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1.2rem;
  }

  .fs-time {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: rgba(255, 255, 255, 0.5);
    min-width: 36px;
    text-align: center;
  }

  .fs-progress {
    flex: 1;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    cursor: pointer;
    overflow: hidden;
  }

  .fs-progress-bar {
    height: 100%;
    background: linear-gradient(90deg, var(--color-accent), rgba(255, 107, 157, 0.8));
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .fs-buttons {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
  }

  .fb-btn {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .fb-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .fb-btn:active {
    transform: scale(0.9);
  }

  .fb-play {
    width: 56px;
    height: 56px;
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .fb-play:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .fb-mode {
    font-size: 0.9rem;
  }

  .fb-volume {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: rgba(255, 255, 255, 0.5);
    margin-left: 0.5rem;
  }

  .fs-volume {
    width: 80px;
    -webkit-appearance: none;
    height: 3px;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 2px;
    outline: none;
  }

  .fs-volume::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
  }

  /* 字 button + lyrics settings popup */
  .fs-lyrics-settings-wrap {
    position: relative;
    display: flex;
    align-items: center;
    margin-left: 0.5rem;
  }

  .fs-lyrics-btn {
    font-size: 0.9rem;
  }

  .fs-lyrics-glyph {
    font-weight: 700;
    font-family: var(--font-display, sans-serif);
    font-size: 0.85rem;
  }

  .fs-lyrics-btn:hover {
    color: var(--color-accent);
  }

  .fs-lyrics-popup {
    position: absolute;
    bottom: calc(100% + 12px);
    right: 0;
    width: 260px;
    padding: 0.9rem;
    border-radius: 12px;
    background: rgba(12, 12, 18, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(32px) saturate(180%);
    -webkit-backdrop-filter: blur(32px) saturate(180%);
    box-shadow: 0 16px 50px rgba(0, 0, 0, 0.7);
    z-index: 200;
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
    animation: fs-lyrics-in 0.18s ease-out;
  }

  @keyframes fs-lyrics-in {
    from { opacity: 0; transform: translateY(8px) scale(0.97); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  .fs-lyrics-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.82rem;
    font-weight: 600;
    color: #fff;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .fs-lyrics-close {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.6);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .fs-lyrics-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .fs-lyrics-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .fs-lyrics-label {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.6);
    width: 3.4rem;
    flex-shrink: 0;
  }

  .fs-lyrics-slider {
    flex: 1;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }

  .fs-lyrics-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: var(--color-accent);
    box-shadow: 0 0 8px var(--color-accent-glow);
    cursor: pointer;
    transition: transform 0.2s ease;
  }

  .fs-lyrics-slider::-webkit-slider-thumb:hover {
    transform: scale(1.3);
  }

  .fs-lyrics-slider::-moz-range-thumb {
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: var(--color-accent);
    border: none;
    box-shadow: 0 0 8px var(--color-accent-glow);
    cursor: pointer;
  }

  .fs-lyrics-val {
    font-size: 0.72rem;
    color: var(--color-accent);
    min-width: 2.6rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
</style>
