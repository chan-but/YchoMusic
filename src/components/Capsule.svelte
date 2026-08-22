<script lang="ts">
  export let title: string;
  export let description: string;
  export let icon: string;
  export let href: string | undefined = undefined;
  export let onclick: (() => void) | undefined = undefined;
</script>

<button
  type="button"
  class="glass p-6 rounded-2xl cursor-pointer transition-all duration-300 hover:scale-[1.02] hover:shadow-lg hover:shadow-accent/20 hover:border-accent/50 group block text-left w-full"
  style="transform-style: preserve-3d;"
  {onclick}
>
  <div class="flex items-start gap-4">
    <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-accent/50 to-text-secondary/50 flex items-center justify-center flex-shrink-0 transform group-hover:rotate-3 transition-transform">
      <span class="text-xl">{icon}</span>
    </div>
    <div>
      <h3 class="text-text-primary font-medium mb-1">{title}</h3>
      <p class="text-text-secondary text-sm">{description}</p>
    </div>
  </div>
  <div class="mt-4 flex items-center gap-2">
    <div class="flex-1 h-0.5 bg-bg-hover rounded-full overflow-hidden">
      <div class="h-full bg-accent/50 w-0 group-hover:w-full transition-all duration-500 rounded-full"></div>
    </div>
    <span class="text-text-muted text-xs opacity-0 group-hover:opacity-100 transition-opacity">→</span>
  </div>
</button>
