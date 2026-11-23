<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Pencil, Trash2, Phone, Mail } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, formatCurrency } from '$lib/stores/app.svelte';
  import type { Pelanggan, PelangganInput } from '$lib/types';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import * as Table from '$lib/components/ui/table/index.js';

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
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-xl font-semibold text-foreground">Pelanggan</h1>
      <p class="text-sm text-muted-foreground mt-0.5">{list.length} pelanggan terdaftar</p>
    </div>
    <Button onclick={openCreate}><Plus size={16} /> Tambah Pelanggan</Button>
  </div>

  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
      <Input class="pl-9" placeholder="Cari pelanggan..." bind:value={search} />
    </div>
  </div>

  <div class="overflow-x-auto rounded-xl border border-border">
    <Table.Root>
      <Table.Header>
        <Table.Row>
          <Table.Head>Kode</Table.Head>
          <Table.Head>Nama</Table.Head>
          <Table.Head>Telepon</Table.Head>
          <Table.Head>Email</Table.Head>
          <Table.Head class="text-right">Piutang</Table.Head>
          <Table.Head class="text-center">Aksi</Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#if loading}
          <Table.Row><Table.Cell colspan={6} class="text-center py-12 text-muted-foreground">Memuat data...</Table.Cell></Table.Row>
        {:else if filtered.length === 0}
          <Table.Row><Table.Cell colspan={6} class="text-center py-12 text-muted-foreground">Tidak ada data</Table.Cell></Table.Row>
        {:else}
          {#each filtered as p}
            <Table.Row>
              <Table.Cell class="font-mono text-xs font-medium text-muted-foreground">{p.kode}</Table.Cell>
              <Table.Cell class="font-medium text-foreground">{p.nama}</Table.Cell>
              <Table.Cell>
                {#if p.telepon}
                  <div class="flex items-center gap-1 text-muted-foreground text-xs">
                    <Phone size={12} />{p.telepon}
                  </div>
                {:else}
                  <span class="text-muted-foreground/40">-</span>
                {/if}
              </Table.Cell>
              <Table.Cell>
                {#if p.email}
                  <div class="flex items-center gap-1 text-muted-foreground text-xs">
                    <Mail size={12} />{p.email}
                  </div>
                {:else}
                  <span class="text-muted-foreground/40">-</span>
                {/if}
              </Table.Cell>
              <Table.Cell class="text-right font-medium {p.piutang > 0 ? 'text-orange-600' : 'text-muted-foreground'}">
                {formatCurrency(p.piutang)}
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

<Modal open={showModal} title={editingId ? 'Edit Pelanggan' : 'Tambah Pelanggan'} onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="plg-kode">Kode *</Label>
          <Input id="plg-kode" placeholder="PLG-001" bind:value={form.kode} />
        </div>
        <div>
          <Label for="plg-nama">Nama *</Label>
          <Input id="plg-nama" placeholder="Nama pelanggan" bind:value={form.nama} />
        </div>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="plg-telepon">Telepon</Label>
          <Input id="plg-telepon" placeholder="08xx" bind:value={form.telepon} />
        </div>
        <div>
          <Label for="plg-email">Email</Label>
          <Input id="plg-email" type="email" placeholder="email@domain.com" bind:value={form.email} />
        </div>
      </div>
      <div>
        <Label for="plg-alamat">Alamat</Label>
        <textarea id="plg-alamat" class="flex min-h-15 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" rows="2" placeholder="Alamat lengkap..." bind:value={form.alamat}></textarea>
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <Button variant="outline" onclick={() => showModal = false}>Batal</Button>
    <Button onclick={save} disabled={saving}>{saving ? 'Menyimpan...' : 'Simpan'}</Button>
  {/snippet}
</Modal>

<ConfirmDialog open={showConfirm} message="Hapus pelanggan ini?" onConfirm={doDelete} onCancel={() => showConfirm = false} loading={deleting} />
