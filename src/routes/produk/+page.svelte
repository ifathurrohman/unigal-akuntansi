<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Pencil, Trash2, Package } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, formatCurrency, formatNumber } from '$lib/stores/app.svelte';
  import type { Produk, ProdukInput } from '$lib/types';

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
  <div class="page-header">
    <div>
      <h1 class="page-title">Produk & Persediaan</h1>
      <p class="page-subtitle">{list.length} produk terdaftar</p>
    </div>
    <button class="btn-primary" onclick={openCreate}><Plus size={16} /> Tambah Produk</button>
  </div>

  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
      <input class="input pl-9" placeholder="Cari produk..." bind:value={search} />
    </div>
  </div>

  <div class="table-wrapper">
    <table class="data-table">
      <thead>
        <tr>
          <th>Kode</th>
          <th>Nama Produk</th>
          <th>Satuan</th>
          <th class="text-right">Harga Beli</th>
          <th class="text-right">Harga Jual</th>
          <th class="text-right">Stok</th>
          <th class="text-center">Aksi</th>
        </tr>
      </thead>
      <tbody>
        {#if loading}
          <tr><td colspan="7" class="text-center py-12 text-gray-400">Memuat data...</td></tr>
        {:else if filtered.length === 0}
          <tr><td colspan="7" class="text-center py-12 text-gray-400">Tidak ada data</td></tr>
        {:else}
          {#each filtered as p}
            <tr>
              <td class="font-mono text-xs font-medium text-gray-500">{p.kode}</td>
              <td>
                <div class="flex items-center gap-2">
                  <div class="w-7 h-7 rounded-lg bg-teal-50 flex items-center justify-center shrink-0">
                    <Package size={13} class="text-teal-600" />
                  </div>
                  <span class="font-medium text-gray-900">{p.nama}</span>
                </div>
              </td>
              <td><span class="badge-gray">{p.satuan}</span></td>
              <td class="text-right text-gray-600">{formatCurrency(p.harga_beli)}</td>
              <td class="text-right font-medium text-gray-900">{formatCurrency(p.harga_jual)}</td>
              <td class="text-right font-medium {p.stok <= 0 ? 'text-red-500' : p.stok < 10 ? 'text-yellow-600' : 'text-green-600'}">
                {formatNumber(p.stok)}
              </td>
              <td>
                <div class="flex items-center justify-center gap-1">
                  <button class="btn-ghost btn-sm p-1.5" onclick={() => openEdit(p)}><Pencil size={14} /></button>
                  <button class="btn-ghost btn-sm p-1.5 text-red-500 hover:text-red-700 hover:bg-red-50" onclick={() => { deletingId = p.id; showConfirm = true; }}><Trash2 size={14} /></button>
                </div>
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>

<Modal open={showModal} title={editingId ? 'Edit Produk' : 'Tambah Produk'} onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label" for="prd-kode">Kode *</label>
          <input id="prd-kode" class="input" placeholder="PRD-001" bind:value={form.kode} />
        </div>
        <div>
          <label class="label" for="prd-satuan">Satuan *</label>
          <input id="prd-satuan" class="input" placeholder="pcs, kg, box..." bind:value={form.satuan} />
        </div>
      </div>
      <div>
        <label class="label" for="prd-nama">Nama Produk *</label>
        <input id="prd-nama" class="input" placeholder="Nama produk" bind:value={form.nama} />
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label" for="prd-harga-beli">Harga Beli</label>
          <input id="prd-harga-beli" class="input" type="number" placeholder="0" bind:value={form.harga_beli} />
        </div>
        <div>
          <label class="label" for="prd-harga-jual">Harga Jual</label>
          <input id="prd-harga-jual" class="input" type="number" placeholder="0" bind:value={form.harga_jual} />
        </div>
      </div>
      {#if !editingId}
        <div>
          <label class="label" for="prd-stok">Stok Awal</label>
          <input id="prd-stok" class="input" type="number" placeholder="0" bind:value={form.stok} />
        </div>
      {/if}
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn-secondary" onclick={() => showModal = false}>Batal</button>
    <button class="btn-primary" onclick={save} disabled={saving}>{saving ? 'Menyimpan...' : 'Simpan'}</button>
  {/snippet}
</Modal>

<ConfirmDialog open={showConfirm} message="Hapus produk ini?" onConfirm={doDelete} onCancel={() => showConfirm = false} loading={deleting} />
