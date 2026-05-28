<script lang="ts">
  import { onMount } from 'svelte';
  import { BookOpen } from 'lucide-svelte';

  interface Props {
    onDone: () => void;
  }

  let { onDone }: Props = $props();

  let phase = $state<'enter' | 'hold' | 'exit'>('enter');

  onMount(() => {
    // enter → hold setelah animasi masuk selesai
    const t1 = setTimeout(() => { phase = 'hold'; }, 600);
    // hold → exit
    const t2 = setTimeout(() => { phase = 'exit'; }, 2200);
    // exit selesai → panggil onDone
    const t3 = setTimeout(() => { onDone(); }, 2900);
    return () => { clearTimeout(t1); clearTimeout(t2); clearTimeout(t3); };
  });
</script>

<div
  class="splash-wrap"
  class:exit={phase === 'exit'}
  aria-hidden="true"
>
  <!-- Background gradient -->
  <div class="absolute inset-0 bg-gradient-to-br from-indigo-700 via-indigo-600 to-violet-700"></div>

  <!-- Decorative circles -->
  <div class="absolute -top-24 -left-24 w-96 h-96 bg-white/5 rounded-full blur-3xl"></div>
  <div class="absolute -bottom-32 -right-32 w-[500px] h-[500px] bg-indigo-900/40 rounded-full blur-3xl"></div>

  <!-- Content -->
  <div class="relative z-10 flex flex-col items-center gap-6"
    class:content-enter={phase !== 'exit'}
    class:content-exit={phase === 'exit'}
  >
    <!-- Logo icon -->
    <div class="logo-ring">
      <div class="w-20 h-20 rounded-2xl bg-white/20 backdrop-blur-sm flex items-center justify-center shadow-2xl ring-1 ring-white/30">
        <BookOpen size={40} class="text-white" strokeWidth={1.5} />
      </div>
    </div>

    <!-- App name -->
    <div class="text-center">
      <h1 class="text-3xl font-bold text-white tracking-tight">Unigal Akuntansi</h1>
      <p class="text-indigo-200 text-sm mt-1.5 font-medium">Solusi Pembukuan Modern</p>
    </div>

    <!-- Loading bar -->
    <div class="w-48 h-1 bg-white/20 rounded-full overflow-hidden mt-2">
      <div class="loading-bar h-full bg-white rounded-full"></div>
    </div>

    <p class="text-indigo-300 text-xs tracking-widest uppercase">Memuat aplikasi...</p>
  </div>

  <!-- Version -->
  <div class="absolute bottom-6 text-indigo-300/60 text-xs">v1.0.0</div>
</div>

<style>
  .splash-wrap {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    transition: opacity 0.6s ease, transform 0.6s ease;
  }

  .splash-wrap.exit {
    opacity: 0;
    transform: scale(1.04);
    pointer-events: none;
  }

  /* Logo pulse ring */
  .logo-ring {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .logo-ring::before,
  .logo-ring::after {
    content: '';
    position: absolute;
    border-radius: 50%;
    border: 2px solid rgba(255, 255, 255, 0.2);
    animation: ring-pulse 2s ease-out infinite;
  }
  .logo-ring::before {
    width: 110px;
    height: 110px;
    animation-delay: 0s;
  }
  .logo-ring::after {
    width: 150px;
    height: 150px;
    animation-delay: 0.5s;
  }

  @keyframes ring-pulse {
    0%   { opacity: 0.6; transform: scale(0.95); }
    100% { opacity: 0;   transform: scale(1.3); }
  }

  /* Content enter animation */
  .content-enter {
    animation: content-in 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
  .content-exit {
    animation: content-out 0.6s ease forwards;
  }

  @keyframes content-in {
    from { opacity: 0; transform: translateY(16px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  @keyframes content-out {
    from { opacity: 1; transform: translateY(0); }
    to   { opacity: 0; transform: translateY(-8px); }
  }

  /* Loading bar */
  .loading-bar {
    animation: loading 1.8s ease-in-out forwards;
  }
  @keyframes loading {
    0%   { width: 0%; }
    30%  { width: 40%; }
    70%  { width: 75%; }
    100% { width: 100%; }
  }
</style>
