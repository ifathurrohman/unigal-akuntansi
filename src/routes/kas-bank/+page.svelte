<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Trash2, ArrowUpCircle, ArrowDownCircle, ArrowLeftRight } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, formatCurrency, formatDate } from '$lib/stores/app.svelte';
  import type { KasBank, KasBankInput, Akun } from '$lib/types';

  let list = $state<KasBank[]>([]);
  let akunKasBankList = $state<Akun[]>([]);
  let akunAll = $state<Akun[]>([]);
  let loading = $state(true);
  let search = $state('');
  let filterTipe = $state('');
  let showModal = $state(false);
  let showConfirm = $state(false);
  let deletingId = $state<number | null>(null);
  let saving = $state(false);
  let deleting = $state(false);

  const today = new Date().toISOString().slice(0, 10);

  let form = $state<KasBankInput>({
    tanggal: today,
    keterangan: null,
    tipe: 'Masuk',
    akun_id: 0,
    akun_lawan_id: null,
    jumlah: 0,
  });

  const filtered = $derived(
    list.filter(k => {
      const q = search.toLowerCase();
      const matchQ = !q || k.nomor.toLowerCase().includes(q) || (k.keterangan || '').toLowerCase().includes(q);
      const matchTipe = !filterTipe || k.tipe === filterTipe;
      return matchQ && matchTipe;
    })
  );

  const totalMasuk = $derived(list.filter(k => k.tipe === 'Masuk').reduce((s, k) => s + k.jumlah, 0));
  const totalKeluar = $derived(list.filter(k => k.tipe === 'Keluar').reduce((s, k) => s + k.jumlah, 0));

  async function load() {
    loading = true;
    try {
      [list, akunKasBankList, akunAll] = await Promise.all([
        callCommand<KasBank[]>('get_kas_bank'),
        callCommand<Akun[]>('get_akun_kas_bank'),
        callCommand<Akun[]>('get_akun'),
      ]);
    } finally { loading = false; }
  }

  function openCreate(tipe: 'Masuk' | 'Keluar') {
    form = { tanggal: today, keterangan: null, tipe, akun_id: akunKasBankList[0]?.id || 0, akun_lawan_id: null, jumlah: 0 };
    showModal = true;
  }

  async function save() {
    if (!form.akun_id || !form.jumlah) return;
    saving = true;
    try {
      const input = { ...form, keterangan: form.keterangan || null, akun_lawan_id: form.akun_lawan_id || null, jumlah: Number(form.jumlah) };
      await callCommand('create_kas_bank', { input });
      showNotification('success', `Kas ${form.tipe.toLowerCase()} berhasil disimpan`);
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
      await callCommand('delete_kas_bank', { id: deletingId });
      showNotification('success', 'Transaksi berhasil dihapus');
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
      <h1 class="page-title">Kas & Bank</h1>
      <p class="page-subtitle">Manajemen transaksi kas dan bank</p>
    </div>
    <div class="flex gap-2">
      <button class="btn bg-green-600 text-white hover:bg-green-700 shadow-sm" onclick={() => openCreate('Masuk')}>
        <ArrowUpCircle size={16} /> Kas Masuk
      </button>
      <button class="btn bg-red-600 text-white hover:bg-red-700 shadow-sm" onclick={() => openCreate('Keluar')}>
        <ArrowDownCircle size={16} /> Kas Keluar
      </button>
    </div>
  </div>

  <!-- Summary -->
  <div class="grid grid-cols-3 gap-4 mb-6">
    <div class="stat-card">
      <p class="text-xs text-gray-500 mb-1">Total Kas Masuk</p>
      <p class="text-lg font-bold text-green-600">{formatCurrency(totalMasuk)}</p>
    </div>
    <div class="stat-card">
      <p class="text-xs text-gray-500 mb-1">Total Kas Keluar</p>
      <p class="text-lg font-bold text-red-600">{formatCurrency(totalKeluar)}</p>
    </div>
    <div class="stat-card">
      <p class="text-xs text-gray-500 mb-1">Saldo Bersih</p>
      <p class="text-lg font-bold {totalMasuk - totalKeluar >= 0 ? 'text-gray-900' : 'text-red-600'}">{formatCurrency(totalMasuk - totalKeluar)}</p>
    </div>
  </div>

  <!-- Filters -->
  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
      <input class="input pl-9" placeholder="Cari transaksi..." bind:value={search} />
    </div>
    <select class="input w-40" bind:value={filterTipe}>
      <option value="">Semua Tipe</option>
      <option value="Masuk">Masuk</option>
      <option value="Keluar">Keluar</option>
    </select>
  </div>

  <div class="table-wrapper">
    <table class="data-table">
      <thead>
        <tr>
          <th>Nomor</th>
          <th>Tanggal</th>
          <th>Keterangan</th>
          <th>Tipe</th>
          <th>Akun</th>
          <th class="text-right">Jumlah</th>
          <th class="text-center">Aksi</th>
        </tr>
      </thead>
      <tbody>
        {#if loading}
          <tr><td colspan="7" class="text-center py-12 text-gray-400">Memuat data...</td></tr>
        {:else if filtered.length === 0}
          <tr><td colspan="7" class="text-center py-12 text-gray-400">Tidak ada transaksi</td></tr>
        {:else}
          {#each filtered as k}
            <tr>
              <td class="font-mono text-xs font-semibold text-primary-600">{k.nomor}</td>
              <td class="text-xs text-gray-500">{formatDate(k.tanggal)}</td>
              <td class="text-gray-700">{k.keterangan || '-'}</td>
              <td>
                {#if k.tipe === 'Masuk'}
                  <span class="badge-green flex items-center gap-1 w-fit"><ArrowUpCircle size={11} />{k.tipe}</span>
                {:else if k.tipe === 'Keluar'}
                  <span class="badge-red flex items-center gap-1 w-fit"><ArrowDownCircle size={11} />{k.tipe}</span>
                {:else}
                  <span class="badge-blue flex items-center gap-1 w-fit"><ArrowLeftRight size={11} />{k.tipe}</span>
                {/if}
              </td>
              <td class="text-xs text-gray-600">{k.akun_nama}</td>
              <td class="text-right font-semibold {k.tipe === 'Masuk' ? 'text-green-600' : 'text-red-600'}">
                {formatCurrency(k.jumlah)}
              </td>
              <td>
                <div class="flex justify-center">
                  <button class="btn-ghost btn-sm p-1.5 text-red-400 hover:text-red-600 hover:bg-red-50"
                    onclick={() => { deletingId = k.id; showConfirm = true; }}>
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
</div>

<Modal open={showModal} title={form.tipe === 'Masuk' ? 'Kas Masuk' : 'Kas Keluar'} onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label">Tanggal *</label>
          <input class="input" type="date" bind:value={form.tanggal} />
        </div>
        <div>
          <label class="label">Akun Kas/Bank *</label>
          <select class="input" bind:value={form.akun_id}>
            <option value={0}>-- Pilih Akun --</option>
            {#each akunKasBankList as a}
              <option value={a.id}>{a.nama}</option>
            {/each}
          </select>
        </div>
      </div>
      <div>
        <label class="label">Akun Lawan</label>
        <select class="input" bind:value={form.akun_lawan_id}>
          <option value={null}>-- Pilih Akun Lawan --</option>
          {#each akunAll as a}
            <option value={a.id}>{a.kode} - {a.nama}</option>
          {/each}
        </select>
      </div>
      <div>
        <label class="label">Jumlah *</label>
        <input class="input" type="number" placeholder="0" bind:value={form.jumlah} />
      </div>
      <div>
        <label class="label">Keterangan</label>
        <input class="input" placeholder="Keterangan transaksi..." bind:value={form.keterangan} />
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn-secondary" onclick={() => showModal = false}>Batal</button>
    <button class="btn-primary" onclick={save} disabled={saving}>{saving ? 'Menyimpan...' : 'Simpan'}</button>
  {/snippet}
</Modal>

<ConfirmDialog open={showConfirm} message="Hapus transaksi ini? Saldo akun akan dikembalikan." onConfirm={doDelete} onCancel={() => showConfirm = false} loading={deleting} />
