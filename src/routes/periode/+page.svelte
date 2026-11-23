<script lang="ts">
  import { onMount } from 'svelte';
  import { Lock, FileText, Save } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import {
    callCommand, showNotification, formatCurrency, formatDate, loadAppState, namaPerusahaan,
  } from '$lib/stores/app.svelte';
  import type { Periode, PeriodeInput, LaporanArsip, LabaRugiItem, NeracaItem } from '$lib/types';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import * as Card from '$lib/components/ui/card/index.js';
  import * as Table from '$lib/components/ui/table/index.js';
  import { DatePicker } from '$lib/components/ui/date-picker/index.js';

  let list = $state<Periode[]>([]);
  let loading = $state(true);
  let saving = $state(false);

  const aktif = $derived(list.find(p => p.status === 'Aktif') ?? null);
  const tertutup = $derived(list.filter(p => p.status === 'Ditutup'));

  let editForm = $state<PeriodeInput>({ nama: '', tanggal_mulai: '', tanggal_selesai: '' });
  let baruForm = $state<PeriodeInput>({ nama: '', tanggal_mulai: '', tanggal_selesai: '' });
  let showTutup = $state(false);

  let arsipPeriode = $state<Periode | null>(null);
  let arsip = $state<LaporanArsip[]>([]);
  let arsipTab = $state<'LabaRugi' | 'Neraca'>('LabaRugi');

  function isoPlus(iso: string, days: number): string {
    const d = new Date(iso + 'T00:00:00Z');
    d.setUTCDate(d.getUTCDate() + days);
    return d.toISOString().slice(0, 10);
  }

  async function load() {
    loading = true;
    try {
      list = await callCommand<Periode[]>('get_periode');
      if (aktif) {
        editForm = { nama: aktif.nama, tanggal_mulai: aktif.tanggal_mulai, tanggal_selesai: aktif.tanggal_selesai };
      }
    } finally {
      loading = false;
    }
  }

  async function simpanPeriode() {
    if (!aktif) return;
    saving = true;
    try {
      await callCommand('update_periode', { id: aktif.id, input: editForm });
      await loadAppState();
      await load();
      showNotification('success', 'Periode berhasil diperbarui');
    } catch (e) {
      showNotification('error', String(e));
    } finally {
      saving = false;
    }
  }

  function bukaTutup() {
    if (!aktif) return;
    const mulai = isoPlus(aktif.tanggal_selesai, 1);
    const selesai = isoPlus(`${Number(mulai.slice(0, 4)) + 1}${mulai.slice(4)}`, -1);
    baruForm = { nama: `Tahun ${mulai.slice(0, 4)}`, tanggal_mulai: mulai, tanggal_selesai: selesai };
    showTutup = true;
  }

  async function tutupBuku() {
    saving = true;
    try {
      await callCommand('tutup_periode', { periodeBaru: baruForm });
      showTutup = false;
      await loadAppState();
      await load();
      showNotification('success', 'Tutup buku berhasil. Laporan periode lalu telah diarsipkan.');
    } catch (e) {
      showNotification('error', String(e));
    } finally {
      saving = false;
    }
  }

  async function lihatArsip(p: Periode) {
    try {
      arsip = await callCommand<LaporanArsip[]>('get_arsip_laporan', { periodeId: p.id });
      arsipTab = 'LabaRugi';
      arsipPeriode = p;
    } catch (e) {
      showNotification('error', String(e));
    }
  }

  const arsipAktif = $derived(arsip.find(a => a.jenis === arsipTab));
  const labaRugiItems = $derived((arsipAktif?.data.items ?? []) as LabaRugiItem[]);
  const neracaItems = $derived((arsipAktif?.data.items ?? []) as NeracaItem[]);
  const total = (items: { saldo: number }[]) => items.reduce((s, i) => s + i.saldo, 0);

  onMount(load);
</script>

