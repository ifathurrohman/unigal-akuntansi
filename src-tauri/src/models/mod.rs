use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Akun {
    pub id: i64,
    pub kode: String,
    pub nama: String,
    pub tipe: String,
    pub sub_tipe: Option<String>,
    pub saldo_normal: String,
    pub saldo: f64,
    pub aktif: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AkunInput {
    pub kode: String,
    pub nama: String,
    pub tipe: String,
    pub sub_tipe: Option<String>,
    pub saldo_normal: String,
    pub saldo: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pelanggan {
    pub id: i64,
    pub kode: String,
    pub nama: String,
    pub telepon: Option<String>,
    pub email: Option<String>,
    pub alamat: Option<String>,
    pub piutang: f64,
    pub aktif: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PelangganInput {
    pub kode: String,
    pub nama: String,
    pub telepon: Option<String>,
    pub email: Option<String>,
    pub alamat: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Supplier {
    pub id: i64,
    pub kode: String,
    pub nama: String,
    pub telepon: Option<String>,
    pub email: Option<String>,
    pub alamat: Option<String>,
    pub hutang: f64,
    pub aktif: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplierInput {
    pub kode: String,
    pub nama: String,
    pub telepon: Option<String>,
    pub email: Option<String>,
    pub alamat: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Produk {
    pub id: i64,
    pub kode: String,
    pub nama: String,
    pub satuan: String,
    pub harga_beli: f64,
    pub harga_jual: f64,
    pub stok: f64,
    pub aktif: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProdukInput {
    pub kode: String,
    pub nama: String,
    pub satuan: String,
    pub harga_beli: f64,
    pub harga_jual: f64,
    pub stok: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JurnalDetail {
    pub id: i64,
    pub jurnal_id: i64,
    pub akun_id: i64,
    pub akun_kode: Option<String>,
    pub akun_nama: Option<String>,
    pub keterangan: Option<String>,
    pub debit: f64,
    pub kredit: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jurnal {
    pub id: i64,
    pub nomor: String,
    pub tanggal: String,
    pub keterangan: Option<String>,
    pub tipe: String,
    pub total_debit: f64,
    pub total_kredit: f64,
    pub created_at: String,
    pub detail: Vec<JurnalDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JurnalDetailInput {
    pub akun_id: i64,
    pub keterangan: Option<String>,
    pub debit: f64,
    pub kredit: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JurnalInput {
    pub tanggal: String,
    pub keterangan: Option<String>,
    pub tipe: Option<String>,
    pub detail: Vec<JurnalDetailInput>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KasBank {
    pub id: i64,
    pub nomor: String,
    pub tanggal: String,
    pub keterangan: Option<String>,
    pub tipe: String,
    pub akun_id: i64,
    pub akun_nama: Option<String>,
    pub akun_lawan_id: Option<i64>,
    pub akun_lawan_nama: Option<String>,
    pub jumlah: f64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KasBankInput {
    pub tanggal: String,
    pub keterangan: Option<String>,
    pub tipe: String,
    pub akun_id: i64,
    pub akun_lawan_id: Option<i64>,
    pub jumlah: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_kas: f64,
    pub total_piutang: f64,
    pub total_hutang: f64,
    pub total_pendapatan: f64,
    pub total_beban: f64,
    pub laba_rugi: f64,
    pub total_aset: f64,
    pub jumlah_pelanggan: i64,
    pub jumlah_supplier: i64,
    pub jumlah_produk: i64,
    pub jurnal_bulan_ini: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabaRugiItem {
    pub kode: String,
    pub nama: String,
    pub tipe: String,
    pub saldo: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NeracaItem {
    pub kode: String,
    pub nama: String,
    pub tipe: String,
    pub sub_tipe: Option<String>,
    pub saldo: f64,
}
