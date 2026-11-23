<script lang="ts">
  import { onMount } from 'svelte';
  import { Scale, Printer } from 'lucide-svelte';
  import { callCommand, formatCurrency, namaPerusahaan } from '$lib/stores/app.svelte';
  import type { NeracaItem } from '$lib/types';
  import { Button } from '$lib/components/ui/button/index.js';
  import * as Card from '$lib/components/ui/card/index.js';

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
  <div class="flex items-center justify-between mb-6">
    <div>
      <p class="text-sm font-medium text-muted-foreground">{namaPerusahaan()}</p>
      <h1 class="text-xl font-semibold text-foreground">Neraca</h1>
      <p class="text-sm text-muted-foreground mt-0.5">Balance Sheet — Per hari ini</p>
    </div>
    <Button variant="outline" onclick={() => window.print()}>
      <Printer size={16} /> Cetak
    </Button>
  </div>

  {#if loading}
    <div class="flex items-center justify-center h-48">
      <div class="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
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
      <Card.Root class="overflow-hidden">
        <div class="px-5 py-4 border-b border-border bg-blue-50">
          <h3 class="text-sm font-bold text-blue-900">ASET</h3>
        </div>

        {#if aset.filter(i => i.sub_tipe === 'Lancar').length > 0}
          <div class="px-5 py-2 bg-muted/30 border-b border-border">
            <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Aset Lancar</span>
          </div>
          {#each aset.filter(i => i.sub_tipe === 'Lancar') as item}
            <div class="flex justify-between items-center px-5 py-2.5 border-b border-border hover:bg-muted/30">
              <div>
                <span class="text-xs text-muted-foreground font-mono mr-2">{item.kode}</span>
                <span class="text-sm text-foreground">{item.nama}</span>
              </div>
              <span class="text-sm font-medium {item.saldo < 0 ? 'text-red-600' : 'text-foreground'}">{formatCurrency(item.saldo)}</span>
            </div>
          {/each}
          <div class="flex justify-between items-center px-5 py-2.5 border-b border-border bg-blue-50/50">
            <span class="text-xs font-semibold text-muted-foreground">Jumlah Aset Lancar</span>
            <span class="text-sm font-bold text-blue-700">{formatCurrency(aset.filter(i => i.sub_tipe === 'Lancar').reduce((s,i) => s+i.saldo,0))}</span>
          </div>
        {/if}

        {#if aset.filter(i => i.sub_tipe === 'Tetap').length > 0}
          <div class="px-5 py-2 bg-muted/30 border-b border-border">
            <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Aset Tetap</span>
          </div>
          {#each aset.filter(i => i.sub_tipe === 'Tetap') as item}
            <div class="flex justify-between items-center px-5 py-2.5 border-b border-border hover:bg-muted/30">
              <div>
                <span class="text-xs text-muted-foreground font-mono mr-2">{item.kode}</span>
                <span class="text-sm text-foreground">{item.nama}</span>
              </div>
              <span class="text-sm font-medium {item.saldo < 0 ? 'text-red-600' : 'text-foreground'}">{formatCurrency(item.saldo)}</span>
            </div>
          {/each}
          <div class="flex justify-between items-center px-5 py-2.5 border-b border-border bg-blue-50/50">
            <span class="text-xs font-semibold text-muted-foreground">Jumlah Aset Tetap</span>
            <span class="text-sm font-bold text-blue-700">{formatCurrency(aset.filter(i => i.sub_tipe === 'Tetap').reduce((s,i) => s+i.saldo,0))}</span>
          </div>
        {/if}

        <div class="flex justify-between items-center px-5 py-3 bg-blue-100">
          <span class="text-sm font-bold text-blue-900">TOTAL ASET</span>
          <span class="text-base font-bold text-blue-900">{formatCurrency(totalAset)}</span>
        </div>
      </Card.Root>

      <!-- KEWAJIBAN + MODAL -->
      <div class="space-y-4">
        <!-- Kewajiban -->
        <Card.Root class="overflow-hidden">
          <div class="px-5 py-4 border-b border-border bg-red-50">
            <h3 class="text-sm font-bold text-red-900">KEWAJIBAN</h3>
          </div>
          {#each kewajiban as item}
            <div class="flex justify-between items-center px-5 py-2.5 border-b border-border hover:bg-muted/30">
              <div>
                <span class="text-xs text-muted-foreground font-mono mr-2">{item.kode}</span>
                <span class="text-sm text-foreground">{item.nama}</span>
              </div>
              <span class="text-sm font-medium text-foreground">{formatCurrency(item.saldo)}</span>
            </div>
          {:else}
            <div class="px-5 py-4 text-sm text-muted-foreground text-center">Tidak ada kewajiban</div>
          {/each}
          <div class="flex justify-between items-center px-5 py-3 bg-red-50">
            <span class="text-sm font-bold text-red-900">Total Kewajiban</span>
            <span class="text-sm font-bold text-red-700">{formatCurrency(totalKewajiban)}</span>
          </div>
        </Card.Root>

        <!-- Modal -->
        <Card.Root class="overflow-hidden">
          <div class="px-5 py-4 border-b border-border bg-green-50">
            <h3 class="text-sm font-bold text-green-900">MODAL</h3>
          </div>
          {#each modal as item}
            <div class="flex justify-between items-center px-5 py-2.5 border-b border-border hover:bg-muted/30">
              <div>
                <span class="text-xs text-muted-foreground font-mono mr-2">{item.kode}</span>
                <span class="text-sm text-foreground">{item.nama}</span>
              </div>
              <span class="text-sm font-medium text-foreground">{formatCurrency(item.saldo)}</span>
            </div>
          {:else}
            <div class="px-5 py-4 text-sm text-muted-foreground text-center">Tidak ada modal</div>
          {/each}
          <div class="flex justify-between items-center px-5 py-3 bg-green-50">
            <span class="text-sm font-bold text-green-900">Total Modal</span>
            <span class="text-sm font-bold text-green-700">{formatCurrency(totalModal)}</span>
          </div>
        </Card.Root>

        <Card.Root class="px-5 py-3 flex justify-between items-center bg-gray-900">
          <span class="text-sm font-bold text-white">TOTAL KEWAJIBAN + MODAL</span>
          <span class="text-base font-bold text-white">{formatCurrency(totalKewMod)}</span>
        </Card.Root>
      </div>
    </div>
  {/if}
</div>
