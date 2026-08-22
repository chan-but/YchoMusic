<script lang="ts">
  import { get } from 'svelte/store';
  import { playerState, play, pause, prev, next, seek, setMode, togglePlayPause, getCurrentPlayerState } from '@/stores/player';
  import { tracks } from '@/stores/library';
  import { windowMode } from '@/stores/windowMode';
  import { setNormalMode, startDragging, toggleAlwaysOnTop } from '@/lib/window';
  import type { PlayMode, LyricLine, Track } from '@/types';

  let isAlwaysOnTop = $state(false);
  let currentLyrics: LyricLine[] = $state([]);
  let prevTrackId: number | null = null;

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

  // Current lyric line based on playback position
  let currentLyricText = $derived.by(() => {
    if (currentLyrics.length === 0) return '';
    const pos = $playerState.position;
    let line = '';
    for (const l of currentLyrics) {
      if (l.time <= pos) line = l.text;
      else break;
    }
    return line;
  });

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function parseLrc(lrc: string): LyricLine[] {
    const lines: LyricLine[] = [];
    const regex = /\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)/g;
    let match;
    while ((match = regex.exec(lrc)) !== null) {
      const time = parseInt(match[1]) * 60 + parseInt(match[2]) + parseInt(match[3]) / 1000;
      lines.push({ time, text: match[4].trim() });
    }
    return lines.sort((a, b) => a.time - b.time);
  }

  function loadLyrics(track: Track | undefined) {
    if (track?.lyrics) {
      currentLyrics = parseLrc(track.lyrics);
    } else {
      currentLyrics = [];
    }
  }

  // React to track changes
  $effect(() => {
    const tid = $playerState.current_track_id;
    if (tid && tid !== prevTrackId) {
      prevTrackId = tid;
      const track = get(tracks).find(t => t.id === tid);
      loadLyrics(track);
    } else if (!tid) {
      prevTrackId = null;
      currentLyrics = [];
    }
  });

  const modeIcons: Record<string, string> = {
    RepeatList: '🔁',
    RepeatOne: '🔂',
    Shuffle: '🔀',
  };

  function cycleMode() {
    const modes = ['repeat_list', 'shuffle', 'repeat_one'];
    const state = getCurrentPlayerState();
    const cur = state.mode === 'Shuffle' ? 'shuffle' : state.mode === 'RepeatOne' ? 'repeat_one' : 'repeat_list';
    const currentIdx = modes.indexOf(cur);
    const nextMode = modes[(currentIdx + 1) % modes.length];
    setMode(nextMode);
  }

  async function exitMini() {
    try {
      await setNormalMode();
    } catch {}
    windowMode.set('normal');
  }

  async function handleTogglePin() {
    isAlwaysOnTop = await toggleAlwaysOnTop();
  }

  function onProgressClick(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const pct = (e.clientX - rect.left) / rect.width;
    const state = getCurrentPlayerState();
    if (state.duration > 0) {
      seek(pct * state.duration);
    }
  }

  function handleMouseDown(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.closest('button') || target.closest('.mini-progress')) {
      return;
    }
    startDragging();
  }

  function handlePlayPause() {
    togglePlayPause();
  }
</script>

