<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { playerState, seek } from '@/stores/player';
  import type { LyricLine } from '@/types';

  export let lyrics: LyricLine[] = [];
  let currentIndex = -1;
  let lastActiveIndex = -1;
  let container: HTMLDivElement;
  let listEl: HTMLDivElement;
  let userScrollTimer: ReturnType<typeof setTimeout> | null = null;
  let userIsScrolling = false;
  let unsubscribePlayerState: (() => void) | null = null;

  function scrollToCurrent() {
    if (!container || !listEl) return;
    const activeLine = listEl.children[lastActiveIndex] as HTMLElement;
    if (activeLine) {
      const containerRect = container.getBoundingClientRect();
      const lineRect = activeLine.getBoundingClientRect();
      const offset = lineRect.top - containerRect.top - container.clientHeight / 2 + activeLine.clientHeight / 2;
      container.scrollBy({ top: offset, behavior: 'smooth' });
    }
  }

  function updateActiveIndex() {
    const state = get(playerState);
    if (!lyrics.length || state.duration <= 0) return;
    const pos = state.position;
    let newIndex = -1;
    for (let i = 0; i < lyrics.length; i++) {
      if (pos >= lyrics[i].time) {
        if (i === lyrics.length - 1 || pos < lyrics[i + 1].time) {
          newIndex = i;
          break;
        }
      }
    }
    if (newIndex !== -1 && newIndex !== currentIndex) {
      currentIndex = newIndex;
    }
    if (newIndex !== -1 && newIndex !== lastActiveIndex) {
      lastActiveIndex = newIndex;
      if (!userIsScrolling) {
        scrollToCurrent();
      }
    }
  }

  function handleUserScroll() {
    userIsScrolling = true;
    if (userScrollTimer) clearTimeout(userScrollTimer);
    userScrollTimer = setTimeout(() => {
      userIsScrolling = false;
      scrollToCurrent();
    }, 5000);
  }

  async function handleLineClick(line: LyricLine) {
    await seek(line.time);
    if (userScrollTimer) {
      clearTimeout(userScrollTimer);
      userScrollTimer = null;
    }
    userIsScrolling = false;
    requestAnimationFrame(() => scrollToCurrent());
  }

  onMount(() => {
    lastActiveIndex = -1;
    unsubscribePlayerState = playerState.subscribe(() => {
      updateActiveIndex();
    });
  });

  onDestroy(() => {
    if (userScrollTimer) clearTimeout(userScrollTimer);
    if (unsubscribePlayerState) {
      unsubscribePlayerState();
      unsubscribePlayerState = null;
    }
  });
</script>

<div
  bind:this={container}
  class="lyrics-container scrollbar-hidden"
  onwheel={handleUserScroll}
>
  {#if lyrics.length === 0}
    <div class="flex items-center justify-center h-full">
      <p class="text-text-muted text-sm">暂无歌词</p>
    </div>
  {:else}
    <div class="lyrics-center-spacer"></div>
    <div bind:this={listEl} class="space-y-3 px-4">
      {#each lyrics as line, index}
        <div
          class={`lyric-line text-center transition-all duration-300 cursor-pointer
            ${index === lastActiveIndex
              ? 'lyric-active'
              : index < lastActiveIndex
                ? 'text-text-secondary text-base opacity-60'
                : 'text-text-muted text-base opacity-40 hover:opacity-70'
            }`}
          role="button"
          tabindex={index === lastActiveIndex ? 0 : -1}
          onmousedown={() => handleLineClick(line)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleLineClick(line); }}
        >
          {line.text}
        </div>
      {/each}
    </div>
    <div class="lyrics-center-spacer"></div>
  {/if}
</div>

<style>
  .lyrics-container {
    height: 100%;
    overflow-y: auto;
    scroll-behavior: auto;
    mask-image: linear-gradient(
      to bottom,
      transparent 0%,
      black 10%,
      black 90%,
      transparent 100%
    );
    -webkit-mask-image: linear-gradient(
      to bottom,
      transparent 0%,
      black 10%,
      black 90%,
      transparent 100%
    );
  }

  .lyrics-center-spacer {
    height: 40%;
    flex-shrink: 0;
    pointer-events: none;
  }

  .lyric-line {
    transform-origin: center center;
    line-height: var(--lyrics-line-height, 2.2);
    font-size: var(--lyrics-font-size, 16px);
    user-select: none;
    padding: 0.25rem 1rem;
    display: block;
    width: fit-content;
    margin: 0 auto;
  }

  .lyric-active {
    color: var(--color-accent);
    font-size: calc(var(--lyrics-font-size, 16px) * 1.25);
    font-weight: 600;
    transform: scale(1.03);
    padding: 0.35rem 1.25rem;
  }

  .lyric-line:hover {
    opacity: 1 !important;
  }
</style>
