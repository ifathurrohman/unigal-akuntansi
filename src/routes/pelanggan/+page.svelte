<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Pencil, Trash2, Phone, Mail } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, formatCurrency } from '$lib/stores/app.svelte';
  import type { Pelanggan, PelangganInput } from '$lib/types';

  let list = $state<Pelanggan[]>([]);
  let loading = $state(true);
  let search = $state('');
  let showModal = $state(false);
  let showConfirm = $state(false);
  let editingId = $state<number | null>(null);
  let deletingId = $state<number | null>(null);
  let saving = $state(false);
  let deleting = $state(false);

  let form = $state<PelangganInput>({ kode: '', nama: '', telepon: null, email: null, alamat: null });

  const filtered = $derived(
    list.filter(p => {
      const q = search.toLowerCase();
      return !q || p.kode.toLowerCase().includes(q) || p.nama.toLowerCase().includes(q) || (p.telepon || '').includes(q);
    })
  );

  async function load() {
    loading = true;
    try { list = await callCommand<Pelanggan[]>('get_pelanggan'); }
    finally { loading = false; }
  }

  function openCreate() {
    editingId = null;
    form = { kode: '', nama: '', telepon: null, email: null, alamat: null };
    showModal = true;
  }

  function openEdit(p: Pelanggan) {
    editingId = p.id;
    form = { kode: p.kode, nama: p.nama, telepon: p.telepon, email: p.email, alamat: p.alamat };
    showModal = true;
  }

  async function save() {
    if (!form.kode || !form.nama) return;
    saving = true;
    try {
      const input = { ...form, telepon: form.telepon || null, email: form.email || null, alamat: form.alamat || null };
      if (editingId) {
        await callCommand('update_pelanggan', { id: editingId, input });
        showNotification('success', 'Pelanggan berhasil diupdate');
      } else {
        await callCommand('create_pelanggan', { input });
        showNotification('success', 'Pelanggan berhasil ditambahkan');
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
      await callCommand('delete_pelanggan', { id: deletingId });
      showNotification('success', 'Pelanggan berhasil dihapus');
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
      <h1 class="page-title">Pelanggan</h1>
      <p class="page-subtitle">{list.length} pelanggan terdaftar</p>
    </div>
    <button class="btn-primary" onclick={openCreate}><Plus size={16} /> Tambah Pelanggan</button>
  </div>

  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
      <input class="input pl-9" placeholder="Cari pelanggan..." bind:value={search} />
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
          <th class="text-right">Piutang</th>
          <th class="text-center">Aksi</th>
        </tr>
      </thead>
      <tbody>
        {#if loading}
          <tr><td colspan="6" class="text-center py-12 text-gray-400">Memuat data...</td></tr>
        {:else if filtered.length === 0}
          <tr><td colspan="6" class="text-center py-12 text-gray-400">Tidak ada data</td></tr>
        {:else}
          {#each filtered as p}
            <tr>
              <td class="font-mono text-xs font-medium text-gray-500">{p.kode}</td>
              <td class="font-medium text-gray-900">{p.nama}</td>
              <td>
                {#if p.telepon}
                  <div class="flex items-center gap-1 text-gray-500 text-xs">
                    <Phone size={12} />{p.telepon}
                  </div>
                {:else}
                  <span class="text-gray-300">-</span>
                {/if}
              </td>
              <td>
                {#if p.email}
                  <div class="flex items-center gap-1 text-gray-500 text-xs">
                    <Mail size={12} />{p.email}
                  </div>
                {:else}
                  <span class="text-gray-300">-</span>
                {/if}
              </td>
              <td class="text-right font-medium {p.piutang > 0 ? 'text-orange-600' : 'text-gray-500'}">
                {formatCurrency(p.piutang)}
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

<Modal open={showModal} title={editingId ? 'Edit Pelanggan' : 'Tambah Pelanggan'} onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label" for="plg-kode">Kode *</label>
          <input id="plg-kode" class="input" placeholder="PLG-001" bind:value={form.kode} />
        </div>
        <div>
          <label class="label" for="plg-nama">Nama *</label>
          <input id="plg-nama" class="input" placeholder="Nama pelanggan" bind:value={form.nama} />
        </div>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label" for="plg-telepon">Telepon</label>
          <input id="plg-telepon" class="input" placeholder="08xx" bind:value={form.telepon} />
        </div>
        <div>
          <label class="label" for="plg-email">Email</label>
          <input id="plg-email" class="input" type="email" placeholder="email@domain.com" bind:value={form.email} />
        </div>
      </div>
      <div>
        <label class="label" for="plg-alamat">Alamat</label>
        <textarea id="plg-alamat" class="input" rows="2" placeholder="Alamat lengkap..." bind:value={form.alamat}></textarea>
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn-secondary" onclick={() => showModal = false}>Batal</button>
    <button class="btn-primary" onclick={save} disabled={saving}>{saving ? 'Menyimpan...' : 'Simpan'}</button>
  {/snippet}
</Modal>

<ConfirmDialog open={showConfirm} message="Hapus pelanggan ini?" onConfirm={doDelete} onCancel={() => showConfirm = false} loading={deleting} />
