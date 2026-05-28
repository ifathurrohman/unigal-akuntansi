<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Pencil, Trash2, Phone, Mail } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, formatCurrency } from '$lib/stores/app.svelte';
  import type { Supplier, SupplierInput } from '$lib/types';

  let list = $state<Supplier[]>([]);
  let loading = $state(true);
  let search = $state('');
  let showModal = $state(false);
  let showConfirm = $state(false);
  let editingId = $state<number | null>(null);
  let deletingId = $state<number | null>(null);
  let saving = $state(false);
  let deleting = $state(false);

  let form = $state<SupplierInput>({ kode: '', nama: '', telepon: null, email: null, alamat: null });

  const filtered = $derived(
    list.filter(s => {
      const q = search.toLowerCase();
      return !q || s.kode.toLowerCase().includes(q) || s.nama.toLowerCase().includes(q);
    })
  );

  async function load() {
    loading = true;
    try { list = await callCommand<Supplier[]>('get_supplier'); }
    finally { loading = false; }
  }

  function openCreate() {
    editingId = null;
    form = { kode: '', nama: '', telepon: null, email: null, alamat: null };
    showModal = true;
  }

  function openEdit(s: Supplier) {
    editingId = s.id;
    form = { kode: s.kode, nama: s.nama, telepon: s.telepon, email: s.email, alamat: s.alamat };
    showModal = true;
  }

  async function save() {
    if (!form.kode || !form.nama) return;
    saving = true;
    try {
      const input = { ...form, telepon: form.telepon || null, email: form.email || null, alamat: form.alamat || null };
      if (editingId) {
        await callCommand('update_supplier', { id: editingId, input });
        showNotification('success', 'Supplier berhasil diupdate');
      } else {
        await callCommand('create_supplier', { input });
        showNotification('success', 'Supplier berhasil ditambahkan');
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
      await callCommand('delete_supplier', { id: deletingId });
      showNotification('success', 'Supplier berhasil dihapus');
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
      <h1 class="page-title">Supplier</h1>
      <p class="page-subtitle">{list.length} supplier terdaftar</p>
    </div>
    <button class="btn-primary" onclick={openCreate}><Plus size={16} /> Tambah Supplier</button>
  </div>

  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
      <input class="input pl-9" placeholder="Cari supplier..." bind:value={search} />
    </div>
  </div>

  <div class="table-wrapper">
    <table class="data-table">
      <thead>
        <tr>
          <th>Kode</th>
          <th>Nama</th>
          <th>Telepon</th>
          <th>Email</th>
          <th class="text-right">Hutang</th>
          <th class="text-center">Aksi</th>
        </tr>
      </thead>
      <tbody>
        {#if loading}
          <tr><td colspan="6" class="text-center py-12 text-gray-400">Memuat data...</td></tr>
        {:else if filtered.length === 0}
          <tr><td colspan="6" class="text-center py-12 text-gray-400">Tidak ada data</td></tr>
        {:else}
          {#each filtered as s}
            <tr>
              <td class="font-mono text-xs font-medium text-gray-500">{s.kode}</td>
              <td class="font-medium text-gray-900">{s.nama}</td>
              <td>
                {#if s.telepon}
                  <div class="flex items-center gap-1 text-gray-500 text-xs"><Phone size={12} />{s.telepon}</div>
                {:else}<span class="text-gray-300">-</span>{/if}
              </td>
              <td>
                {#if s.email}
                  <div class="flex items-center gap-1 text-gray-500 text-xs"><Mail size={12} />{s.email}</div>
                {:else}<span class="text-gray-300">-</span>{/if}
              </td>
              <td class="text-right font-medium {s.hutang > 0 ? 'text-red-600' : 'text-gray-500'}">{formatCurrency(s.hutang)}</td>
              <td>
                <div class="flex items-center justify-center gap-1">
                  <button class="btn-ghost btn-sm p-1.5" onclick={() => openEdit(s)}><Pencil size={14} /></button>
                  <button class="btn-ghost btn-sm p-1.5 text-red-500 hover:text-red-700 hover:bg-red-50" onclick={() => { deletingId = s.id; showConfirm = true; }}><Trash2 size={14} /></button>
                </div>
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>

<Modal open={showModal} title={editingId ? 'Edit Supplier' : 'Tambah Supplier'} onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label" for="sup-kode">Kode *</label>
          <input id="sup-kode" class="input" placeholder="SUP-001" bind:value={form.kode} />
        </div>
        <div>
          <label class="label" for="sup-nama">Nama *</label>
          <input id="sup-nama" class="input" placeholder="Nama supplier" bind:value={form.nama} />
        </div>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label" for="sup-telepon">Telepon</label>
          <input id="sup-telepon" class="input" placeholder="08xx" bind:value={form.telepon} />
        </div>
        <div>
          <label class="label" for="sup-email">Email</label>
          <input id="sup-email" class="input" type="email" placeholder="email@domain.com" bind:value={form.email} />
        </div>
      </div>
      <div>
        <label class="label" for="sup-alamat">Alamat</label>
        <textarea id="sup-alamat" class="input" rows="2" placeholder="Alamat lengkap..." bind:value={form.alamat}></textarea>
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn-secondary" onclick={() => showModal = false}>Batal</button>
    <button class="btn-primary" onclick={save} disabled={saving}>{saving ? 'Menyimpan...' : 'Simpan'}</button>
  {/snippet}
</Modal>

<ConfirmDialog open={showConfirm} message="Hapus supplier ini?" onConfirm={doDelete} onCancel={() => showConfirm = false} loading={deleting} />
