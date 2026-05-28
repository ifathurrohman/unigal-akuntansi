<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Pencil, Trash2, FileText } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, formatCurrency } from '$lib/stores/app.svelte';
  import type { Akun, AkunInput } from '$lib/types';

  let akun = $state<Akun[]>([]);
  let loading = $state(true);
  let search = $state('');
  let showModal = $state(false);
  let showConfirm = $state(false);
  let editingId = $state<number | null>(null);
  let deletingId = $state<number | null>(null);
  let saving = $state(false);
  let deleting = $state(false);
  let filterTipe = $state('');

  let form = $state<AkunInput>({
    kode: '',
    nama: '',
    tipe: 'Aset',
    sub_tipe: null,
    saldo_normal: 'Debit',
    saldo: 0,
  });

  const tipeOptions = ['Aset', 'Kewajiban', 'Modal', 'Pendapatan', 'Beban'];
  const saldoNormalByTipe: Record<string, string> = {
    Aset: 'Debit', Beban: 'Debit',
    Kewajiban: 'Kredit', Modal: 'Kredit', Pendapatan: 'Kredit',
  };

  const filtered = $derived(
    akun.filter(a => {
      const q = search.toLowerCase();
      const matchQ = !q || a.kode.toLowerCase().includes(q) || a.nama.toLowerCase().includes(q);
      const matchTipe = !filterTipe || a.tipe === filterTipe;
      return matchQ && matchTipe;
    })
  );

  async function load() {
    loading = true;
    try {
      akun = await callCommand<Akun[]>('get_akun');
    } finally {
      loading = false;
    }
  }

  function openCreate() {
    editingId = null;
    form = { kode: '', nama: '', tipe: 'Aset', sub_tipe: null, saldo_normal: 'Debit', saldo: 0 };
    showModal = true;
  }

  function openEdit(a: Akun) {
    editingId = a.id;
    form = { kode: a.kode, nama: a.nama, tipe: a.tipe, sub_tipe: a.sub_tipe, saldo_normal: a.saldo_normal };
    showModal = true;
  }

  function confirmDelete(id: number) {
    deletingId = id;
    showConfirm = true;
  }

  function onTipeChange() {
    form.saldo_normal = saldoNormalByTipe[form.tipe] || 'Debit';
  }

  async function save() {
    if (!form.kode || !form.nama) return;
    saving = true;
    try {
      if (editingId) {
        await callCommand('update_akun', { id: editingId, input: form });
        showNotification('success', 'Akun berhasil diupdate');
      } else {
        await callCommand('create_akun', { input: form });
        showNotification('success', 'Akun berhasil ditambahkan');
      }
      showModal = false;
      await load();
    } catch (e) {
      showNotification('error', String(e));
    } finally {
      saving = false;
    }
  }

  async function doDelete() {
    if (!deletingId) return;
    deleting = true;
    try {
      await callCommand('delete_akun', { id: deletingId });
      showNotification('success', 'Akun berhasil dihapus');
      showConfirm = false;
      await load();
    } catch (e) {
      showNotification('error', String(e));
    } finally {
      deleting = false;
    }
  }

  function getTipeBadge(tipe: string) {
    const map: Record<string, string> = {
      Aset: 'badge-blue', Kewajiban: 'badge-red', Modal: 'badge-green',
      Pendapatan: 'badge-yellow', Beban: 'badge-gray',
    };
    return map[tipe] || 'badge-gray';
  }

  onMount(load);
</script>

