<script lang="ts">
  import { onMount } from 'svelte';
  import { Scale, Printer } from 'lucide-svelte';
  import { callCommand, formatCurrency } from '$lib/stores/app.svelte';
  import type { NeracaItem } from '$lib/types';

  let items = $state<NeracaItem[]>([]);
  let loading = $state(true);

  const aset = $derived(items.filter(i => i.tipe === 'Aset'));
  const kewajiban = $derived(items.filter(i => i.tipe === 'Kewajiban'));
  const modal = $derived(items.filter(i => i.tipe === 'Modal'));

  const totalAset = $derived(aset.reduce((s, i) => s + i.saldo, 0));
  const totalKewajiban = $derived(kewajiban.reduce((s, i) => s + i.saldo, 0));
  const totalModal = $derived(modal.reduce((s, i) => s + i.saldo, 0));
  const totalKewMod = $derived(totalKewajiban + totalModal);
  const balanced = $derived(Math.abs(totalAset - totalKewMod) < 1);

  async function load() {
    loading = true;
    try { items = await callCommand<NeracaItem[]>('get_neraca'); }
    finally { loading = false; }
  }

  onMount(load);
</script>

<div class="p-6">
  <div class="page-header">
    <div>
      <h1 class="page-title">Neraca</h1>
      <p class="page-subtitle">Balance Sheet — Per hari ini</p>
    </div>
    <button class="btn-secondary" onclick={() => window.print()}>
      <Printer size={16} /> Cetak
    </button>
  </div>

  {#if loading}
    <div class="flex items-center justify-center h-48">
      <div class="w-8 h-8 border-2 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
    </div>
  {:else}
    <!-- Balance indicator -->
    <div class="mb-4 flex items-center gap-2 px-4 py-2.5 rounded-xl w-fit
      {balanced ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'}">
      <Scale size={16} class={balanced ? 'text-green-600' : 'text-red-600'} />
      <span class="text-sm font-medium {balanced ? 'text-green-700' : 'text-red-700'}">
        {balanced ? 'Neraca Seimbang' : 'Neraca Tidak Seimbang — periksa data jurnal'}
      </span>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <!-- ASET -->
      <div class="card overflow-hidden">
        <div class="px-5 py-4 border-b border-gray-100 bg-blue-50">
          <h3 class="text-sm font-bold text-blue-900">ASET</h3>
        </div>

        {#if aset.filter(i => i.sub_tipe === 'Lancar').length > 0}
          <div class="px-5 py-2 bg-gray-50 border-b border-gray-100">
            <span class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Aset Lancar</span>
          </div>
          {#each aset.filter(i => i.sub_tipe === 'Lancar') as item}
            <div class="flex justify-between items-center px-5 py-2.5 border-b border-gray-50 hover:bg-gray-50">
              <div>
                <span class="text-xs text-gray-400 font-mono mr-2">{item.kode}</span>
                <span class="text-sm text-gray-700">{item.nama}</span>
              </div>
              <span class="text-sm font-medium {item.saldo < 0 ? 'text-red-600' : 'text-gray-900'}">{formatCurrency(item.saldo)}</span>
            </div>
          {/each}
          <div class="flex justify-between items-center px-5 py-2.5 border-b border-gray-200 bg-blue-50/50">
            <span class="text-xs font-semibold text-gray-600">Jumlah Aset Lancar</span>
            <span class="text-sm font-bold text-blue-700">{formatCurrency(aset.filter(i => i.sub_tipe === 'Lancar').reduce((s,i) => s+i.saldo,0))}</span>
          </div>
        {/if}

        {#if aset.filter(i => i.sub_tipe === 'Tetap').length > 0}
          <div class="px-5 py-2 bg-gray-50 border-b border-gray-100">
            <span class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Aset Tetap</span>
          </div>
          {#each aset.filter(i => i.sub_tipe === 'Tetap') as item}
            <div class="flex justify-between items-center px-5 py-2.5 border-b border-gray-50 hover:bg-gray-50">
              <div>
                <span class="text-xs text-gray-400 font-mono mr-2">{item.kode}</span>
                <span class="text-sm text-gray-700">{item.nama}</span>
              </div>
              <span class="text-sm font-medium {item.saldo < 0 ? 'text-red-600' : 'text-gray-900'}">{formatCurrency(item.saldo)}</span>
            </div>
          {/each}
          <div class="flex justify-between items-center px-5 py-2.5 border-b border-gray-200 bg-blue-50/50">
            <span class="text-xs font-semibold text-gray-600">Jumlah Aset Tetap</span>
            <span class="text-sm font-bold text-blue-700">{formatCurrency(aset.filter(i => i.sub_tipe === 'Tetap').reduce((s,i) => s+i.saldo,0))}</span>
          </div>
        {/if}

        <div class="flex justify-between items-center px-5 py-3 bg-blue-100">
          <span class="text-sm font-bold text-blue-900">TOTAL ASET</span>
          <span class="text-base font-bold text-blue-900">{formatCurrency(totalAset)}</span>
        </div>
      </div>

      <!-- KEWAJIBAN + MODAL -->
      <div class="space-y-4">
        <!-- Kewajiban -->
        <div class="card overflow-hidden">
          <div class="px-5 py-4 border-b border-gray-100 bg-red-50">
            <h3 class="text-sm font-bold text-red-900">KEWAJIBAN</h3>
          </div>
          {#each kewajiban as item}
            <div class="flex justify-between items-center px-5 py-2.5 border-b border-gray-50 hover:bg-gray-50">
              <div>
                <span class="text-xs text-gray-400 font-mono mr-2">{item.kode}</span>
                <span class="text-sm text-gray-700">{item.nama}</span>
              </div>
              <span class="text-sm font-medium text-gray-900">{formatCurrency(item.saldo)}</span>
            </div>
          {:else}
            <div class="px-5 py-4 text-sm text-gray-400 text-center">Tidak ada kewajiban</div>
          {/each}
          <div class="flex justify-between items-center px-5 py-3 bg-red-50">
            <span class="text-sm font-bold text-red-900">Total Kewajiban</span>
            <span class="text-sm font-bold text-red-700">{formatCurrency(totalKewajiban)}</span>
          </div>
        </div>

        <!-- Modal -->
        <div class="card overflow-hidden">
          <div class="px-5 py-4 border-b border-gray-100 bg-green-50">
            <h3 class="text-sm font-bold text-green-900">MODAL</h3>
          </div>
          {#each modal as item}
            <div class="flex justify-between items-center px-5 py-2.5 border-b border-gray-50 hover:bg-gray-50">
              <div>
                <span class="text-xs text-gray-400 font-mono mr-2">{item.kode}</span>
                <span class="text-sm text-gray-700">{item.nama}</span>
              </div>
              <span class="text-sm font-medium text-gray-900">{formatCurrency(item.saldo)}</span>
            </div>
          {:else}
            <div class="px-5 py-4 text-sm text-gray-400 text-center">Tidak ada modal</div>
          {/each}
          <div class="flex justify-between items-center px-5 py-3 bg-green-50">
            <span class="text-sm font-bold text-green-900">Total Modal</span>
            <span class="text-sm font-bold text-green-700">{formatCurrency(totalModal)}</span>
          </div>
        </div>

        <div class="card px-5 py-3 flex justify-between items-center bg-gray-900">
          <span class="text-sm font-bold text-white">TOTAL KEWAJIBAN + MODAL</span>
          <span class="text-base font-bold text-white">{formatCurrency(totalKewMod)}</span>
        </div>
      </div>
    </div>
  {/if}
</div>
