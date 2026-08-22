<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { applyTheme, getSettings, applySettingsToRuntime } from '@/stores/settings';
  import { settings } from '@/stores/settings';
  import { searchOpen, openSearch, closeSearch } from '@/stores/ui';
  import { currentRoute, navigate } from '@/stores/router';
  import { playerState, play, pause, resume, togglePlayPause, prev, next, seek, setVolume, setMode, startPositionUpdate, stopPositionUpdate, getCurrentPlayerState, playNow } from '@/stores/player';
  import { tracks, getTracks, getAlbums, getArtists } from '@/stores/library';
  import { windowMode } from '@/stores/windowMode';
  import { invoke } from '@/lib/tauri';
  import { getQualityTier } from '@/lib/quality';
  import SearchOverlay from '@/components/SearchOverlay.svelte';
  import WindowControls from '@/components/WindowControls.svelte';
  import MiniPlayerView from '@/components/MiniPlayerView.svelte';
  import FullscreenPlayer from '@/components/FullscreenPlayer.svelte';
  import Home from '@/routes/Home.svelte';
  import Player from '@/routes/Player.svelte';
  import Songs from '@/routes/Songs.svelte';
  import Albums from '@/routes/Albums.svelte';
  import Artists from '@/routes/Artists.svelte';
  import MusicTags from '@/routes/MusicTags.svelte';
  import Stats from '@/routes/Stats.svelte';
  import SettingsPage from '@/routes/Settings.svelte';
  import PlaylistEditor from '@/routes/PlaylistEditor.svelte';
  import PlaylistList from '@/routes/PlaylistList.svelte';

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function cyclePlayMode() {
    const modes = ['repeat_list', 'shuffle', 'repeat_one'];
    const state = getCurrentPlayerState();
    const cur = state.mode === 'Shuffle' ? 'shuffle' : state.mode === 'RepeatOne' ? 'repeat_one' : 'repeat_list';
    const currentIndex = modes.indexOf(cur);
    const nextMode = modes[(currentIndex + 1) % modes.length];
    setMode(nextMode);
  }

  function getPlayModeIcon(): string {
    const state = getCurrentPlayerState();
    switch (state.mode) {
      case 'Shuffle': return 'shuffle';
      case 'RepeatOne': return 'repeat-one';
      case 'RepeatList': return 'repeat-list';
      default: return 'repeat-list';
    }
  }

  // Volume popup
  let showVolumePopup = $state(false);
  let volumePopupTimer: ReturnType<typeof setTimeout> | null = null;

  // Home playlist overlay
  let showHomePlaylistOverlay = $state(false);
  let homePlaylistSearchQuery = $state('');

  function handleMiniVolume(e: Event) {
    const target = e.target as HTMLInputElement;
    setVolume(parseFloat(target.value));
    showVolumePopup = true;
    if (volumePopupTimer) clearTimeout(volumePopupTimer);
    volumePopupTimer = setTimeout(() => { showVolumePopup = false; }, 1500);
  }

  function handleMiniSeek(e: MouseEvent) {
    const target = e.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percentage = x / rect.width;
    const state = getCurrentPlayerState();
    seek(percentage * state.duration);
  }

  function handleKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const tag = target?.tagName;
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || target?.isContentEditable;

    if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K' || e.key === 'f' || e.key === 'F')) {
      e.preventDefault();
      if ($searchOpen) {
        closeSearch();
      } else {
        openSearch();
      }
    } else if (e.key === 'Escape' && $searchOpen) {
      closeSearch();
    } else if (e.key === ' ' && !isInput) {
      e.preventDefault();
      console.log('[diag] spacebar pressed, toggling play/pause');
      togglePlayPause();
    }
  }

  onMount(async () => {
    window.addEventListener('keydown', handleKeydown);

    const topBar = document.querySelector('.top-bar') as HTMLElement | null;
    if (topBar) {
      topBar.setAttribute('data-tauri-drag-region', '');
    }

    const bar = document.getElementById('boot-bar') as HTMLElement | null;
    const hint = document.getElementById('boot-hint') as HTMLElement | null;
    const splash = document.getElementById('boot-splash');

    function setProgress(pct: number) {
      if (bar) bar.style.width = pct + '%';
    }

    function setHint(text: string) {
      if (hint) hint.textContent = text;
    }

    function removeSplash() {
      if (splash) {
        splash.classList.add('is-hidden');
        setTimeout(() => splash.remove(), 400);
      }
    }

    setProgress(15);
    setHint('LOADING SETTINGS…');

    try {
      const s = await getSettings();
      applyTheme(s.theme);
      setProgress(55);
    } catch (e) {
      console.warn('[startup] Settings load failed:', e);
      setProgress(55);
    }

    setHint('LOADING LIBRARY…');
    setProgress(75);

    // Start library loading in parallel
    const libraryPromise = Promise.all([
      getTracks().catch(() => []),
      getAlbums().catch(() => []),
      getArtists().catch(() => []),
    ]);

    // Show UI immediately — don't block full library load
    setProgress(100);
    removeSplash();

    try {
      await applySettingsToRuntime();
    } catch {}

    // Now show the window
    try {
      await invoke('show_main_window');
    } catch {}

    // Start position update after UI is visible
    startPositionUpdate();

    // Wait for library in background (pages will handle showing loading states)
    try {
      await libraryPromise;
    } catch (e) {
      console.warn('[startup] Library load failed:', e);
    }
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
    stopPositionUpdate();
  });

  // Routes that render as a full-screen overlay on top of Home.
  const overlayRoutes = ['/player', '/songs', '/albums', '/artists', '/tags', '/stats', '/settings'];

  let isOverlay = $derived.by(() => overlayRoutes.includes($currentRoute) || $currentRoute.startsWith('/songs') || $currentRoute.startsWith('/playlist-list') || $currentRoute.startsWith('/playlist-editor'));

  let currentTrack = $derived.by(() => {
    const id = $playerState.current_track_id;
    if (!id) return null;
    return $tracks.find(t => t.id === id) || null;
  });

  let currentCoverUrl = $derived.by(() => {
    const t = currentTrack;
    if (!t?.cover_blob) return '';
    return `data:image/jpeg;base64,${t.cover_blob}`;
  });

  let currentTitle = $derived.by(() => currentTrack?.title || '未知歌曲');
  let currentArtist = $derived.by(() => currentTrack?.artist || '未知艺人');

  let filteredHomePlaylist = $derived.by(() => {
    const all = $tracks;
    if (!homePlaylistSearchQuery.trim()) return all;
    const q = homePlaylistSearchQuery.trim().toLowerCase();
    return all.filter(t =>
      (t.title || '').toLowerCase().includes(q) ||
      (t.artist || '').toLowerCase().includes(q) ||
      (t.album || '').toLowerCase().includes(q)
    );
  });

  async function scrollHomePlaylistToCurrent() {
    const overlay = document.querySelector('.home-playlist-overlay .playlist-items');
    if (!overlay) return;
    const state = getCurrentPlayerState();
    const trackId = state.current_track_id;
    if (!trackId) return;
    const list = filteredHomePlaylist;
    const idx = list.findIndex(t => t.id === trackId);
    if (idx < 0) return;
    const items = overlay.querySelectorAll('.playlist-item');
    if (items.length === 0 || items.length <= idx) return;
    const itemHeight = items[idx].getBoundingClientRect().height;
    const containerHeight = overlay.clientHeight;
    const scrollTop = idx * itemHeight - containerHeight * 0.3;
    overlay.scrollTo({ top: Math.max(0, scrollTop), behavior: 'smooth' });
    items.forEach((item, i) => item.classList.toggle('current', i === idx));
  }

  async function playFromHomeOverlay(trackId: number) {
    try {
      await playNow(trackId);
    } catch (e) {
      console.error('Failed to play:', e);
    }
  }
