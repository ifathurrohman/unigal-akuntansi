<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Trash2, ChevronDown, ChevronRight, X } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, hariIni, formatCurrency, formatDate } from '$lib/stores/app.svelte';
  import type { Jurnal, JurnalInput, Akun } from '$lib/types';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import * as Table from '$lib/components/ui/table/index.js';
  import { DatePicker } from '$lib/components/ui/date-picker/index.js';

  let list = $state<Jurnal[]>([]);
  let akun = $state<Akun[]>([]);
  let loading = $state(true);
  let search = $state('');
  let showModal = $state(false);
  let showConfirm = $state(false);
  let expandedId = $state<number | null>(null);
  let deletingId = $state<number | null>(null);
  let saving = $state(false);
  let deleting = $state(false);

  let form = $state<JurnalInput>({
    tanggal: hariIni(),
    keterangan: null,
    tipe: 'Umum',
    detail: [
      { akun_id: 0, keterangan: null, debit: 0, kredit: 0 },
      { akun_id: 0, keterangan: null, debit: 0, kredit: 0 },
    ],
  });

  const totalDebit = $derived(form.detail.reduce((s, d) => s + Number(d.debit || 0), 0));
  const totalKredit = $derived(form.detail.reduce((s, d) => s + Number(d.kredit || 0), 0));
  const balanced = $derived(Math.abs(totalDebit - totalKredit) < 0.01);

  const filtered = $derived(
    list.filter(j => {
      const q = search.toLowerCase();
      return !q || j.nomor.toLowerCase().includes(q) || (j.keterangan || '').toLowerCase().includes(q);
    })
  );

  async function load() {
    loading = true;
    try {
      [list, akun] = await Promise.all([
        callCommand<Jurnal[]>('get_jurnal'),
        callCommand<Akun[]>('get_akun'),
      ]);
    } finally { loading = false; }
  }

  function openCreate() {
    form = {
      tanggal: hariIni(),
      keterangan: null,
      tipe: 'Umum',
      detail: [
        { akun_id: 0, keterangan: null, debit: 0, kredit: 0 },
        { akun_id: 0, keterangan: null, debit: 0, kredit: 0 },
      ],
    };
    showModal = true;
  }

  function addRow() {
    form.detail = [...form.detail, { akun_id: 0, keterangan: null, debit: 0, kredit: 0 }];
  }

  function removeRow(i: number) {
    if (form.detail.length <= 2) return;
    form.detail = form.detail.filter((_, idx) => idx !== i);
  }

  function onDebitChange(i: number) {
    if (Number(form.detail[i].debit) > 0) form.detail[i].kredit = 0;
  }

  function onKreditChange(i: number) {
    if (Number(form.detail[i].kredit) > 0) form.detail[i].debit = 0;
  }

  async function save() {
    if (!form.tanggal || form.detail.some(d => !d.akun_id)) {
      showNotification('error', 'Harap lengkapi semua field');
      return;
    }
    if (!balanced) {
      showNotification('error', 'Total debit harus sama dengan total kredit');
      return;
    }
    saving = true;
    try {
      const input = {
        ...form,
        keterangan: form.keterangan || null,
        detail: form.detail.map(d => ({
          akun_id: Number(d.akun_id),
          keterangan: d.keterangan || null,
          debit: Number(d.debit || 0),
          kredit: Number(d.kredit || 0),
        })),
      };
      await callCommand('create_jurnal', { input });
      showNotification('success', 'Jurnal berhasil disimpan');
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
      await callCommand('delete_jurnal', { id: deletingId });
      showNotification('success', 'Jurnal berhasil dihapus');
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
      <h1 class="text-xl font-semibold text-foreground">Jurnal Umum</h1>
      <p class="text-sm text-muted-foreground mt-0.5">{list.length} entri jurnal</p>
    </div>
    <Button onclick={openCreate}><Plus size={16} /> Buat Jurnal</Button>
  </div>

  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
      <Input class="pl-9" placeholder="Cari nomor atau keterangan..." bind:value={search} />
    </div>
  </div>

  <div class="space-y-2">
    {#if loading}
      <div class="bg-card rounded-xl border border-border p-8 text-center text-muted-foreground">Memuat data...</div>
    {:else if filtered.length === 0}
      <div class="bg-card rounded-xl border border-border p-8 text-center text-muted-foreground">Tidak ada jurnal</div>
    {:else}
      {#each filtered as j}
        <div class="bg-card rounded-xl border border-border overflow-hidden">
          <!-- Header row -->
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
          <div
            class="w-full flex items-center gap-4 px-4 py-3 hover:bg-muted/30 transition-colors cursor-pointer"
            role="button"
            tabindex="0"
            onclick={() => expandedId = expandedId === j.id ? null : j.id}
          >
            <div class="shrink-0 text-muted-foreground/40">
              {#if expandedId === j.id}
                <ChevronDown size={16} />
              {:else}
                <ChevronRight size={16} />
              {/if}
            </div>
            <div class="font-mono text-xs font-semibold text-primary w-32 shrink-0">{j.nomor}</div>
            <div class="text-xs text-muted-foreground w-24 shrink-0">{formatDate(j.tanggal)}</div>
            <div class="flex-1 text-sm text-foreground truncate">{j.keterangan || '-'}</div>
            <div class="text-sm font-semibold text-foreground w-36 text-right shrink-0">{formatCurrency(j.total_debit)}</div>
            <Button
              variant="ghost"
              size="icon-sm"
              class="text-destructive hover:text-destructive hover:bg-destructive/10 shrink-0"
              onclick={(e) => { e.stopPropagation(); deletingId = j.id; showConfirm = true; }}
            >
              <Trash2 size={14} />
            </Button>
          </div>

          <!-- Detail rows -->
          {#if expandedId === j.id}
            <div class="border-t border-border bg-muted/20">
              <table class="w-full text-xs">
                <thead>
                  <tr class="border-b border-border">
                    <th class="px-8 py-2 text-left text-muted-foreground font-medium">Akun</th>
                    <th class="px-4 py-2 text-left text-muted-foreground font-medium">Keterangan</th>
                    <th class="px-4 py-2 text-right text-muted-foreground font-medium">Debit</th>
                    <th class="px-4 py-2 text-right text-muted-foreground font-medium">Kredit</th>
                  </tr>
                </thead>
                <tbody>
                  {#each j.detail as d}
                    <tr class="border-b border-border/50">
                      <td class="px-8 py-2">
                        <span class="font-mono text-muted-foreground">{d.akun_kode}</span>
                        <span class="ml-2 text-foreground">{d.akun_nama}</span>
                      </td>
                      <td class="px-4 py-2 text-muted-foreground">{d.keterangan || '-'}</td>
                      <td class="px-4 py-2 text-right font-medium {d.debit > 0 ? 'text-foreground' : 'text-muted-foreground/40'}">
                        {d.debit > 0 ? formatCurrency(d.debit) : '-'}
                      </td>
                      <td class="px-4 py-2 text-right font-medium {d.kredit > 0 ? 'text-foreground' : 'text-muted-foreground/40'}">
                        {d.kredit > 0 ? formatCurrency(d.kredit) : '-'}
                      </td>
                    </tr>
                  {/each}
                  <tr class="bg-muted/50">
                    <td colspan="2" class="px-8 py-2 font-semibold text-muted-foreground">Total</td>
                    <td class="px-4 py-2 text-right font-bold text-foreground">{formatCurrency(j.total_debit)}</td>
                    <td class="px-4 py-2 text-right font-bold text-foreground">{formatCurrency(j.total_kredit)}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<!-- Modal Jurnal -->
<Modal open={showModal} title="Buat Jurnal Umum" maxWidth="sm:max-w-3xl" onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-3 gap-3">
        <div>
          <Label for="jrn-tanggal">Tanggal *</Label>
          <DatePicker id="jrn-tanggal" bind:value={form.tanggal} />
        </div>
        <div>
          <Label for="jrn-tipe">Tipe</Label>
          <Select.Root bind:value={form.tipe}>
            <Select.Trigger class="w-full" id="jrn-tipe">
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="Umum">Jurnal Umum</Select.Item>
              <Select.Item value="Saldo Awal">Saldo Awal (Neraca Awal)</Select.Item>
              <Select.Item value="Kas">Kas</Select.Item>
              <Select.Item value="Bank">Bank</Select.Item>
            </Select.Content>
          </Select.Root>
        </div>
        <div>
          <Label for="jrn-ket">Keterangan</Label>
          <Input id="jrn-ket" placeholder="Keterangan jurnal..." bind:value={form.keterangan} />
        </div>
      </div>

      <!-- Detail lines -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <span class="text-xs font-medium text-muted-foreground">Detail Jurnal</span>
          <Button variant="outline" size="sm" onclick={addRow}><Plus size={13} /> Tambah Baris</Button>
        </div>
        <div class="rounded-xl border border-border overflow-hidden">
          <table class="w-full text-sm">
            <thead>
              <tr class="bg-muted/50 border-b border-border">
                <th class="px-3 py-2 text-left text-xs text-muted-foreground font-medium w-8">#</th>
                <th class="px-3 py-2 text-left text-xs text-muted-foreground font-medium">Akun *</th>
                <th class="px-3 py-2 text-left text-xs text-muted-foreground font-medium">Keterangan</th>
                <th class="px-3 py-2 text-right text-xs text-muted-foreground font-medium w-32">Debit</th>
                <th class="px-3 py-2 text-right text-xs text-muted-foreground font-medium w-32">Kredit</th>
                <th class="w-8"></th>
              </tr>
            </thead>
            <tbody>
              {#each form.detail as row, i}
                <tr class="border-b border-border last:border-0">
                  <td class="px-3 py-2 text-muted-foreground text-xs">{i + 1}</td>
                  <td class="px-3 py-2">
                    <!-- Native select styled to match shadcn input for inline table use -->
                    <select
                      class="h-8 w-full rounded-lg border border-input bg-transparent pl-2.5 pr-2 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                      bind:value={form.detail[i].akun_id}
                    >
                      <option value={0}>-- Pilih Akun --</option>
                      {#each akun.filter(x => !x.header && x.aktif) as a}
                        <option value={a.id}>{a.kode} - {a.nama}</option>
                      {/each}
                    </select>
                  </td>
                  <td class="px-3 py-2">
                    <Input class="text-xs py-1 h-8" placeholder="Keterangan..." bind:value={form.detail[i].keterangan} />
                  </td>
                  <td class="px-3 py-2">
                    <Input class="text-xs py-1 h-8 text-right" type="number" placeholder="0"
                      bind:value={form.detail[i].debit}
                      oninput={() => onDebitChange(i)} />
                  </td>
                  <td class="px-3 py-2">
                    <Input class="text-xs py-1 h-8 text-right" type="number" placeholder="0"
                      bind:value={form.detail[i].kredit}
                      oninput={() => onKreditChange(i)} />
                  </td>
                  <td class="px-3 py-2">
                    <button class="p-1 text-muted-foreground/40 hover:text-destructive transition-colors" onclick={() => removeRow(i)}>
                      <X size={14} />
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
            <tfoot>
              <tr class="bg-muted/50 border-t border-border">
                <td colspan="3" class="px-3 py-2 text-xs font-semibold text-muted-foreground">Total</td>
                <td class="px-3 py-2 text-right text-sm font-bold text-foreground">{formatCurrency(totalDebit)}</td>
                <td class="px-3 py-2 text-right text-sm font-bold text-foreground">{formatCurrency(totalKredit)}</td>
                <td></td>
              </tr>
            </tfoot>
          </table>
        </div>
        {#if !balanced && (totalDebit > 0 || totalKredit > 0)}
          <p class="text-xs text-destructive mt-1.5">
            Selisih: {formatCurrency(Math.abs(totalDebit - totalKredit))} — Debit dan Kredit harus seimbang
          </p>
        {:else if balanced && totalDebit > 0}
          <p class="text-xs text-green-600 mt-1.5">✓ Jurnal seimbang</p>
        {/if}
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <Button variant="outline" onclick={() => showModal = false}>Batal</Button>
    <Button onclick={save} disabled={saving || !balanced}>
      {saving ? 'Menyimpan...' : 'Simpan Jurnal'}
    </Button>
  {/snippet}
</Modal>

<ConfirmDialog open={showConfirm} message="Hapus jurnal ini? Saldo akun akan dikembalikan." onConfirm={doDelete} onCancel={() => showConfirm = false} loading={deleting} />
