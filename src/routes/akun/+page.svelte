<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Pencil, Trash2, FileText, X } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, formatCurrency } from '$lib/stores/app.svelte';
  import type { Akun, AkunInput } from '$lib/types';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import * as Table from '$lib/components/ui/table/index.js';

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
    header: false,
    kas_bank: false,
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
    form = { kode: '', nama: '', tipe: 'Aset', sub_tipe: null, saldo_normal: 'Debit', header: false, kas_bank: false };
    showModal = true;
  }

  function openEdit(a: Akun) {
    editingId = a.id;
    form = { kode: a.kode, nama: a.nama, tipe: a.tipe, sub_tipe: a.sub_tipe, saldo_normal: a.saldo_normal, header: a.header, kas_bank: a.kas_bank };
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

  function getTipeBadgeClass(tipe: string) {
    const map: Record<string, string> = {
      Aset: 'bg-blue-100 text-blue-700 hover:bg-blue-100',
      Kewajiban: 'bg-red-100 text-red-700 hover:bg-red-100',
      Modal: 'bg-green-100 text-green-700 hover:bg-green-100',
      Pendapatan: 'bg-yellow-100 text-yellow-700 hover:bg-yellow-100',
      Beban: '',
    };
    return map[tipe] ?? '';
  }

  onMount(load);
</script>

<div class="p-6">
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-xl font-semibold text-foreground">Daftar Akun</h1>
      <p class="text-sm text-muted-foreground mt-0.5">Chart of Accounts (CoA)</p>
    </div>
    <Button onclick={openCreate}>
      <Plus size={16} /> Tambah Akun
    </Button>
  </div>

  <!-- Filters -->
  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
      <Input class="pl-9" placeholder="Cari kode atau nama akun..." bind:value={search} />
    </div>
    <Select.Root bind:value={filterTipe}>
      <Select.Trigger class="w-40">
        <Select.Value placeholder="Semua Tipe" />
      </Select.Trigger>
      <Select.Content>
        <Select.Item value="" label="Semua Tipe">Semua Tipe</Select.Item>
        {#each tipeOptions as t}
          <Select.Item value={t}>{t}</Select.Item>
        {/each}
      </Select.Content>
    </Select.Root>
    {#if search || filterTipe}
      <Button variant="ghost" size="sm" onclick={() => { search = ''; filterTipe = ''; }} class="text-muted-foreground gap-1.5">
        <X size={14} /> Reset Filter
      </Button>
    {/if}
  </div>

  <!-- Table -->
  <div class="overflow-x-auto rounded-xl border border-border">
    <Table.Root>
      <Table.Header>
        <Table.Row>
          <Table.Head>Kode</Table.Head>
          <Table.Head>Nama Akun</Table.Head>
          <Table.Head>Tipe</Table.Head>
          <Table.Head>Saldo Normal</Table.Head>
          <Table.Head class="text-right">Saldo</Table.Head>
          <Table.Head class="text-center">Aksi</Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#if loading}
          <Table.Row>
            <Table.Cell colspan={6} class="text-center py-12 text-muted-foreground">Memuat data...</Table.Cell>
          </Table.Row>
        {:else if filtered.length === 0}
          <Table.Row>
            <Table.Cell colspan={6} class="text-center py-12 text-muted-foreground">Tidak ada data</Table.Cell>
          </Table.Row>
        {:else}
          {#each filtered as a}
            <Table.Row>
              <Table.Cell class="font-mono text-xs font-medium text-muted-foreground">{a.kode}</Table.Cell>
              <Table.Cell class="font-medium {a.header ? 'font-bold' : ''}">
                {a.nama}
                {#if a.header}<Badge variant="outline" class="ml-2">Induk</Badge>{/if}
                {#if a.kas_bank}<Badge variant="outline" class="ml-2">Kas/Bank</Badge>{/if}
              </Table.Cell>
              <Table.Cell>
                {#if a.tipe === 'Beban'}
                  <Badge variant="secondary">{a.tipe}</Badge>
                {:else}
                  <Badge class={getTipeBadgeClass(a.tipe)}>{a.tipe}</Badge>
                {/if}
              </Table.Cell>
              <Table.Cell>
                <Badge class={a.saldo_normal === 'Debit' ? 'bg-blue-100 text-blue-700 hover:bg-blue-100' : 'bg-green-100 text-green-700 hover:bg-green-100'}>
                  {a.saldo_normal}
                </Badge>
              </Table.Cell>
              <Table.Cell class="text-right font-medium {a.saldo < 0 ? 'text-red-600' : 'text-foreground'}">
                {formatCurrency(a.saldo)}
              </Table.Cell>
              <Table.Cell>
                <div class="flex items-center justify-center gap-1">
                  <Button variant="ghost" size="icon-sm" onclick={() => openEdit(a)} title="Edit">
                    <Pencil size={14} />
                  </Button>
                  <Button variant="ghost" size="icon-sm" class="text-destructive hover:text-destructive hover:bg-destructive/10" onclick={() => confirmDelete(a.id)} title="Hapus">
                    <Trash2 size={14} />
                  </Button>
                </div>
              </Table.Cell>
            </Table.Row>
          {/each}
        {/if}
      </Table.Body>
    </Table.Root>
  </div>

  <div class="mt-3 text-xs text-muted-foreground">{filtered.length} akun ditemukan</div>
</div>

<!-- Modal Tambah/Edit -->
<Modal open={showModal} title={editingId ? 'Edit Akun' : 'Tambah Akun'} onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="akun-kode">Kode Akun *</Label>
          <Input id="akun-kode" placeholder="1-1001" bind:value={form.kode} />
        </div>
        <div>
          <Label for="akun-tipe">Tipe *</Label>
          <Select.Root bind:value={form.tipe} onValueChange={onTipeChange}>
            <Select.Trigger class="w-full">
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              {#each tipeOptions as t}
                <Select.Item value={t}>{t}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
      </div>
      <div>
        <Label for="akun-nama">Nama Akun *</Label>
        <Input id="akun-nama" placeholder="Masukkan nama akun..." bind:value={form.nama} />
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="akun-sub-tipe">Sub Tipe</Label>
          <Input id="akun-sub-tipe" placeholder="Lancar, Tetap, dll." bind:value={form.sub_tipe} />
        </div>
        <div>
          <Label for="akun-saldo-normal">Saldo Normal</Label>
          <Select.Root bind:value={form.saldo_normal}>
            <Select.Trigger class="w-full">
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="Debit">Debit</Select.Item>
              <Select.Item value="Kredit">Kredit</Select.Item>
            </Select.Content>
          </Select.Root>
        </div>
      </div>
      <div class="space-y-2">
        <label class="flex items-center gap-2 text-sm">
          <input type="checkbox" bind:checked={form.header} onchange={() => { if (form.header) form.kas_bank = false; }} />
          Akun induk / grup (hanya pengelompok, tidak dipakai dalam transaksi)
        </label>
        {#if !form.header && form.tipe === 'Aset'}
          <label class="flex items-center gap-2 text-sm">
            <input type="checkbox" bind:checked={form.kas_bank} />
            Akun Kas / Bank (muncul di menu Kas & Bank)
          </label>
        {/if}
      </div>
      {#if !editingId}
        <p class="text-xs text-muted-foreground">
          Saldo awal tidak diisi di sini. Catat melalui Jurnal Umum dengan tipe "Saldo Awal".
        </p>
      {/if}
    </div>
  {/snippet}
  {#snippet footer()}
    <Button variant="outline" onclick={() => showModal = false}>Batal</Button>
    <Button onclick={save} disabled={saving}>
      {saving ? 'Menyimpan...' : 'Simpan'}
    </Button>
  {/snippet}
</Modal>

<ConfirmDialog
  open={showConfirm}
  message="Apakah Anda yakin ingin menghapus akun ini? Tindakan ini tidak dapat dibatalkan."
  onConfirm={doDelete}
  onCancel={() => showConfirm = false}
  loading={deleting}
/>
