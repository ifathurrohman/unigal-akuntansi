<script lang="ts">
  import { Download, RefreshCw, X, CheckCircle, AlertCircle } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Progress } from '$lib/components/ui/progress/index.js';

  interface Props {
    version: string;
    notes: string | null;
    onConfirm: () => void;
    onDismiss: () => void;
    state: 'idle' | 'downloading' | 'done' | 'error';
    progress: number;
    errorMsg: string;
  }

  let { version, notes, onConfirm, onDismiss, state, progress, errorMsg }: Props = $props();

  function slideUp(node: Element, { duration = 300 }: { duration?: number } = {}) {
    return {
      duration,
      css: (t: number) => {
        const e = cubicOut(t);
        return `opacity:${e}; transform:translateY(${(1-e)*28}px) scale(${0.95+e*0.05});`;
      },
    };
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center p-4"
  onclick={(e) => { if (state === 'idle' && e.target === e.currentTarget) onDismiss(); }}
  in:fade={{ duration: 200 }}
  out:fade={{ duration: 180 }}
>
  <div
    class="bg-popover text-popover-foreground rounded-2xl shadow-2xl w-full max-w-md overflow-hidden ring-1 ring-foreground/10"
    in:slideUp={{ duration: 300 }}
  >

    {#if state === 'done'}
      <div class="p-8 text-center">
        <div class="w-16 h-16 rounded-full bg-green-100 flex items-center justify-center mx-auto mb-4">
          <CheckCircle size={32} class="text-green-600" />
        </div>
        <h2 class="text-lg font-bold mb-1">Update Berhasil!</h2>
        <p class="text-sm text-muted-foreground mb-6">Aplikasi akan restart untuk menerapkan pembaruan.</p>
        <div class="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin mx-auto"></div>
      </div>

    {:else if state === 'error'}
      <div class="p-8 text-center">
        <div class="w-16 h-16 rounded-full bg-red-100 flex items-center justify-center mx-auto mb-4">
          <AlertCircle size={32} class="text-red-600" />
        </div>
        <h2 class="text-lg font-bold mb-1">Gagal Mengunduh</h2>
        <p class="text-xs text-muted-foreground mb-1 font-mono break-all">{errorMsg}</p>
        <p class="text-sm text-muted-foreground mt-3">Silakan coba lagi nanti.</p>
        <Button variant="outline" class="mt-5 w-full" onclick={onDismiss}>Tutup</Button>
      </div>

    {:else}
      <!-- Header gradient -->
      <div class="relative px-6 pt-6 pb-4 bg-gradient-to-br from-indigo-600 to-violet-600 text-white">
        <Button
          variant="ghost"
          size="icon-sm"
          class="absolute top-3 right-3 text-white/80 hover:text-white hover:bg-white/10"
          onclick={onDismiss}
          disabled={state === 'downloading'}
        >
          <X size={16} />
        </Button>

        <div class="flex items-center gap-3 mb-3">
          <div class="w-10 h-10 rounded-xl bg-white/20 flex items-center justify-center">
            <RefreshCw size={20} />
          </div>
          <div>
            <div class="text-xs text-indigo-200 font-medium uppercase tracking-wide">Pembaruan Tersedia</div>
            <div class="text-lg font-bold">Versi {version}</div>
          </div>
        </div>
      </div>

      <!-- Body -->
      <div class="px-6 py-5">
        {#if notes}
          <div class="mb-4">
            <p class="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">Catatan Rilis</p>
            <div class="text-sm bg-muted rounded-xl p-3 leading-relaxed max-h-32 overflow-y-auto whitespace-pre-wrap">
              {notes}
            </div>
          </div>
        {/if}

        {#if state === 'downloading'}
          <div class="mt-2">
            <div class="flex justify-between text-xs text-muted-foreground mb-1.5">
              <span class="flex items-center gap-1.5">
                <Download size={12} class="animate-bounce" /> Mengunduh pembaruan...
              </span>
              <span class="font-semibold text-primary">{Math.round(progress)}%</span>
            </div>
            <Progress value={progress} class="h-2" />
            <p class="text-xs text-muted-foreground mt-2 text-center">Mohon tunggu, jangan tutup aplikasi</p>
          </div>
        {:else}
          <p class="text-sm text-foreground/80 leading-relaxed">
            Update baru tersedia untuk <strong>Unigal Akuntansi</strong>.
            Apakah Anda ingin mengunduh dan menginstal sekarang?
          </p>
          <p class="text-xs text-muted-foreground mt-1.5">Aplikasi akan restart otomatis setelah selesai.</p>
        {/if}
      </div>

      <!-- Footer -->
      {#if state === 'idle'}
        <div class="px-6 pb-5 flex gap-2">
          <Button variant="outline" class="flex-1" onclick={onDismiss}>Nanti Saja</Button>
          <Button class="flex-1" onclick={onConfirm}>
            <Download size={15} /> Update Sekarang
          </Button>
        </div>
      {/if}
    {/if}
  </div>
</div>
