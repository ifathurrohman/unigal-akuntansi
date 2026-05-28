<script lang="ts">
  import { CheckCircle, XCircle } from 'lucide-svelte';
  import { notif } from '$lib/stores/app.svelte';
</script>

{#if notif.visible}
  <div class="fixed top-4 right-4 z-50 animate-slide-in">
    <div class="flex items-center gap-3 px-4 py-3 rounded-xl shadow-lg border text-sm font-medium
      {notif.type === 'success' ? 'bg-green-50 border-green-200 text-green-800' : 'bg-red-50 border-red-200 text-red-800'}">
      {#if notif.type === 'success'}
        <CheckCircle size={18} class="text-green-600 shrink-0" />
      {:else}
        <XCircle size={18} class="text-red-600 shrink-0" />
      {/if}
      <span>{notif.message}</span>
    </div>
  </div>
{/if}

<style>
  @keyframes slide-in {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
  .animate-slide-in {
    animation: slide-in 0.2s ease-out;
  }
</style>
