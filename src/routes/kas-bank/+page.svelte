<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Trash2, ArrowUpCircle, ArrowDownCircle, ArrowLeftRight, X } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, hariIni, formatCurrency, formatDate } from '$lib/stores/app.svelte';
  import type { KasBank, KasBankInput, Akun } from '$lib/types';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import * as Table from '$lib/components/ui/table/index.js';
  import * as Card from '$lib/components/ui/card/index.js';
  import { DatePicker } from '$lib/components/ui/date-picker/index.js';

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


  let form = $state<KasBankInput>({
    tanggal: hariIni(),
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
    form = { tanggal: hariIni(), keterangan: null, tipe, akun_id: akunKasBankList[0]?.id || 0, akun_lawan_id: null, jumlah: 0 };
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
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-xl font-semibold text-foreground">Kas & Bank</h1>
      <p class="text-sm text-muted-foreground mt-0.5">Manajemen transaksi kas dan bank</p>
    </div>
    <div class="flex gap-2">
      <Button class="bg-green-600 hover:bg-green-700 text-white" onclick={() => openCreate('Masuk')}>
        <ArrowUpCircle size={16} /> Kas Masuk
      </Button>
      <Button class="bg-red-600 hover:bg-red-700 text-white" onclick={() => openCreate('Keluar')}>
        <ArrowDownCircle size={16} /> Kas Keluar
      </Button>
    </div>
  </div>

  <!-- Summary -->
  <div class="grid grid-cols-3 gap-4 mb-6">
    <Card.Root class="p-5">
      <p class="text-xs text-muted-foreground mb-1">Total Kas Masuk</p>
      <p class="text-lg font-bold text-green-600">{formatCurrency(totalMasuk)}</p>
    </Card.Root>
    <Card.Root class="p-5">
      <p class="text-xs text-muted-foreground mb-1">Total Kas Keluar</p>
      <p class="text-lg font-bold text-red-600">{formatCurrency(totalKeluar)}</p>
    </Card.Root>
    <Card.Root class="p-5">
      <p class="text-xs text-muted-foreground mb-1">Saldo Bersih</p>
      <p class="text-lg font-bold {totalMasuk - totalKeluar >= 0 ? 'text-foreground' : 'text-red-600'}">{formatCurrency(totalMasuk - totalKeluar)}</p>
    </Card.Root>
  </div>

  <!-- Filters -->
  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
      <Input class="pl-9" placeholder="Cari transaksi..." bind:value={search} />
    </div>
    <Select.Root bind:value={filterTipe}>
      <Select.Trigger class="w-40">
        <Select.Value placeholder="Semua Tipe" />
      </Select.Trigger>
      <Select.Content>
        <Select.Item value="" label="Semua Tipe">Semua Tipe</Select.Item>
        <Select.Item value="Masuk">Masuk</Select.Item>
        <Select.Item value="Keluar">Keluar</Select.Item>
      </Select.Content>
    </Select.Root>
    {#if search || filterTipe}
      <Button variant="ghost" size="sm" onclick={() => { search = ''; filterTipe = ''; }} class="text-muted-foreground gap-1.5">
        <X size={14} /> Reset Filter
      </Button>
    {/if}
  </div>

  <div class="overflow-x-auto rounded-xl border border-border">
    <Table.Root>
      <Table.Header>
        <Table.Row>
          <Table.Head>Nomor</Table.Head>
          <Table.Head>Tanggal</Table.Head>
          <Table.Head>Keterangan</Table.Head>
          <Table.Head>Tipe</Table.Head>
          <Table.Head>Akun</Table.Head>
          <Table.Head class="text-right">Jumlah</Table.Head>
          <Table.Head class="text-center">Aksi</Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#if loading}
          <Table.Row><Table.Cell colspan={7} class="text-center py-12 text-muted-foreground">Memuat data...</Table.Cell></Table.Row>
        {:else if filtered.length === 0}
          <Table.Row><Table.Cell colspan={7} class="text-center py-12 text-muted-foreground">Tidak ada transaksi</Table.Cell></Table.Row>
        {:else}
          {#each filtered as k}
            <Table.Row>
              <Table.Cell class="font-mono text-xs font-semibold text-primary">{k.nomor}</Table.Cell>
              <Table.Cell class="text-xs text-muted-foreground">{formatDate(k.tanggal)}</Table.Cell>
              <Table.Cell class="text-foreground">{k.keterangan || '-'}</Table.Cell>
              <Table.Cell>
                {#if k.tipe === 'Masuk'}
                  <Badge class="bg-green-100 text-green-700 hover:bg-green-100 flex items-center gap-1 w-fit">
                    <ArrowUpCircle size={11} />{k.tipe}
                  </Badge>
                {:else if k.tipe === 'Keluar'}
                  <Badge class="bg-red-100 text-red-700 hover:bg-red-100 flex items-center gap-1 w-fit">
                    <ArrowDownCircle size={11} />{k.tipe}
                  </Badge>
                {:else}
                  <Badge class="bg-blue-100 text-blue-700 hover:bg-blue-100 flex items-center gap-1 w-fit">
                    <ArrowLeftRight size={11} />{k.tipe}
                  </Badge>
                {/if}
              </Table.Cell>
              <Table.Cell class="text-xs text-muted-foreground">{k.akun_nama}</Table.Cell>
              <Table.Cell class="text-right font-semibold {k.tipe === 'Masuk' ? 'text-green-600' : 'text-red-600'}">
                {formatCurrency(k.jumlah)}
              </Table.Cell>
              <Table.Cell>
                <div class="flex justify-center">
                  <Button variant="ghost" size="icon-sm" class="text-destructive hover:text-destructive hover:bg-destructive/10"
                    onclick={() => { deletingId = k.id; showConfirm = true; }}>
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
</div>

<Modal open={showModal} title={form.tipe === 'Masuk' ? 'Kas Masuk' : 'Kas Keluar'} onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="kb-tanggal">Tanggal *</Label>
          <DatePicker id="kb-tanggal" bind:value={form.tanggal} />
        </div>
        <div>
          <Label for="kb-akun">Akun Kas/Bank *</Label>
          <Select.Root
            value={form.akun_id > 0 ? String(form.akun_id) : ''}
            onValueChange={(v) => { form.akun_id = Number(v) || 0; }}
          >
            <Select.Trigger class="w-full" id="kb-akun">
              <Select.Value placeholder="-- Pilih Akun --" />
            </Select.Trigger>
            <Select.Content>
              {#each akunKasBankList as a}
                <Select.Item value={String(a.id)}>{a.nama}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
      </div>
      <div>
        <Label for="kb-akun-lawan">Akun Lawan *</Label>
        <Select.Root
          value={form.akun_lawan_id ? String(form.akun_lawan_id) : ''}
          onValueChange={(v) => { form.akun_lawan_id = v ? Number(v) : null; }}
        >
          <Select.Trigger class="w-full" id="kb-akun-lawan">
            <Select.Value placeholder="-- Pilih Akun Lawan --" />
          </Select.Trigger>
          <Select.Content>
            {#each akunAll.filter(x => !x.header && x.aktif && x.id !== form.akun_id) as a}
              <Select.Item value={String(a.id)}>{a.kode} - {a.nama}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <div>
        <Label for="kb-jumlah">Jumlah *</Label>
        <Input id="kb-jumlah" type="number" placeholder="0" bind:value={form.jumlah} />
      </div>
      <div>
        <Label for="kb-ket">Keterangan</Label>
        <Input id="kb-ket" placeholder="Keterangan transaksi..." bind:value={form.keterangan} />
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <Button variant="outline" onclick={() => showModal = false}>Batal</Button>
    <Button onclick={save} disabled={saving}>{saving ? 'Menyimpan...' : 'Simpan'}</Button>
  {/snippet}
</Modal>

<ConfirmDialog open={showConfirm} message="Hapus transaksi ini? Saldo akun akan dikembalikan." onConfirm={doDelete} onCancel={() => showConfirm = false} loading={deleting} />