<div class="mini-view" onmousedown={handleMouseDown}>
  <!-- Drag region -->
  <div class="mini-drag" data-tauri-drag-region></div>

  <!-- Always-on-top pin button -->
  <button
    type="button"
    class={`mini-pin ${isAlwaysOnTop ? 'pinned' : ''}`}
    onclick={handleTogglePin}
    aria-label={isAlwaysOnTop ? '取消置顶' : '始终置顶'}
    title={isAlwaysOnTop ? '取消置顶' : '始终置顶'}
  >
    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M12 17v5"/>
      <path d="M9 10.5V3h6v7.5l2 2.5H7l2-2.5z"/>
    </svg>
  </button>

  {#if currentTrack}
    <div class="mini-body">
      <div class="mini-cover">
        {#if currentCoverUrl}
          <img src={currentCoverUrl} alt="cover" />
        {:else}
          <div class="mini-cover-fallback">♪</div>
        {/if}
      </div>

      <div class="mini-info">
        <div class="mini-meta-row">
          <span class="mini-title">{currentTrack?.title || '未知歌曲'}</span>
          <button type="button" class="mini-expand" onclick={exitMini} title="展开">
            <svg viewBox="0 0 16 16" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
              <path d="M4 2H2.5A.5.5 0 0 0 2 2.5V4" />
              <path d="M12 2h1.5a.5.5 0 0 1 .5.5V4" />
              <path d="M14 12v1.5a.5.5 0 0 1-.5.5H12" />
              <path d="M4 14H2.5a.5.5 0 0 1-.5-.5V12" />
              <line x1="6" y1="6" x2="10" y2="10" />
              <line x1="10" y1="6" x2="6" y2="10" />
            </svg>
          </button>
        </div>
        <span class="mini-artist">{currentTrack?.artist || '未知艺人'}</span>

        <!-- Single-line lyrics -->
        <div class="mini-lyrics-row">
          {#if currentLyricText}
            <span class="mini-lyrics-text" key={currentLyricText}>{currentLyricText}</span>
          {:else}
            <span class="mini-lyrics-text muted">♪</span>
          {/if}
        </div>

        <div class="mini-progress" onclick={onProgressClick}>
          <div class="mini-progress-bar" style="width: {$playerState.duration > 0 ? ($playerState.position / $playerState.duration) * 100 : 0}%"></div>
        </div>

        <div class="mini-controls">
          <button type="button" class="mc-btn" onclick={() => prev()} aria-label="上一首">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
          </button>
          {#if currentState === 'Playing'}
            <button type="button" class="mc-btn mc-play" onclick={() => pause()} aria-label="暂停">
              <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M6 5h4v14H6zm8 0h4v14h-4z"/></svg>
            </button>
          {:else if currentTrackId}
            <button type="button" class="mc-btn mc-play" onclick={handlePlayPause} aria-label="播放">
              <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
            </button>
          {/if}
          <button type="button" class="mc-btn" onclick={() => next()} aria-label="下一首">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M16 6h2v12h-2zm-1.5 6L6 6v12z"/></svg>
          </button>
          <button type="button" class="mc-btn mc-mode" onclick={cycleMode} aria-label="播放模式" title={$playerState.mode}>
            {modeIcons[$playerState.mode] || '➡'}
          </button>
        </div>
      </div>
    </div>
  {:else}
    <div class="mini-empty">
      <span class="mini-empty-glyph">♪</span>
      <span class="mini-empty-text">未在播放</span>
      <button type="button" class="mini-expand" onclick={exitMini}>展开</button>
    </div>
  {/if}
</div>

<style>
  .mini-view {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(12, 12, 18, 0.95);
    backdrop-filter: blur(30px);
    overflow: hidden;
    position: relative;
  }

  .mini-drag {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 24px;
    -webkit-app-region: drag;
  }

  .mini-pin {
    position: absolute;
    top: 3px;
    right: 4px;
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 6px;
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
    -webkit-app-region: no-drag;
    z-index: 10;
  }

  .mini-pin:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-primary);
  }

  .mini-pin.pinned {
    color: var(--color-accent);
    filter: drop-shadow(0 0 4px var(--color-accent-glow));
  }

  .mini-body {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    padding: 0 0.6rem;
    width: 100%;
  }

  .mini-cover {
    width: 56px;
    height: 56px;
    border-radius: 10px;
    overflow: hidden;
    flex-shrink: 0;
    background: linear-gradient(135deg, var(--color-accent), rgba(255, 107, 157, 0.4));
    display: grid;
    place-items: center;
  }

  .mini-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .mini-cover-fallback {
    color: #08080c;
    font-size: 1.5rem;
    font-weight: 700;
  }

  .mini-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .mini-meta-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .mini-title {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .mini-artist {
    font-size: 0.68rem;
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mini-lyrics-row {
    height: 16px;
    overflow: hidden;
    display: flex;
    align-items: center;
  }

  .mini-lyrics-text {
    font-size: 0.66rem;
    color: var(--color-accent);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    width: 100%;
    animation: lyric-fade 0.3s ease;
  }

  .mini-lyrics-text.muted {
    color: var(--color-text-muted);
    opacity: 0.5;
  }

  @keyframes lyric-fade {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .mini-progress {
    height: 3px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 2px;
    cursor: pointer;
    margin-top: 1px;
  }

  .mini-progress-bar {
    height: 100%;
    background: linear-gradient(90deg, var(--color-accent), rgba(255, 107, 157, 0.8));
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .mini-controls {
    display: flex;
    align-items: center;
    gap: 0.15rem;
    margin-top: 1px;
  }

  .mc-btn {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
    -webkit-app-region: no-drag;
  }

  .mc-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-primary);
  }

  .mc-btn:active {
    transform: scale(0.9);
  }

  .mc-play {
    width: 26px;
    height: 26px;
    color: var(--color-accent);
  }

  .mc-mode {
    font-size: 0.7rem;
    margin-left: 2px;
  }

  .mini-expand {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--color-text-muted);
    cursor: pointer;
    flex-shrink: 0;
    transition: all 0.15s ease;
    -webkit-app-region: no-drag;
  }

  .mini-expand:hover {
    color: var(--color-accent);
    background: rgba(255, 255, 255, 0.12);
    border-color: var(--color-accent);
  }

  .mini-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.3rem;
    color: var(--color-text-muted);
  }

  .mini-empty-glyph {
    font-size: 1.5rem;
    opacity: 0.4;
  }

  .mini-empty-text {
    font-size: 0.7rem;
  }
</style>