</script>

<div class="app-shell" class:mini-mode={$windowMode === 'mini'} class:fs-mode={$windowMode === 'fullscreen'}>
  {#if $windowMode === 'mini'}
    <MiniPlayerView />
  {:else if $windowMode === 'fullscreen'}
    <FullscreenPlayer />
  {:else}
  <!-- Ambient background layer (persists across all routes) -->
  <div class="ambient-bg" aria-hidden="true">
    <div class="ambient-orb ambient-orb-a"></div>
    <div class="ambient-orb ambient-orb-b"></div>
    <div class="ambient-orb ambient-orb-c"></div>
    <div class="ambient-grain"></div>
    <div class="ambient-vignette"></div>
  </div>

  <!-- Floating top bar -->
  <header class="top-bar" data-tauri-drag-region>
    <button
      type="button"
      class="brand"
      onclick={() => navigate('/')}
      aria-label="YchoMusic 首页"
    >
      <span class="brand-mark">Y</span>
      <span class="brand-name">YchoMusic</span>
    </button>
    <button
      type="button"
      class="top-settings-btn"
      onclick={() => navigate('/settings')}
      aria-label="设置"
      title="设置"
    >
      <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
    </button>
    <WindowControls />
  </header>

  <!-- Home is always rendered as the base layer -->
  <main class="main-stage">
    <div class="home-layer" class:is-hidden={isOverlay}>
      <Home />
    </div>

    {#if $currentRoute === '/player'}
      <div class="overlay-layer"><Player /></div>
    {:else if $currentRoute.startsWith('/songs')}
      <div class="overlay-layer"><Songs /></div>
    {:else if $currentRoute === '/albums'}
      <div class="overlay-layer"><Albums /></div>
    {:else if $currentRoute === '/artists'}
      <div class="overlay-layer"><Artists /></div>
    {:else if $currentRoute === '/tags'}
      <div class="overlay-layer"><MusicTags /></div>
    {:else if $currentRoute === '/stats'}
      <div class="overlay-layer"><Stats /></div>
    {:else if $currentRoute === '/settings'}
      <div class="overlay-layer"><SettingsPage /></div>
    {:else if $currentRoute.startsWith('/playlist-editor')}
      <div class="overlay-layer"><PlaylistEditor /></div>
    {:else if $currentRoute.startsWith('/playlist-list')}
      <div class="overlay-layer"><PlaylistList /></div>
    {/if}
  </main>

  <!-- Floating mini player with full controls (only on non-player routes) -->
  {#if $playerState.current_track_id && $currentRoute !== '/player'}
    <div class="mini-player-bar">
      <!-- Track info -->
      <button
        type="button"
        class="mini-player-info"
        onclick={() => navigate('/player')}
        aria-label="打开播放界面"
      >
        <div class="mini-cover">
          {#if currentCoverUrl}
            <img src={currentCoverUrl} alt="" class="mini-cover-img" />
          {:else}
            <span class="mini-cover-glyph">♪</span>
          {/if}
        </div>
        <div class="mini-meta">
          <p class="mini-title">{currentTitle}</p>
          <p class="mini-artist">
            {currentArtist}
            {#if currentTrack && $settings.show_audio_quality}
              {#each [getQualityTier(currentTrack)] as q}
                <span class={`quality-pill quality-${q.cls}`}>{q.label}</span>
              {/each}
            {/if}
          </p>
        </div>
      </button>

      <!-- Controls -->
      <div class="mini-controls">
        <button type="button" class="mini-ctrl-btn" onclick={cyclePlayMode} title={$playerState.mode} aria-label="播放模式">
          {#if getPlayModeIcon() === 'shuffle'}
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/></svg>
          {:else if getPlayModeIcon() === 'repeat-one'}
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/><text x="12" y="16" text-anchor="middle" font-size="7" fill="currentColor" stroke="none">1</text></svg>
          {:else}
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/></svg>
          {/if}
        </button>
        <button type="button" class="mini-ctrl-btn" onclick={() => prev()} aria-label="上一首">
          <span>⏮</span>
        </button>
        <button type="button" class="mini-play-btn" onclick={togglePlayPause} aria-label={$playerState.state === 'Playing' ? '暂停' : '播放'}>
          <span>{$playerState.state === 'Playing' ? '⏸' : '▶'}</span>
        </button>
        <button type="button" class="mini-ctrl-btn" onclick={() => next()} aria-label="下一首">
          <span>⏭</span>
        </button>
        <button type="button" class="mini-ctrl-btn" onclick={() => { showHomePlaylistOverlay = !showHomePlaylistOverlay; if (showHomePlaylistOverlay) setTimeout(scrollHomePlaylistToCurrent, 150); }} aria-label="播放列表" title="播放列表">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="3" y1="6" x2="21" y2="6"/>
            <line x1="3" y1="12" x2="15" y2="12"/>
            <line x1="3" y1="18" x2="11" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Progress -->
      <div class="mini-progress-wrap">
        <span class="mini-time">{formatTime($playerState.position)}</span>
        <div class="mini-progress-track" onclick={handleMiniSeek} role="slider" tabindex="0" aria-label="播放进度">
          <div class="mini-progress-fill" style={`width: ${$playerState.duration > 0 ? ($playerState.position / $playerState.duration) * 100 : 0}%`}></div>
        </div>
        <span class="mini-time">{formatTime($playerState.duration)}</span>
      </div>

      <!-- Volume -->
      <div class="mini-volume-wrap" class:show-popup={showVolumePopup}>
        <div class="volume-popup" class:visible={showVolumePopup}>{Math.round($playerState.volume * 100)}</div>
        <span>🔊</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={$playerState.volume}
          oninput={handleMiniVolume}
          class="mini-volume-slider"
          aria-label="音量"
        />
      </div>

      <!-- Play mode label -->
      <div class="mini-mode-label">
        {$playerState.mode === 'Shuffle' ? '随机' : $playerState.mode === 'RepeatOne' ? '单曲循环' : '列表循环'}
      </div>

      <button type="button" class="mini-close-btn" onclick={() => navigate('/player')} aria-label="展开">
        <span>⤢</span>
      </button>
    </div>
  {/if}

  {#if $searchOpen}
    <SearchOverlay onnavigate={(path) => navigate(path)} />
  {/if}

  {#if showHomePlaylistOverlay && $playerState.current_track_id}
    <div class="home-playlist-overlay" onclick={(e) => { if (e.target === e.currentTarget) showHomePlaylistOverlay = false; }}>
      <div class="home-playlist-panel">
        <div class="playlist-overlay-header">
          <h3 class="playlist-title">播放列表</h3>
          <div class="playlist-header-actions">
            <button type="button" class="playlist-action-btn" onclick={scrollHomePlaylistToCurrent} title="定位到当前歌曲">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"/>
                <path d="M12 2v3M12 19v3M2 12h3M19 12h3"/>
              </svg>
              <span>定位</span>
            </button>
            <button type="button" class="playlist-close-btn" onclick={() => (showHomePlaylistOverlay = false)} aria-label="关闭">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
        </div>
        <div class="playlist-search-bar">
          <input type="text" class="playlist-search-input" placeholder="搜索..." bind:value={homePlaylistSearchQuery} />
        </div>
        <div class="playlist-items">
          {#each filteredHomePlaylist as track, i (track.id)}
            <div class="playlist-item" class:current={track.id === $playerState.current_track_id} onclick={() => playFromHomeOverlay(track.id)}>
              <span class="playlist-item-num">{i + 1}</span>
              <span class="playlist-item-title">{track.title || '未知'}</span>
              <span class="playlist-item-artist">{track.artist || ''}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
  {/if}
</div>

<style>
  .app-shell {
    position: relative;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
  }

  /* Ensure html/body never show white flash during mode switching */
  :global(html), :global(body) {
    background: var(--color-bg-primary) !important;
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  /* === Ambient background === */
  .ambient-bg {
    position: fixed;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    overflow: hidden;
  }

  .ambient-orb {
    position: absolute;
    border-radius: 50%;
    filter: blur(80px);
    opacity: 0.5;
    will-change: transform;
  }

  .ambient-orb-a {
    width: 50vw;
    height: 50vw;
    top: -10%;
    left: -10%;
    background: radial-gradient(circle, var(--color-accent-glow), transparent 60%);
    animation: drift-a 28s ease-in-out infinite alternate;
  }

  .ambient-orb-b {
    width: 45vw;
    height: 45vw;
    bottom: -15%;
    right: -10%;
    background: radial-gradient(circle, rgba(255, 107, 157, 0.18), transparent 60%);
    animation: drift-b 34s ease-in-out infinite alternate;
  }

  .ambient-orb-c {
    width: 35vw;
    height: 35vw;
    top: 30%;
    right: 20%;
    background: radial-gradient(circle, var(--color-accent-glow), transparent 60%);
    opacity: 0.25;
    animation: drift-c 40s ease-in-out infinite alternate;
  }

  .ambient-grain {
    position: absolute;
    inset: 0;
    opacity: 0.04;
    mix-blend-mode: overlay;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='200' height='200'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/></filter><rect width='100%25' height='100%25' filter='url(%23n)' opacity='0.6'/></svg>");
  }

  .ambient-vignette {
    position: absolute;
    inset: 0;
    background: radial-gradient(ellipse at center, transparent 40%, rgba(0, 0, 0, 0.55) 100%);
  }

  /* === Top bar === */
  .top-bar {
    position: relative;
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 1rem;
    height: 56px;
    padding: 0 1.25rem;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.4rem 0.6rem 0.4rem 0.4rem;
    border-radius: var(--radius-full);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: transform 0.2s ease;
  }

  .brand:hover {
    transform: translateY(-1px);
  }

  .brand-mark {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--color-accent), rgba(255, 107, 157, 0.8));
    color: #08080c;
    font-family: var(--font-display);
    font-weight: 700;
    font-size: 18px;
    box-shadow: 0 4px 16px var(--color-accent-glow), inset 0 1px 0 rgba(255, 255, 255, 0.3);
  }

  .brand-name {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 1.05rem;
    letter-spacing: -0.01em;
    background: linear-gradient(135deg, var(--color-text-primary), var(--color-text-secondary));
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 1rem;
    border-radius: var(--radius-full);
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border);
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.8rem;
    transition: all 0.25s ease;
  }

  .back-btn:hover {
    color: var(--color-text-primary);
    border-color: var(--color-accent);
    box-shadow: 0 0 20px var(--color-accent-glow);
  }

  .back-btn span:first-child {
    display: inline-block;
    transform: rotate(180deg);
    font-size: 0.9rem;
  }

  /* === Main stage === */
  .main-stage {
    position: relative;
    z-index: 10;
    height: calc(100vh - 56px);
    overflow: hidden;
  }

  .home-layer {
    position: absolute;
    inset: 0;
    transition: opacity 0.5s ease, transform 0.5s ease, filter 0.5s ease;
  }

  .home-layer.is-hidden {
    opacity: 0;
    transform: scale(0.96);
    filter: blur(8px);
    pointer-events: none;
  }

  .overlay-layer {
    position: absolute;
    inset: 0;
    animation: overlay-in 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  }

  /* === Mini player bar with controls === */
  .mini-player-bar {
    position: fixed;
    bottom: 1rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 40;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 1rem;
    border-radius: var(--radius-full);
    background: rgba(12, 12, 18, 0.85);
    border: 1px solid var(--color-border);
    backdrop-filter: blur(32px) saturate(200%);
    box-shadow: 0 16px 50px rgba(0, 0, 0, 0.5), 0 0 30px var(--color-accent-glow);
    max-width: 720px;
    width: calc(100% - 2rem);
  }

  .mini-player-info {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
  }

  .mini-player-info:hover .mini-title {
    color: var(--color-accent);
  }

  .mini-cover {
    position: relative;
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 12px;
    background: linear-gradient(135deg, var(--color-accent), rgba(255, 107, 157, 0.6));
    color: #08080c;
    flex-shrink: 0;
    overflow: hidden;
  }

  .mini-cover-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    border-radius: 12px;
  }

  .mini-cover-glyph {
    font-size: 1.1rem;
    font-weight: 700;
  }

  .mini-meta {
    min-width: 0;
    text-align: left;
  }

  .mini-title {
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 140px;
    transition: color 0.2s ease;
  }

  .mini-artist {
    font-size: 0.7rem;
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 140px;
  }

  .mini-controls {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex-shrink: 0;
  }

  .mini-ctrl-btn {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 0.75rem;
  }

  .mini-ctrl-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .mini-play-btn {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--color-accent), var(--color-accent-deep));
    border: none;
    color: #08080c;
    cursor: pointer;
    transition: all 0.25s ease;
    box-shadow: 0 4px 16px var(--color-accent-glow);
    flex-shrink: 0;
    font-size: 0.9rem;
  }

  .mini-play-btn:hover {
    transform: scale(1.08);
  }

  .mini-progress-wrap {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    min-width: 0;
  }

  .mini-time {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--color-text-muted);
    min-width: 2.5rem;
    text-align: center;
    flex-shrink: 0;
  }

  .mini-progress-track {
    position: relative;
    flex: 1;
    height: 4px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 2px;
    cursor: pointer;
    min-width: 40px;
  }

  .mini-progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--color-accent-deep), var(--color-accent));
    border-radius: 2px;
    transition: width 0.1s linear;
  }

  .mini-volume-wrap {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    flex-shrink: 0;
    font-size: 0.7rem;
    color: var(--color-text-muted);
    position: relative;
  }

  .volume-popup {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%) scale(0.8);
    padding: 4px 10px;
    border-radius: 8px;
    background: rgba(12, 12, 18, 0.95);
    border: 1px solid var(--color-border);
    color: var(--color-accent);
    font-size: 0.75rem;
    font-weight: 600;
    font-family: var(--font-mono);
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.15s ease, transform 0.15s ease;
    white-space: nowrap;
    z-index: 50;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
  }
  .volume-popup.visible {
    opacity: 1;
    transform: translateX(-50%) scale(1);
  }
  .volume-popup::after {
    content: '';
    position: absolute;
    top: 100%;
    left: 50%;
    transform: translateX(-50%);
    border: 4px solid transparent;
    border-top-color: rgba(12, 12, 18, 0.95);
  }

  .mini-volume-slider {
    width: 56px;
    height: 3px;
    -webkit-appearance: none;
    appearance: none;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }

  .mini-volume-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
  }

  .mini-mode-label {
    font-size: 0.7rem;
    color: var(--color-text-muted);
    padding: 0.2rem 0.5rem;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.04);
    flex-shrink: 0;
  }

  .mini-close-btn {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all 0.2s ease;
    flex-shrink: 0;
    font-size: 0.7rem;
  }

  .mini-close-btn:hover {
    color: var(--color-accent);
    border-color: var(--color-accent);
  }

  @keyframes drift-a {
    0% { transform: translate(0, 0) scale(1); }
    100% { transform: translate(8%, 12%) scale(1.15); }
  }
  @keyframes drift-b {
    0% { transform: translate(0, 0) scale(1); }
    100% { transform: translate(-10%, -8%) scale(1.1); }
  }
  @keyframes drift-c {
    0% { transform: translate(0, 0) scale(1); }
    100% { transform: translate(-6%, 10%) scale(0.9); }
  }

  @keyframes overlay-in {
    from { opacity: 0; transform: translateY(20px) scale(0.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  .quality-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 16px;
    padding: 0 7px;
    margin-left: 8px;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    line-height: 1;
    vertical-align: middle;
  }
  .quality-muted { background: transparent; color: rgba(255,255,255,0.25); }
  .quality-sq    { background: rgba(255,255,255,0.06); color: rgba(255,255,255,0.55); }
  .quality-hq    { background: rgba(0,200,255,0.12); color: #49dcff; border: 1px solid rgba(0,200,255,0.22); }
  .quality-hqp   { background: rgba(170,120,255,0.12); color: #c29bff; border: 1px solid rgba(170,120,255,0.22); }
  .quality-flac  { background: rgba(70,220,140,0.12); color: #6fe6a5; border: 1px solid rgba(70,220,140,0.22); }
  .quality-hr    { background: linear-gradient(90deg, rgba(255,200,80,0.14), rgba(255,230,120,0.16)); color: #ffcf66; border: 1px solid rgba(255,200,80,0.3); }

  .mini-player-bar .quality-pill {
    height: 13px;
    padding: 0 6px;
    margin-left: 6px;
    font-size: 9px;
  }

  /* === Top settings button === */
  .top-settings-btn {
    position: absolute;
    right: 140px;
    top: 50%;
    transform: translateY(-50%);
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: rgba(255,255,255,0.5);
    cursor: pointer;
    display: grid;
    place-items: center;
    border-radius: 8px;
    transition: all 0.2s;
    z-index: 10;
  }
  .top-settings-btn:hover {
    color: var(--color-accent);
    background: rgba(255,255,255,0.05);
  }

  /* === Home playlist overlay === */
  .home-playlist-overlay {
    position: fixed;
    inset: 0;
    z-index: 500;
    background: rgba(0,0,0,0.4);
    backdrop-filter: blur(8px);
    display: flex;
    justify-content: flex-end;
  }
  .home-playlist-panel {
    width: 400px;
    max-width: 90vw;
    height: 100%;
    background: var(--color-bg-secondary);
    border-left: 1px solid rgba(255,255,255,0.06);
    display: flex;
    flex-direction: column;
  }
  .home-playlist-panel .playlist-overlay-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid rgba(255,255,255,0.06);
  }
  .home-playlist-panel .playlist-overlay-header .playlist-title {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: rgba(255,255,255,0.9);
  }
  .home-playlist-panel .playlist-header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .home-playlist-panel .playlist-action-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border: none;
    background: rgba(255,255,255,0.05);
    color: rgba(255,255,255,0.6);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.75rem;
    transition: all 0.2s;
  }
  .home-playlist-panel .playlist-action-btn:hover {
    background: rgba(255,255,255,0.1);
    color: rgba(255,255,255,0.9);
  }
  .home-playlist-panel .playlist-close-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: rgba(255,255,255,0.05);
    color: rgba(255,255,255,0.6);
    border-radius: 6px;
    cursor: pointer;
    display: grid;
    place-items: center;
  }
  .home-playlist-panel .playlist-close-btn:hover {
    background: rgba(255,255,255,0.1);
    color: rgba(255,255,255,0.9);
  }
  .home-playlist-panel .playlist-search-bar {
    padding: 8px 20px;
    border-bottom: 1px solid rgba(255,255,255,0.06);
  }
  .home-playlist-panel .playlist-search-input {
    width: 100%;
    padding: 6px 10px;
    border: 1px solid rgba(255,255,255,0.08);
    background: rgba(0,0,0,0.2);
    color: rgba(255,255,255,0.9);
    border-radius: 6px;
    font-size: 0.8rem;
    outline: none;
  }
  .home-playlist-panel .playlist-items {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }
  .home-playlist-panel .playlist-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 20px;
    cursor: pointer;
    transition: background 0.15s;
  }
  .home-playlist-panel .playlist-item:hover {
    background: rgba(255,255,255,0.04);
  }
  .home-playlist-panel .playlist-item.current {
    background: rgba(0,212,255,0.08);
  }
  .home-playlist-panel .playlist-item .playlist-item-num {
    width: 24px;
    text-align: right;
    color: rgba(255,255,255,0.3);
    font-size: 0.75rem;
    font-family: monospace;
  }
  .home-playlist-panel .playlist-item .playlist-item-title {
    flex: 1;
    color: rgba(255,255,255,0.8);
    font-size: 0.85rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .home-playlist-panel .playlist-item .playlist-item-artist {
    color: rgba(255,255,255,0.4);
    font-size: 0.75rem;
  }
  .home-playlist-panel .playlist-item.current .playlist-item-title {
    color: var(--color-accent);
  }
</style>
