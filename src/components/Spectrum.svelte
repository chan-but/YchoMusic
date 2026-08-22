<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { playerState, getCurrentPlayerState } from '@/stores/player';

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let animationId: number;
  const bars = 64;

  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d')!;
      resize();
      window.addEventListener('resize', resize);
      animate();
    }
  });

  onDestroy(() => {
    if (animationId) cancelAnimationFrame(animationId);
    window.removeEventListener('resize', resize);
  });

  function resize() {
    if (canvas) {
      canvas.width = canvas.offsetWidth * window.devicePixelRatio;
      canvas.height = canvas.offsetHeight * window.devicePixelRatio;
      if (ctx) ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    }
  }

  function animate() {
    if (!ctx || !canvas) return;

    const width = canvas.offsetWidth;
    const height = canvas.offsetHeight;
    const barWidth = width / bars;

    ctx.clearRect(0, 0, width, height);

    const state = getCurrentPlayerState();
    if (state.state === 'Playing') {
      for (let i = 0; i < bars; i++) {
        const value = Math.random() * 0.8 + 0.1;
        const barHeight = value * height * 0.8;
        const x = i * barWidth;
        const y = height - barHeight;

        const gradient = ctx.createLinearGradient(0, height, 0, y);
        gradient.addColorStop(0, 'rgba(0, 212, 255, 0.1)');
        gradient.addColorStop(0.5, 'rgba(0, 212, 255, 0.5)');
        gradient.addColorStop(1, 'rgba(0, 212, 255, 0.9)');

        ctx.fillStyle = gradient;
        ctx.fillRect(x + 1, y, barWidth - 2, barHeight);
      }
    }

    animationId = requestAnimationFrame(animate);
  }
</script>

<canvas
  bind:this={canvas}
  class="w-full h-full"
  style="display: block;"
></canvas>
