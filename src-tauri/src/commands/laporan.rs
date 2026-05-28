use crate::db::Database;
use crate::models::{DashboardData, LabaRugiItem, NeracaItem};
use tauri::State;

#[tauri::command]
pub fn get_dashboard(db: State<Database>) -> Result<DashboardData, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let total_kas: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE tipe='Aset' AND sub_tipe='Lancar' AND (nama LIKE '%Kas%' OR nama LIKE '%Bank%')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let total_piutang: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE nama LIKE '%Piutang%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let total_hutang: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE nama LIKE '%Hutang%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let total_pendapatan: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE tipe='Pendapatan'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let total_beban: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE tipe='Beban'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let total_aset: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE tipe='Aset' AND LENGTH(kode) > 6",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let jumlah_pelanggan: i64 = conn
        .query_row("SELECT COUNT(*) FROM pelanggan WHERE aktif=1", [], |r| r.get(0))
        .unwrap_or(0);

    let jumlah_supplier: i64 = conn
        .query_row("SELECT COUNT(*) FROM supplier WHERE aktif=1", [], |r| r.get(0))
        .unwrap_or(0);

    let jumlah_produk: i64 = conn
        .query_row("SELECT COUNT(*) FROM produk WHERE aktif=1", [], |r| r.get(0))
        .unwrap_or(0);

    let jurnal_bulan_ini: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jurnal WHERE strftime('%Y-%m', tanggal) = strftime('%Y-%m', 'now')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(DashboardData {
        total_kas,
        total_piutang,
        total_hutang,
        total_pendapatan,
        total_beban,
        laba_rugi: total_pendapatan - total_beban,
        total_aset,
        jumlah_pelanggan,
        jumlah_supplier,
        jumlah_produk,
        jurnal_bulan_ini,
    })
}

#[tauri::command]
pub fn get_laba_rugi(db: State<Database>, tahun: i32, bulan: Option<i32>) -> Result<Vec<LabaRugiItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let filter = match bulan {
        Some(b) => format!("AND strftime('%Y-%m', tanggal) = '{}-{:02}'", tahun, b),
        None => format!("AND strftime('%Y', tanggal) = '{}'", tahun),
    };

    let sql = format!(
        "SELECT a.kode, a.nama, a.tipe,
         COALESCE(SUM(jd.debit - jd.kredit), 0) as saldo
         FROM akun a
         LEFT JOIN jurnal_detail jd ON jd.akun_id = a.id
         LEFT JOIN jurnal j ON j.id = jd.jurnal_id {}
         WHERE a.tipe IN ('Pendapatan', 'Beban') AND LENGTH(a.kode) > 6
         GROUP BY a.id ORDER BY a.kode",
        filter
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let tipe: String = row.get(2)?;
            let raw_saldo: f64 = row.get(3)?;
            let saldo = if tipe == "Pendapatan" { -raw_saldo } else { raw_saldo };
            Ok(LabaRugiItem {
                kode: row.get(0)?,
                nama: row.get(1)?,
                tipe,
                saldo,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn get_neraca(db: State<Database>) -> Result<Vec<NeracaItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT kode,nama,tipe,sub_tipe,saldo FROM akun WHERE tipe IN ('Aset','Kewajiban','Modal') AND LENGTH(kode) > 6 ORDER BY kode",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(NeracaItem {
                kode: row.get(0)?,
                nama: row.get(1)?,
                tipe: row.get(2)?,
                sub_tipe: row.get(3)?,
                saldo: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}
