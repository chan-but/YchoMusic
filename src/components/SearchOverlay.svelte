<script lang="ts">
  import { invoke } from '@/lib/tauri';
  import { play, setPlaylistFromTracks } from '@/stores/player';
  import { searchOpen, closeSearch } from '@/stores/ui';
  import type { Track } from '@/types';

  // Svelte 5 callback prop (replaces createEventDispatcher)
  export let onnavigate: (path: string) => void = () => {};

  let searchQuery = '';
  let searchResults: Track[] = [];
  let selectedIndex = 0;
  let inputEl: HTMLInputElement;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  $: if ($searchOpen) {
    searchQuery = '';
    searchResults = [];
    setTimeout(() => inputEl?.focus(), 50);
  }

  $: if (searchQuery) {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => doSearch(searchQuery), 200);
  } else {
    searchResults = [];
  }

  async function doSearch(query: string) {
    const trimmed = query.trim();
    if (!trimmed) {
      searchResults = [];
      return;
    }
    try {
      const results = await invoke<Track[]>('get_tracks', {
        filter: { search: trimmed, limit: 20, offset: 0, artist: null, album: null },
      });
      console.log('[diag] search:', trimmed, 'results:', results?.length || 0);
      searchResults = results || [];
      selectedIndex = 0;
    } catch (e) {
      console.error('[diag] Search failed:', e);
      searchResults = [];
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closeSearch();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, searchResults.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter' && searchResults[selectedIndex]) {
      handleSelect(searchResults[selectedIndex]);
    }
  }

  async function handleSelect(track: Track) {
    await setPlaylistFromTracks(searchResults);
    await play(track.id);
    closeSearch();
    onnavigate('/player');
  }

  function coverUrl(cover: string | null): string {
    if (!cover || cover.length === 0) return '';
    return `data:image/jpeg;base64,${cover}`;
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

{#if $searchOpen}
  <div
    class="fixed inset-0 z-50 flex items-start justify-center pt-20 bg-black/50 backdrop-blur-sm"
    onclick={(e) => { if (e.target === e.currentTarget) closeSearch(); }}
    onkeydown={handleKeydown}
    role="dialog"
    aria-modal="true"
    aria-label="搜索"
    tabindex="-1"
  >
    <div class="glass rounded-2xl w-full max-w-xl animate-scale-in" style="max-height: 60vh;">
      <!-- Search Input -->
      <div class="flex items-center gap-3 p-4 border-b border-border">
        <span class="text-text-muted text-lg">🔍</span>
        <input
          bind:this={inputEl}
          type="text"
          class="flex-1 bg-transparent text-text-primary text-base outline-none placeholder:text-text-muted"
          placeholder="搜索歌曲、艺人、专辑..."
          bind:value={searchQuery}
          onkeydown={handleKeydown}
        />
        <kbd class="text-text-muted text-xs bg-bg-hover px-2 py-1 rounded">ESC</kbd>
      </div>

      <!-- Results -->
      <div class="overflow-y-auto scrollbar-hidden" style="max-height: calc(60vh - 70px);">
        {#if searchQuery && searchResults.length === 0}
          <div class="text-center py-12">
            <p class="text-text-muted text-sm">未找到匹配的歌曲</p>
            <p class="text-text-muted text-xs mt-1">试试其他关键词</p>
          </div>
        {:else if searchResults.length > 0}
          <div class="p-2">
            {#each searchResults as track, i (track.id)}
              <button
                type="button"
                class={`w-full flex items-center gap-3 p-2 rounded-lg cursor-pointer transition-all ${
                  i === selectedIndex ? 'bg-accent/20' : 'hover:bg-bg-hover'
                }`}
                onclick={() => handleSelect(track)}
                onmouseenter={() => (selectedIndex = i)}
              >
                <div class="w-10 h-10 rounded-md overflow-hidden flex-shrink-0 bg-gradient-to-br from-accent/30 to-text-secondary/20 flex items-center justify-center">
                  {#if track.cover_blob}
                    <img src={coverUrl(track.cover_blob)} alt="" class="w-full h-full object-cover" />
                  {:else}
                    <span class="text-sm opacity-50">🎵</span>
                  {/if}
                </div>
                <div class="flex-1 min-w-0 text-left">
                  <p class="text-text-primary text-sm truncate">{track.title || '未知歌曲'}</p>
                  <p class="text-text-secondary text-xs truncate">
                    {track.artist || '未知艺人'} · {track.album || '未知专辑'}
                  </p>
                </div>
                <span class="text-text-muted text-xs flex-shrink-0">{formatTime(track.duration)}</span>
              </button>
            {/each}
          </div>
        {:else}
          <div class="text-center py-12">
            <p class="text-text-muted text-sm">开始输入以搜索</p>
            <p class="text-text-muted text-xs mt-1">按 Ctrl+F 快速打开搜索</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
