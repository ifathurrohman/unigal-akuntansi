<script lang="ts">
  import { onMount } from 'svelte';
  import { BarChart3, TrendingUp, TrendingDown, Printer, FileSpreadsheet, FileText } from 'lucide-svelte';
  import { callCommand, formatCurrency, appState, namaPerusahaan } from '$lib/stores/app.svelte';
  import type { LabaRugiItem } from '$lib/types';
  import { Button } from '$lib/components/ui/button/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import * as Card from '$lib/components/ui/card/index.js';
  import * as Separator from '$lib/components/ui/separator/index.js';

  let items = $state<LabaRugiItem[]>([]);
  let loading = $state(true);
  let exporting = $state<'excel' | 'pdf' | null>(null);

  const now = new Date(appState.tanggalKerja + 'T00:00:00');
  let tahun = $state(now.getFullYear());
  let bulan = $state<number | ''>(now.getMonth() + 1);

  const pendapatan = $derived(items.filter(i => i.tipe === 'Pendapatan'));
  const beban = $derived(items.filter(i => i.tipe === 'Beban'));
  const totalPendapatan = $derived(pendapatan.reduce((s, i) => s + i.saldo, 0));
  const totalBeban = $derived(beban.reduce((s, i) => s + i.saldo, 0));
  const laba = $derived(totalPendapatan - totalBeban);

  const bulanNama = ['Januari','Februari','Maret','April','Mei','Juni','Juli','Agustus','September','Oktober','November','Desember'];

  // Range tahun: 5 tahun ke belakang sampai 2 tahun ke depan
  const tahunOptions = Array.from({ length: 8 }, (_, i) => now.getFullYear() - 5 + i);

  const periodeLabel = $derived(
    bulan ? `${bulanNama[Number(bulan) - 1]} ${tahun}` : `Tahun ${tahun}`
  );

  async function load() {
    loading = true;
    try {
      items = await callCommand<LabaRugiItem[]>('get_laba_rugi', {
        tahun,
        bulan: bulan === '' ? null : Number(bulan),
      });
    } finally { loading = false; }
  }

  async function exportExcel() {
    exporting = 'excel';
    try {
      const XLSX = await import('xlsx');
      const rows: (string | number)[][] = [
        [namaPerusahaan()],
        ['LAPORAN LABA RUGI'],
        [`Periode: ${periodeLabel}`],
        [''],
        ['PENDAPATAN'],
        ['Kode', 'Nama Akun', 'Jumlah (Rp)'],
        ...pendapatan.map(i => [i.kode, i.nama, i.saldo]),
        ['', 'Total Pendapatan', totalPendapatan],
        [''],
        ['BEBAN'],
        ['Kode', 'Nama Akun', 'Jumlah (Rp)'],
        ...beban.map(i => [i.kode, i.nama, i.saldo]),
        ['', 'Total Beban', totalBeban],
        [''],
        [laba >= 0 ? 'LABA BERSIH' : 'RUGI BERSIH', '', Math.abs(laba)],
      ];

      const ws = XLSX.utils.aoa_to_sheet(rows);
      // Format kolom jumlah sebagai number
      ws['!cols'] = [{ wch: 12 }, { wch: 36 }, { wch: 20 }];

      const wb = XLSX.utils.book_new();
      XLSX.utils.book_append_sheet(wb, ws, 'Laba Rugi');
      XLSX.writeFile(wb, `laba-rugi-${tahun}${bulan ? '-' + String(bulan).padStart(2,'0') : ''}.xlsx`);
    } finally { exporting = null; }
  }

  async function exportPdf() {
    exporting = 'pdf';
    try {
      const { default: jsPDF } = await import('jspdf');
      const { default: autoTable } = await import('jspdf-autotable');

      const doc = new jsPDF({ orientation: 'portrait', unit: 'mm', format: 'a4' });
      const pageW = doc.internal.pageSize.getWidth();

      // Header
      doc.setFontSize(14);
      doc.setFont('helvetica', 'bold');
      doc.text(namaPerusahaan(), pageW / 2, 14, { align: 'center' });
      doc.text('LAPORAN LABA RUGI', pageW / 2, 21, { align: 'center' });
      doc.setFontSize(10);
      doc.setFont('helvetica', 'normal');
      doc.text(`Periode: ${periodeLabel}`, pageW / 2, 28, { align: 'center' });

      let y = 38;

      // Pendapatan table
      autoTable(doc, {
        startY: y,
        head: [['Kode', 'Nama Akun', 'Jumlah (Rp)']],
        body: [
          ...pendapatan.map(i => [i.kode, i.nama, formatCurrency(i.saldo)]),
          [{ content: 'Total Pendapatan', colSpan: 2, styles: { fontStyle: 'bold' } }, { content: formatCurrency(totalPendapatan), styles: { fontStyle: 'bold' } }],
        ],
        headStyles: { fillColor: [22, 163, 74], textColor: 255 },
        columnStyles: { 2: { halign: 'right' } },
        didDrawPage: (data) => { y = data.cursor?.y ?? y; },
      });

      y = (doc as any).lastAutoTable.finalY + 8;

      // Beban table
      autoTable(doc, {
        startY: y,
        head: [['Kode', 'Nama Akun', 'Jumlah (Rp)']],
        body: [
          ...beban.map(i => [i.kode, i.nama, formatCurrency(i.saldo)]),
          [{ content: 'Total Beban', colSpan: 2, styles: { fontStyle: 'bold' } }, { content: formatCurrency(totalBeban), styles: { fontStyle: 'bold' } }],
        ],
        headStyles: { fillColor: [220, 38, 38], textColor: 255 },
        columnStyles: { 2: { halign: 'right' } },
      });

      y = (doc as any).lastAutoTable.finalY + 8;

      // Laba/Rugi row
      autoTable(doc, {
        startY: y,
        body: [[
          { content: laba >= 0 ? 'LABA BERSIH' : 'RUGI BERSIH', styles: { fontStyle: 'bold', fontSize: 11 } },
          '',
          { content: formatCurrency(Math.abs(laba)), styles: { fontStyle: 'bold', fontSize: 11, halign: 'right' } },
        ]],
        bodyStyles: { fillColor: laba >= 0 ? [220, 252, 231] : [254, 226, 226] },
      });

      doc.save(`laba-rugi-${tahun}${bulan ? '-' + String(bulan).padStart(2,'0') : ''}.pdf`);
    } finally { exporting = null; }
  }

  function doPrint() {
    window.print();
  }

  onMount(load);