<div class="p-6 max-w-4xl">
  <div class="mb-6">
    <h1 class="text-xl font-semibold text-foreground">Periode & Arsip Laporan</h1>
    <p class="text-sm text-muted-foreground mt-0.5">
      Transaksi hanya dapat dicatat pada periode aktif. Tutup buku mengarsipkan laporan dan membuka periode berikutnya.
    </p>
  </div>

  {#if loading}
    <div class="text-muted-foreground">Memuat data...</div>
  {:else}
    {#if aktif}
      <Card.Root class="mb-6">
        <Card.Header>
          <Card.Title class="flex items-center gap-2">Periode Aktif <Badge>Aktif</Badge></Card.Title>
        </Card.Header>
        <Card.Content class="space-y-4">
          <div>
            <Label for="per-nama">Nama Periode</Label>
            <Input id="per-nama" bind:value={editForm.nama} />
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <Label for="per-mulai">Tanggal Mulai</Label>
              <DatePicker id="per-mulai" bind:value={editForm.tanggal_mulai} />
            </div>
            <div>
              <Label for="per-selesai">Tanggal Selesai</Label>
              <DatePicker id="per-selesai" bind:value={editForm.tanggal_selesai} />
            </div>
          </div>
          <div class="flex justify-between">
            <Button variant="outline" onclick={simpanPeriode} disabled={saving}>
              <Save size={16} /> Simpan Perubahan
            </Button>
            <Button onclick={bukaTutup}>
              <Lock size={16} /> Tutup Buku
            </Button>
          </div>
        </Card.Content>
      </Card.Root>
    {:else}
      <p class="text-destructive mb-6">Belum ada periode aktif.</p>
    {/if}

    <h2 class="text-sm font-semibold mb-2">Periode Tertutup</h2>
    <div class="overflow-x-auto rounded-xl border border-border">
      <Table.Root>
        <Table.Header>
          <Table.Row>
            <Table.Head>Periode</Table.Head>
            <Table.Head>Rentang</Table.Head>
            <Table.Head>Ditutup</Table.Head>
            <Table.Head class="text-center">Laporan</Table.Head>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {#if tertutup.length === 0}
            <Table.Row>
              <Table.Cell colspan={4} class="text-center py-10 text-muted-foreground">Belum ada periode yang ditutup</Table.Cell>
            </Table.Row>
          {:else}
            {#each tertutup as p}
              <Table.Row>
                <Table.Cell class="font-medium">{p.nama}</Table.Cell>
                <Table.Cell>{formatDate(p.tanggal_mulai)} – {formatDate(p.tanggal_selesai)}</Table.Cell>
                <Table.Cell>{p.closed_at ?? '-'}</Table.Cell>
                <Table.Cell class="text-center">
                  <Button variant="ghost" size="sm" onclick={() => lihatArsip(p)}>
                    <FileText size={14} /> Lihat
                  </Button>
                </Table.Cell>
              </Table.Row>
            {/each}
          {/if}
        </Table.Body>
      </Table.Root>
    </div>
  {/if}
</div>

<!-- Tutup buku -->
<Modal open={showTutup} title="Tutup Buku" onClose={() => showTutup = false}>
  {#snippet children()}
    <div class="space-y-4">
      <p class="text-sm text-muted-foreground">
        Periode <strong>{aktif?.nama}</strong> akan dikunci. Sistem membuat jurnal penutup (pendapatan dan beban
        dipindahkan ke akun Laba Ditahan), menyimpan Laba Rugi dan Neraca periode ini sebagai arsip, lalu membuka
        periode baru. Tindakan ini tidak dapat dibatalkan.
      </p>
      <div>
        <Label for="baru-nama">Nama Periode Baru</Label>
        <Input id="baru-nama" bind:value={baruForm.nama} />
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="baru-mulai">Mulai</Label>
          <DatePicker id="baru-mulai" bind:value={baruForm.tanggal_mulai} />
        </div>
        <div>
          <Label for="baru-selesai">Selesai</Label>
          <DatePicker id="baru-selesai" bind:value={baruForm.tanggal_selesai} />
        </div>
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <Button variant="outline" onclick={() => showTutup = false}>Batal</Button>
    <Button onclick={tutupBuku} disabled={saving}>{saving ? 'Memproses...' : 'Tutup Buku'}</Button>
  {/snippet}
</Modal>

<!-- Arsip laporan -->
<Modal open={arsipPeriode !== null} title={`Arsip Laporan — ${arsipPeriode?.nama ?? ''}`} onClose={() => arsipPeriode = null}>
  {#snippet children()}
    <div class="space-y-3">
      <div class="text-sm">
        <div class="font-semibold">{namaPerusahaan()}</div>
        {#if arsipPeriode}
          <div class="text-muted-foreground">{formatDate(arsipPeriode.tanggal_mulai)} – {formatDate(arsipPeriode.tanggal_selesai)}</div>
        {/if}
      </div>
      <div class="flex gap-2">
        <Button size="sm" variant={arsipTab === 'LabaRugi' ? 'secondary' : 'ghost'} onclick={() => arsipTab = 'LabaRugi'}>Laba Rugi</Button>
        <Button size="sm" variant={arsipTab === 'Neraca' ? 'secondary' : 'ghost'} onclick={() => arsipTab = 'Neraca'}>Neraca</Button>
      </div>

      {#if !arsipAktif}
        <p class="text-sm text-muted-foreground">Arsip tidak tersedia.</p>
      {:else if arsipTab === 'LabaRugi'}
        {@const pend = labaRugiItems.filter(i => i.tipe === 'Pendapatan')}
        {@const beb = labaRugiItems.filter(i => i.tipe === 'Beban')}
        <Table.Root>
          <Table.Body>
            <Table.Row class="bg-muted/40"><Table.Cell colspan={2} class="font-semibold">Pendapatan</Table.Cell></Table.Row>
            {#each pend as i}
              <Table.Row><Table.Cell>{i.kode} {i.nama}</Table.Cell><Table.Cell class="text-right">{formatCurrency(i.saldo)}</Table.Cell></Table.Row>
            {/each}
            <Table.Row class="bg-muted/40"><Table.Cell colspan={2} class="font-semibold">Beban</Table.Cell></Table.Row>
            {#each beb as i}
              <Table.Row><Table.Cell>{i.kode} {i.nama}</Table.Cell><Table.Cell class="text-right">{formatCurrency(i.saldo)}</Table.Cell></Table.Row>
            {/each}
            <Table.Row>
              <Table.Cell class="font-bold">{total(pend) - total(beb) >= 0 ? 'Laba Bersih' : 'Rugi Bersih'}</Table.Cell>
              <Table.Cell class="text-right font-bold">{formatCurrency(total(pend) - total(beb))}</Table.Cell>
            </Table.Row>
          </Table.Body>
        </Table.Root>
      {:else}
        {#each ['Aset', 'Kewajiban', 'Modal'] as tipe}
          {@const rows = neracaItems.filter(i => i.tipe === tipe)}
          <Table.Root>
            <Table.Body>
              <Table.Row class="bg-muted/40"><Table.Cell colspan={2} class="font-semibold">{tipe}</Table.Cell></Table.Row>
              {#each rows as i}
                <Table.Row><Table.Cell>{i.kode} {i.nama}</Table.Cell><Table.Cell class="text-right">{formatCurrency(i.saldo)}</Table.Cell></Table.Row>
              {/each}
              <Table.Row>
                <Table.Cell class="font-semibold">Total {tipe}</Table.Cell>
                <Table.Cell class="text-right font-semibold">{formatCurrency(total(rows))}</Table.Cell>
              </Table.Row>
            </Table.Body>
          </Table.Root>
        {/each}
      {/if}
    </div>
  {/snippet}
  {#snippet footer()}
    <Button variant="outline" onclick={() => arsipPeriode = null}>Tutup</Button>
  {/snippet}
</Modal>