<div class="p-6">
  <div class="page-header">
    <div>
      <h1 class="page-title">Daftar Akun</h1>
      <p class="page-subtitle">Chart of Accounts (CoA)</p>
    </div>
    <button class="btn-primary" onclick={openCreate}>
      <Plus size={16} /> Tambah Akun
    </button>
  </div>

  <!-- Filters -->
  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
      <input class="input pl-9" placeholder="Cari kode atau nama akun..." bind:value={search} />
    </div>
    <select class="input w-40" bind:value={filterTipe}>
      <option value="">Semua Tipe</option>
      {#each tipeOptions as t}
        <option value={t}>{t}</option>
      {/each}
    </select>
  </div>

  <!-- Table -->
  <div class="table-wrapper">
    <table class="data-table">
      <thead>
        <tr>
          <th>Kode</th>
          <th>Nama Akun</th>
          <th>Tipe</th>
          <th>Saldo Normal</th>
          <th class="text-right">Saldo</th>
          <th class="text-center">Aksi</th>
        </tr>
      </thead>
      <tbody>
        {#if loading}
          <tr>
            <td colspan="6" class="text-center py-12 text-gray-400">Memuat data...</td>
          </tr>
        {:else if filtered.length === 0}
          <tr>
            <td colspan="6" class="text-center py-12 text-gray-400">Tidak ada data</td>
          </tr>
        {:else}
          {#each filtered as a}
            <tr>
              <td class="font-mono text-xs font-medium text-gray-600">{a.kode}</td>
              <td class="font-medium">{a.nama}</td>
              <td><span class={getTipeBadge(a.tipe)}>{a.tipe}</span></td>
              <td>
                <span class={a.saldo_normal === 'Debit' ? 'badge-blue' : 'badge-green'}>{a.saldo_normal}</span>
              </td>
              <td class="text-right font-medium {a.saldo < 0 ? 'text-red-600' : 'text-gray-900'}">
                {formatCurrency(a.saldo)}
              </td>
              <td>
                <div class="flex items-center justify-center gap-1">
                  <button class="btn-ghost btn-sm p-1.5" onclick={() => openEdit(a)} title="Edit">
                    <Pencil size={14} />
                  </button>
                  <button class="btn-ghost btn-sm p-1.5 text-red-500 hover:text-red-700 hover:bg-red-50" onclick={() => confirmDelete(a.id)} title="Hapus">
                    <Trash2 size={14} />
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  <div class="mt-3 text-xs text-gray-400">{filtered.length} akun ditemukan</div>
</div>

<!-- Modal Tambah/Edit -->
<Modal open={showModal} title={editingId ? 'Edit Akun' : 'Tambah Akun'} onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label" for="akun-kode">Kode Akun *</label>
          <input id="akun-kode" class="input" placeholder="1-1001" bind:value={form.kode} />
        </div>
        <div>
          <label class="label" for="akun-tipe">Tipe *</label>
          <select id="akun-tipe" class="input" bind:value={form.tipe} onchange={onTipeChange}>
            {#each tipeOptions as t}
              <option value={t}>{t}</option>
            {/each}
          </select>
        </div>
      </div>
      <div>
        <label class="label" for="akun-nama">Nama Akun *</label>
        <input id="akun-nama" class="input" placeholder="Masukkan nama akun..." bind:value={form.nama} />
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label" for="akun-sub-tipe">Sub Tipe</label>
          <input id="akun-sub-tipe" class="input" placeholder="Lancar, Tetap, dll." bind:value={form.sub_tipe} />
        </div>
        <div>
          <label class="label" for="akun-saldo-normal">Saldo Normal</label>
          <select id="akun-saldo-normal" class="input" bind:value={form.saldo_normal}>
            <option value="Debit">Debit</option>
            <option value="Kredit">Kredit</option>
          </select>
        </div>
      </div>
      {#if !editingId}
        <div>
          <label class="label" for="akun-saldo">Saldo Awal</label>
          <input id="akun-saldo" class="input" type="number" placeholder="0" bind:value={form.saldo} />
        </div>
      {/if}
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn-secondary" onclick={() => showModal = false}>Batal</button>
    <button class="btn-primary" onclick={save} disabled={saving}>
      {saving ? 'Menyimpan...' : 'Simpan'}
    </button>
  {/snippet}
</Modal>

<ConfirmDialog
  open={showConfirm}
  message="Apakah Anda yakin ingin menghapus akun ini? Tindakan ini tidak dapat dibatalkan."
  onConfirm={doDelete}
  onCancel={() => showConfirm = false}
  loading={deleting}
/>