</script>

<!-- Print styles: hanya tampilkan area laporan -->
<svelte:head>
  <style>
    @media print {
      body > div > aside,
      .no-print { display: none !important; }
      body { background: white !important; }
      .print-area { padding: 0 !important; }
    }
  </style>
</svelte:head>

<div class="p-6 print-area">
  <!-- Header -->
  <div class="flex items-center justify-between mb-6 no-print">
    <div>
      <p class="text-sm font-medium text-muted-foreground">{namaPerusahaan()}</p>
      <h1 class="text-xl font-semibold text-foreground">Laporan Laba Rugi</h1>
      <p class="text-sm text-muted-foreground mt-0.5">Income Statement</p>
    </div>
    <div class="flex items-center gap-2">
      <Button variant="outline" size="sm" onclick={doPrint} class="gap-1.5">
        <Printer size={15} /> Cetak
      </Button>
      <Button variant="outline" size="sm" onclick={exportExcel} disabled={exporting === 'excel'} class="gap-1.5 text-green-700 border-green-200 hover:bg-green-50">
        <FileSpreadsheet size={15} />
        {exporting === 'excel' ? 'Mengekspor...' : 'Excel'}
      </Button>
      <Button variant="outline" size="sm" onclick={exportPdf} disabled={exporting === 'pdf'} class="gap-1.5 text-red-700 border-red-200 hover:bg-red-50">
        <FileText size={15} />
        {exporting === 'pdf' ? 'Mengekspor...' : 'PDF'}
      </Button>
    </div>
  </div>

  <!-- Filter -->
  <Card.Root class="p-4 mb-6 no-print">
    <div class="flex flex-wrap items-end gap-4">
      <div>
        <p class="text-xs font-medium text-muted-foreground mb-1.5">Tahun</p>
        <Select.Root
          value={String(tahun)}
          onValueChange={(v) => { tahun = Number(v); load(); }}
        >
          <Select.Trigger class="w-28">
            <Select.Value placeholder="Pilih tahun" />
          </Select.Trigger>
          <Select.Content>
            {#each tahunOptions as y}
              <Select.Item value={String(y)} label={String(y)}>{y}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      <div>
        <p class="text-xs font-medium text-muted-foreground mb-1.5">Bulan</p>
        <Select.Root
          value={bulan === '' ? 'all' : String(bulan)}
          onValueChange={(v) => { bulan = v === 'all' ? '' : Number(v); load(); }}
        >
          <Select.Trigger class="w-40">
            <Select.Value placeholder="Pilih bulan" />
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="all" label="Semua Bulan">Semua Bulan</Select.Item>
            {#each bulanNama as b, i}
              <Select.Item value={String(i + 1)} label={b}>{b}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      <div class="flex-1"></div>

      <div class="text-right">
        <p class="text-xs text-muted-foreground">Periode</p>
        <p class="text-sm font-semibold text-foreground">{periodeLabel}</p>
      </div>
    </div>
  </Card.Root>

  <!-- Print header (hanya muncul saat print) -->
  <div class="hidden print:block mb-6 text-center">
    <h2 class="text-lg font-bold">LAPORAN LABA RUGI</h2>
    <p class="text-sm">Periode: {periodeLabel}</p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center h-48">
      <div class="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
    </div>
  {:else}
    <!-- Summary cards -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 mb-6">
      <Card.Root class="p-5">
        <div class="flex items-center gap-2 mb-1">
          <TrendingUp size={16} class="text-green-600" />
          <span class="text-xs text-muted-foreground">Total Pendapatan</span>
        </div>
        <div class="text-xl font-bold text-green-600">{formatCurrency(totalPendapatan)}</div>
      </Card.Root>
      <Card.Root class="p-5">
        <div class="flex items-center gap-2 mb-1">
          <TrendingDown size={16} class="text-red-500" />
          <span class="text-xs text-muted-foreground">Total Beban</span>
        </div>
        <div class="text-xl font-bold text-red-500">{formatCurrency(totalBeban)}</div>
      </Card.Root>
      <Card.Root class="p-5 border-2 {laba >= 0 ? 'border-green-200 bg-green-50' : 'border-red-200 bg-red-50'}">
        <div class="flex items-center gap-2 mb-1">
          <BarChart3 size={16} class={laba >= 0 ? 'text-green-700' : 'text-red-700'} />
          <span class="text-xs text-muted-foreground">{laba >= 0 ? 'Laba Bersih' : 'Rugi Bersih'}</span>
        </div>
        <div class="text-xl font-bold {laba >= 0 ? 'text-green-700' : 'text-red-700'}">{formatCurrency(Math.abs(laba))}</div>
      </Card.Root>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <!-- Pendapatan -->
      <Card.Root class="overflow-hidden">
        <div class="px-5 py-4 border-b border-border bg-green-50">
          <h3 class="text-sm font-semibold text-green-800 flex items-center gap-2">
            <TrendingUp size={16} /> PENDAPATAN
          </h3>
        </div>
        <div class="divide-y divide-border">
          {#each pendapatan as item}
            <div class="flex items-center justify-between px-5 py-3 hover:bg-muted/30">
              <div>
                <div class="text-xs text-muted-foreground font-mono">{item.kode}</div>
                <div class="text-sm text-foreground">{item.nama}</div>
              </div>
              <div class="text-sm font-semibold text-green-600">{formatCurrency(item.saldo)}</div>
            </div>
          {:else}
            <div class="px-5 py-8 text-center text-sm text-muted-foreground">Tidak ada data pendapatan</div>
          {/each}
          <div class="flex items-center justify-between px-5 py-3 bg-green-50">
            <span class="text-sm font-bold text-green-800">Total Pendapatan</span>
            <span class="text-sm font-bold text-green-700">{formatCurrency(totalPendapatan)}</span>
          </div>
        </div>
      </Card.Root>

      <!-- Beban -->
      <Card.Root class="overflow-hidden">
        <div class="px-5 py-4 border-b border-border bg-red-50">
          <h3 class="text-sm font-semibold text-red-800 flex items-center gap-2">
            <TrendingDown size={16} /> BEBAN
          </h3>
        </div>
        <div class="divide-y divide-border">
          {#each beban as item}
            <div class="flex items-center justify-between px-5 py-3 hover:bg-muted/30">
              <div>
                <div class="text-xs text-muted-foreground font-mono">{item.kode}</div>
                <div class="text-sm text-foreground">{item.nama}</div>
              </div>
              <div class="text-sm font-semibold text-red-500">{formatCurrency(item.saldo)}</div>
            </div>
          {:else}
            <div class="px-5 py-8 text-center text-sm text-muted-foreground">Tidak ada data beban</div>
          {/each}
          <div class="flex items-center justify-between px-5 py-3 bg-red-50">
            <span class="text-sm font-bold text-red-800">Total Beban</span>
            <span class="text-sm font-bold text-red-600">{formatCurrency(totalBeban)}</span>
          </div>
        </div>
      </Card.Root>
    </div>

    <!-- Laba/Rugi total -->
    <Card.Root class="mt-4 px-5 py-4 flex items-center justify-between {laba >= 0 ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'}">
      <span class="font-bold text-foreground">{laba >= 0 ? 'LABA BERSIH' : 'RUGI BERSIH'}</span>
      <span class="text-lg font-bold {laba >= 0 ? 'text-green-700' : 'text-red-700'}">{formatCurrency(Math.abs(laba))}</span>
    </Card.Root>
  {/if}
</div>
