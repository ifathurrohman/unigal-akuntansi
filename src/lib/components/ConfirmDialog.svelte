<script lang="ts">
  import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';

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
    title = 'Konfirmasi Hapus',
    message,
    onConfirm,
    onCancel,
    loading = false,
  }: Props = $props();
</script>

<AlertDialog.Root {open} onOpenChange={(o) => { if (!o) onCancel(); }}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>{title}</AlertDialog.Title>
      <AlertDialog.Description>{message}</AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={onCancel} disabled={loading}>Batal</AlertDialog.Cancel>
      <AlertDialog.Action
        variant="destructive"
        onclick={onConfirm}
        disabled={loading}
      >
        {loading ? 'Menghapus...' : 'Hapus'}
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
