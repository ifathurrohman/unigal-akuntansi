<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Search, Trash2, ChevronDown, ChevronRight, X } from 'lucide-svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { callCommand, showNotification, formatCurrency, formatDate } from '$lib/stores/app.svelte';
  import type { Jurnal, JurnalInput, JurnalDetailInput, Akun } from '$lib/types';

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

  const today = new Date().toISOString().slice(0, 10);

  let form = $state<JurnalInput>({
    tanggal: today,
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
      tanggal: today,
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
    if (Number(form.detail[i].debit) > 0) {
      form.detail[i].kredit = 0;
    }
  }

  function onKreditChange(i: number) {
    if (Number(form.detail[i].kredit) > 0) {
      form.detail[i].debit = 0;
    }
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
  <div class="page-header">
    <div>
      <h1 class="page-title">Jurnal Umum</h1>
      <p class="page-subtitle">{list.length} entri jurnal</p>
    </div>
    <button class="btn-primary" onclick={openCreate}><Plus size={16} /> Buat Jurnal</button>
  </div>

  <div class="flex items-center gap-3 mb-4">
    <div class="relative flex-1 max-w-sm">
      <Search size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
      <input class="input pl-9" placeholder="Cari nomor atau keterangan..." bind:value={search} />
    </div>
  </div>

  <div class="space-y-2">
    {#if loading}
      <div class="card p-8 text-center text-gray-400">Memuat data...</div>
    {:else if filtered.length === 0}
      <div class="card p-8 text-center text-gray-400">Tidak ada jurnal</div>
    {:else}
      {#each filtered as j}
        <div class="card overflow-hidden">
          <!-- Header row -->
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
          <div
            class="w-full flex items-center gap-4 px-4 py-3 hover:bg-gray-50 transition-colors cursor-pointer"
            role="button"
            tabindex="0"
            onclick={() => expandedId = expandedId === j.id ? null : j.id}
          >
            <div class="shrink-0 text-gray-300">
              {#if expandedId === j.id}
                <ChevronDown size={16} />
              {:else}
                <ChevronRight size={16} />
              {/if}
            </div>
            <div class="font-mono text-xs font-semibold text-primary-600 w-32 shrink-0">{j.nomor}</div>
            <div class="text-xs text-gray-500 w-24 shrink-0">{formatDate(j.tanggal)}</div>
            <div class="flex-1 text-sm text-gray-700 truncate">{j.keterangan || '-'}</div>
            <div class="text-sm font-semibold text-gray-900 w-36 text-right shrink-0">{formatCurrency(j.total_debit)}</div>
            <button
              class="btn-ghost btn-sm p-1.5 text-red-400 hover:text-red-600 hover:bg-red-50 shrink-0"
              onclick={(e) => { e.stopPropagation(); deletingId = j.id; showConfirm = true; }}
            >
              <Trash2 size={14} />
            </button>
          </div>

          <!-- Detail rows -->
          {#if expandedId === j.id}
            <div class="border-t border-gray-100 bg-gray-50/50">
              <table class="w-full text-xs">
                <thead>
                  <tr class="border-b border-gray-100">
                    <th class="px-8 py-2 text-left text-gray-400 font-medium">Akun</th>
                    <th class="px-4 py-2 text-left text-gray-400 font-medium">Keterangan</th>
                    <th class="px-4 py-2 text-right text-gray-400 font-medium">Debit</th>
                    <th class="px-4 py-2 text-right text-gray-400 font-medium">Kredit</th>
                  </tr>
                </thead>
                <tbody>
                  {#each j.detail as d}
                    <tr class="border-b border-gray-50">
                      <td class="px-8 py-2">
                        <span class="font-mono text-gray-400">{d.akun_kode}</span>
                        <span class="ml-2 text-gray-700">{d.akun_nama}</span>
                      </td>
                      <td class="px-4 py-2 text-gray-500">{d.keterangan || '-'}</td>
                      <td class="px-4 py-2 text-right font-medium {d.debit > 0 ? 'text-gray-900' : 'text-gray-300'}">
                        {d.debit > 0 ? formatCurrency(d.debit) : '-'}
                      </td>
                      <td class="px-4 py-2 text-right font-medium {d.kredit > 0 ? 'text-gray-900' : 'text-gray-300'}">
                        {d.kredit > 0 ? formatCurrency(d.kredit) : '-'}
                      </td>
                    </tr>
                  {/each}
                  <tr class="bg-gray-100/80">
                    <td colspan="2" class="px-8 py-2 font-semibold text-gray-600">Total</td>
                    <td class="px-4 py-2 text-right font-bold text-gray-900">{formatCurrency(j.total_debit)}</td>
                    <td class="px-4 py-2 text-right font-bold text-gray-900">{formatCurrency(j.total_kredit)}</td>
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
<Modal open={showModal} title="Buat Jurnal Umum" maxWidth="max-w-3xl" onClose={() => showModal = false}>
  {#snippet children()}
    <div class="space-y-4">
      <div class="grid grid-cols-3 gap-3">
        <div>
          <label class="label" for="jrn-tanggal">Tanggal *</label>
          <input id="jrn-tanggal" class="input" type="date" bind:value={form.tanggal} />
        </div>
        <div>
          <label class="label" for="jrn-tipe">Tipe</label>
          <select id="jrn-tipe" class="input" bind:value={form.tipe}>
            <option value="Umum">Jurnal Umum</option>
            <option value="Kas">Kas</option>
            <option value="Bank">Bank</option>
          </select>
        </div>
        <div>
          <label class="label" for="jrn-ket">Keterangan</label>
          <input id="jrn-ket" class="input" placeholder="Keterangan jurnal..." bind:value={form.keterangan} />
        </div>
      </div>

      <!-- Detail lines -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <span class="label mb-0">Detail Jurnal</span>
          <button class="btn-secondary btn-sm" onclick={addRow}><Plus size={13} /> Tambah Baris</button>
        </div>
        <div class="rounded-xl border border-gray-200 overflow-hidden">
          <table class="w-full text-sm">
            <thead>
              <tr class="bg-gray-50 border-b border-gray-200">
                <th class="px-3 py-2 text-left text-xs text-gray-500 font-medium w-8">#</th>
                <th class="px-3 py-2 text-left text-xs text-gray-500 font-medium">Akun *</th>
                <th class="px-3 py-2 text-left text-xs text-gray-500 font-medium">Keterangan</th>
                <th class="px-3 py-2 text-right text-xs text-gray-500 font-medium w-32">Debit</th>
                <th class="px-3 py-2 text-right text-xs text-gray-500 font-medium w-32">Kredit</th>
                <th class="w-8"></th>
              </tr>
            </thead>
            <tbody>
              {#each form.detail as row, i}
                <tr class="border-b border-gray-100 last:border-0">
                  <td class="px-3 py-2 text-gray-400 text-xs">{i + 1}</td>
                  <td class="px-3 py-2">
                    <select class="input text-xs py-1.5" bind:value={form.detail[i].akun_id}>
                      <option value={0}>-- Pilih Akun --</option>
                      {#each akun as a}
                        <option value={a.id}>{a.kode} - {a.nama}</option>
                      {/each}
                    </select>
                  </td>
                  <td class="px-3 py-2">
                    <input class="input text-xs py-1.5" placeholder="Keterangan..." bind:value={form.detail[i].keterangan} />
                  </td>
                  <td class="px-3 py-2">
                    <input class="input text-xs py-1.5 text-right" type="number" placeholder="0"
                      bind:value={form.detail[i].debit}
                      oninput={() => onDebitChange(i)} />
                  </td>
                  <td class="px-3 py-2">
                    <input class="input text-xs py-1.5 text-right" type="number" placeholder="0"
                      bind:value={form.detail[i].kredit}
                      oninput={() => onKreditChange(i)} />
                  </td>
                  <td class="px-3 py-2">
                    <button class="p-1 text-gray-300 hover:text-red-500 transition-colors" onclick={() => removeRow(i)}>
                      <X size={14} />
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
            <tfoot>
              <tr class="bg-gray-50 border-t border-gray-200">
                <td colspan="3" class="px-3 py-2 text-xs font-semibold text-gray-600">Total</td>
                <td class="px-3 py-2 text-right text-sm font-bold text-gray-900">{formatCurrency(totalDebit)}</td>
                <td class="px-3 py-2 text-right text-sm font-bold text-gray-900">{formatCurrency(totalKredit)}</td>
                <td></td>
              </tr>
            </tfoot>
          </table>
        </div>
        {#if !balanced && (totalDebit > 0 || totalKredit > 0)}
          <p class="text-xs text-red-500 mt-1.5">
            Selisih: {formatCurrency(Math.abs(totalDebit - totalKredit))} — Debit dan Kredit harus seimbang
          </p>
        {:else if balanced && totalDebit > 0}
          <p class="text-xs text-green-600 mt-1.5">✓ Jurnal seimbang</p>
        {/if}
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn-secondary" onclick={() => showModal = false}>Batal</button>
    <button class="btn-primary" onclick={save} disabled={saving || !balanced}>
      {saving ? 'Menyimpan...' : 'Simpan Jurnal'}
    </button>
  {/snippet}
</Modal>

<ConfirmDialog open={showConfirm} message="Hapus jurnal ini? Saldo akun akan dikembalikan." onConfirm={doDelete} onCancel={() => showConfirm = false} loading={deleting} />
