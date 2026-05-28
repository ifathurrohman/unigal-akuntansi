<script lang="ts">
  import { X } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { cubicOut, cubicIn } from 'svelte/easing';

  interface Props {
    open: boolean;
    title: string;
    maxWidth?: string;
    onClose: () => void;
    children: import('svelte').Snippet;
    footer?: import('svelte').Snippet;
  }

  let { open, title, maxWidth = 'max-w-lg', onClose, children, footer }: Props = $props();

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function slideUp(node: Element, { duration = 280 }: { duration?: number } = {}) {
    return {
      duration,
      css: (t: number) => {
        const eased = cubicOut(t);
        return `
          opacity: ${eased};
          transform: translateY(${(1 - eased) * 24}px) scale(${0.96 + eased * 0.04});
        `;
      },
    };
  }

  function slideDown(node: Element, { duration = 200 }: { duration?: number } = {}) {
    return {
      duration,
      css: (t: number) => {
        const eased = cubicIn(t);
        return `
          opacity: ${t};
          transform: translateY(${(1 - t) * 12}px) scale(${0.98 + t * 0.02});
        `;
      },
    };
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="modal-backdrop"
    onclick={handleBackdrop}
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 180 }}
  >
    <div
      class="modal-box {maxWidth} w-full"
      in:slideUp={{ duration: 280 }}
      out:slideDown={{ duration: 200 }}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-100">
        <h2 class="text-base font-semibold text-gray-900">{title}</h2>
        <button onclick={onClose} class="btn-ghost p-1.5 rounded-lg text-gray-400 hover:text-gray-600">
          <X size={18} />
        </button>
      </div>

      <!-- Body -->
      <div class="px-6 py-4">
        {@render children()}
      </div>

      <!-- Footer -->
      {#if footer}
        <div class="px-6 py-4 border-t border-gray-100 flex justify-end gap-2">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}
