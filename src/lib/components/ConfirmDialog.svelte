<script lang="ts">
  import { AlertTriangle } from 'lucide-svelte';
  import Modal from './Modal.svelte';

  interface Props {
    open: boolean;
    title?: string;
    message: string;
    onConfirm: () => void;
    onCancel: () => void;
    loading?: boolean;
  }

  let {
    open,
    title = 'Konfirmasi',
    message,
    onConfirm,
    onCancel,
    loading = false,
  }: Props = $props();
</script>

<Modal {open} {title} onClose={onCancel} maxWidth="max-w-sm">
  {#snippet children()}
    <div class="flex items-start gap-3">
      <div class="shrink-0 w-10 h-10 rounded-full bg-red-100 flex items-center justify-center">
        <AlertTriangle size={20} class="text-red-600" />
      </div>
      <p class="text-sm text-gray-600 leading-relaxed">{message}</p>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn-secondary" onclick={onCancel} disabled={loading}>Batal</button>
    <button class="btn-danger" onclick={onConfirm} disabled={loading}>
      {loading ? 'Menghapus...' : 'Hapus'}
    </button>
  {/snippet}
</Modal>
