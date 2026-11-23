use crate::commands::pengaturan::tanggal_hari_ini;
use crate::db::Database;
use crate::models::{DashboardData, LabaRugiItem, NeracaItem};
use rusqlite::Connection;
use tauri::State;

fn sum(conn: &Connection, sql: &str) -> f64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap_or(0.0)
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap_or(0)
}

#[tauri::command]
pub fn get_dashboard(db: State<Database>) -> Result<DashboardData, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Saldo disimpan sebagai debit - kredit; akun kredit dibalik agar tampil positif.
    let total_kas = sum(&conn, "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE kas_bank=1 AND header=0");
    let total_piutang = sum(&conn, "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE header=0 AND tipe='Aset' AND nama LIKE '%Piutang%'");
    let total_hutang = sum(&conn, "SELECT COALESCE(-SUM(saldo),0) FROM akun WHERE header=0 AND tipe='Kewajiban' AND nama LIKE '%Hutang%'");
    let total_pendapatan = sum(&conn, "SELECT COALESCE(-SUM(saldo),0) FROM akun WHERE header=0 AND tipe='Pendapatan'");
    let total_beban = sum(&conn, "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE header=0 AND tipe='Beban'");
    let total_aset = sum(&conn, "SELECT COALESCE(SUM(saldo),0) FROM akun WHERE header=0 AND tipe='Aset'");

    let bulan_ini = tanggal_hari_ini(&conn)[..7].to_string();
    let jurnal_bulan_ini: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jurnal WHERE substr(tanggal,1,7) = ?1 AND tipe != 'Penutup'",
            rusqlite::params![bulan_ini],
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
        jumlah_pelanggan: count(&conn, "SELECT COUNT(*) FROM pelanggan WHERE aktif=1"),
        jumlah_supplier: count(&conn, "SELECT COUNT(*) FROM supplier WHERE aktif=1"),
        jumlah_produk: count(&conn, "SELECT COUNT(*) FROM produk WHERE aktif=1"),
        jurnal_bulan_ini,
    })
}

/// Laba rugi untuk rentang tanggal (inklusif). Jurnal penutup tidak ikut dihitung.
pub fn laba_rugi_rentang(conn: &Connection, dari: &str, sampai: &str) -> Result<Vec<LabaRugiItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT a.kode, a.nama, a.tipe, COALESCE(SUM(jd.debit - jd.kredit), 0)
             FROM akun a
             LEFT JOIN (
                 SELECT d.akun_id, d.debit, d.kredit
                 FROM jurnal_detail d JOIN jurnal j ON j.id = d.jurnal_id
                 WHERE j.tanggal >= ?1 AND j.tanggal <= ?2 AND j.tipe != 'Penutup'
             ) jd ON jd.akun_id = a.id
             WHERE a.tipe IN ('Pendapatan', 'Beban') AND a.header = 0
             GROUP BY a.id ORDER BY a.kode",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![dari, sampai], |row| {
            let tipe: String = row.get(2)?;
            let raw_saldo: f64 = row.get(3)?;
            Ok(LabaRugiItem {
                kode: row.get(0)?,
                nama: row.get(1)?,
                saldo: if tipe == "Pendapatan" { -raw_saldo } else { raw_saldo },
                tipe,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn get_laba_rugi(db: State<Database>, tahun: i32, bulan: Option<i32>) -> Result<Vec<LabaRugiItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let (dari, sampai) = match bulan {
        Some(b) => (format!("{}-{:02}-01", tahun, b), format!("{}-{:02}-31", tahun, b)),
        None => (format!("{}-01-01", tahun), format!("{}-12-31", tahun)),
    };
    laba_rugi_rentang(&conn, &dari, &sampai)
}

/// Neraca per saat ini (berdasarkan saldo akun), termasuk laba berjalan yang belum ditutup.
pub fn neraca_sekarang(conn: &Connection) -> Result<Vec<NeracaItem>, String> {
    // Saldo disimpan sebagai debit - kredit; Kewajiban & Modal dibalik agar tampil positif.
    let mut stmt = conn
        .prepare(
            "SELECT kode,nama,tipe,sub_tipe, CASE WHEN tipe = 'Aset' THEN saldo ELSE -saldo END
             FROM akun WHERE tipe IN ('Aset','Kewajiban','Modal') AND header = 0 ORDER BY kode",
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
    let mut items: Vec<NeracaItem> = rows.map(|r| r.map_err(|e| e.to_string())).collect::<Result<_, _>>()?;

    let laba = sum(conn, "SELECT COALESCE(-SUM(saldo), 0) FROM akun WHERE tipe IN ('Pendapatan','Beban') AND header = 0");
    if laba.abs() > 0.005 {
        items.push(NeracaItem {
            kode: "3-9999".to_string(),
            nama: "Laba (Rugi) Periode Berjalan".to_string(),
            tipe: "Modal".to_string(),
            sub_tipe: None,
            saldo: laba,
        });
    }
    Ok(items)
}

#[tauri::command]
pub fn get_neraca(db: State<Database>) -> Result<Vec<NeracaItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    neraca_sekarang(&conn)
}
