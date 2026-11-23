<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Pencil, Trash2, Package } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, formatCurrency, formatNumber } from '$lib/stores/app.svelte';
  import type { Produk, ProdukInput } from '$lib/types';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import * as Table from '$lib/components/ui/table/index.js';

  let list = $state<Produk[]>([]);
  let loading = $state(true);
  let search = $state('');
  let showModal = $state(false);
  let showConfirm = $state(false);
  let editingId = $state<number | null>(null);
  let deletingId = $state<number | null>(null);
  let saving = $state(false);
  let deleting = $state(false);

  let form = $state<ProdukInput>({ kode: '', nama: '', satuan: 'pcs', harga_beli: 0, harga_jual: 0, stok: 0 });

  const filtered = $derived(
    list.filter(p => {
      const q = search.toLowerCase();
      return !q || p.kode.toLowerCase().includes(q) || p.nama.toLowerCase().includes(q);
    })
  );

  async function load() {
    loading = true;
    try { list = await callCommand<Produk[]>('get_produk'); }
    finally { loading = false; }
  }

  function openCreate() {
    editingId = null;
    form = { kode: '', nama: '', satuan: 'pcs', harga_beli: 0, harga_jual: 0, stok: 0 };
    showModal = true;
  }

  function openEdit(p: Produk) {
    editingId = p.id;
    form = { kode: p.kode, nama: p.nama, satuan: p.satuan, harga_beli: p.harga_beli, harga_jual: p.harga_jual };
    showModal = true;
  }

  async function save() {
    if (!form.kode || !form.nama) return;
    saving = true;
    try {
      if (editingId) {
        await callCommand('update_produk', { id: editingId, input: form });
        showNotification('success', 'Produk berhasil diupdate');
      } else {
        await callCommand('create_produk', { input: form });
        showNotification('success', 'Produk berhasil ditambahkan');
      }
      showModal = false;
      await load();
    } catch (e) {
      showNotification('error', String(e));
    } finally { saving = false; }
  }

  async function doDelete() {
    if (!deletingId) return;
    deleting = true;
    try {
      await callCommand('delete_produk', { id: deletingId });
      showNotification('success', 'Produk berhasil dihapus');
      showConfirm = false;
      await load();
    } catch (e) {
      showNotification('error', String(e));
    } finally { deleting = false; }
  }

  onMount(load);
</script>

<div class="p-6">
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-xl font-semibold text-foreground">Produk & Persediaan</h1>
      <p class="text-sm text-muted-foreground mt-0.5">{list.length} produk terdaftar</p>
    </div>
    <Button onclick={openCreate}><Plus size={16} /> Tambah Produk</Button>
  </div>

  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
      <Input class="pl-9" placeholder="Cari produk..." bind:value={search} />
    </div>
  </div>

  <div class="overflow-x-auto rounded-xl border border-border">
    <Table.Root>
      <Table.Header>
        <Table.Row>
          <Table.Head>Kode</Table.Head>
          <Table.Head>Nama Produk</Table.Head>
          <Table.Head>Satuan</Table.Head>
          <Table.Head class="text-right">Harga Beli</Table.Head>
          <Table.Head class="text-right">Harga Jual</Table.Head>
          <Table.Head class="text-right">Stok</Table.Head>
          <Table.Head class="text-center">Aksi</Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#if loading}
          <Table.Row><Table.Cell colspan={7} class="text-center py-12 text-muted-foreground">Memuat data...</Table.Cell></Table.Row>
        {:else if filtered.length === 0}
          <Table.Row><Table.Cell colspan={7} class="text-center py-12 text-muted-foreground">Tidak ada data</Table.Cell></Table.Row>
        {:else}
          {#each filtered as p}
            <Table.Row>
              <Table.Cell class="font-mono text-xs font-medium text-muted-foreground">{p.kode}</Table.Cell>
              <Table.Cell>
                <div class="flex items-center gap-2">
                  <div class="w-7 h-7 rounded-lg bg-teal-50 flex items-center justify-center shrink-0">
                    <Package size={13} class="text-teal-600" />
                  </div>
                  <span class="font-medium text-foreground">{p.nama}</span>
                </div>
              </Table.Cell>
              <Table.Cell><Badge variant="secondary">{p.satuan}</Badge></Table.Cell>
              <Table.Cell class="text-right text-muted-foreground">{formatCurrency(p.harga_beli)}</Table.Cell>
              <Table.Cell class="text-right font-medium text-foreground">{formatCurrency(p.harga_jual)}</Table.Cell>
              <Table.Cell class="text-right font-medium {p.stok <= 0 ? 'text-red-500' : p.stok < 10 ? 'text-yellow-600' : 'text-green-600'}">
                {formatNumber(p.stok)}
              </Table.Cell>
              <Table.Cell>
                <div class="flex items-center justify-center gap-1">
                  <Button variant="ghost" size="icon-sm" onclick={() => openEdit(p)}><Pencil size={14} /></Button>
                  <Button variant="ghost" size="icon-sm" class="text-destructive hover:text-destructive hover:bg-destructive/10" onclick={() => { deletingId = p.id; showConfirm = true; }}><Trash2 size={14} /></Button>
                </div>
              </Table.Cell>
            </Table.Row>
          {/each}
        {/if}
      </Table.Body>
    </Table.Root>
  </div>
</div>

<Modal open={showModal} title={editingId ? 'Edit Produk' : 'Tambah Produk'} onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="prd-kode">Kode *</Label>
          <Input id="prd-kode" placeholder="PRD-001" bind:value={form.kode} />
        </div>
        <div>
          <Label for="prd-satuan">Satuan *</Label>
          <Input id="prd-satuan" placeholder="pcs, kg, box..." bind:value={form.satuan} />
        </div>
      </div>
      <div>
        <Label for="prd-nama">Nama Produk *</Label>
        <Input id="prd-nama" placeholder="Nama produk" bind:value={form.nama} />
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="prd-harga-beli">Harga Beli</Label>
          <Input id="prd-harga-beli" type="number" placeholder="0" bind:value={form.harga_beli} />
        </div>
        <div>
          <Label for="prd-harga-jual">Harga Jual</Label>
          <Input id="prd-harga-jual" type="number" placeholder="0" bind:value={form.harga_jual} />
        </div>
      </div>
      {#if !editingId}
        <div>
          <Label for="prd-stok">Stok Awal</Label>
          <Input id="prd-stok" type="number" placeholder="0" bind:value={form.stok} />
        </div>
      {/if}
    </div>
  {/snippet}
  {#snippet footer()}
    <Button variant="outline" onclick={() => showModal = false}>Batal</Button>
    <Button onclick={save} disabled={saving}>{saving ? 'Menyimpan...' : 'Simpan'}</Button>
  {/snippet}
</Modal>

<ConfirmDialog open={showConfirm} message="Hapus produk ini?" onConfirm={doDelete} onCancel={() => showConfirm = false} loading={deleting} />
