<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog/index.js';

  interface Props {
    open: boolean;
    title: string;
    maxWidth?: string;
    onClose: () => void;
    children: import('svelte').Snippet;
    footer?: import('svelte').Snippet;
  }

  let { open, title, maxWidth = 'sm:max-w-lg', onClose, children, footer }: Props = $props();
</script>

<Dialog.Root {open} onOpenChange={(o) => { if (!o) onClose(); }}>
  <Dialog.Content class={maxWidth}>
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
    </Dialog.Header>
    <div class="py-1">
      {@render children()}
    </div>
    {#if footer}
      <Dialog.Footer>
        {@render footer()}
      </Dialog.Footer>
    {/if}
  </Dialog.Content>
</Dialog.Root>
