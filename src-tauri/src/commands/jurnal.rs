use crate::commands::pengaturan::pastikan_periode_aktif;
use crate::db::Database;
use crate::models::{Jurnal, JurnalDetail, JurnalDetailInput, JurnalInput};
use tauri::State;

fn prefix_for(tipe: &str) -> &'static str {
    match tipe {
        "Kas" => "KAS",
        "Bank" => "BNK",
        _ => "JU",
    }
}

/// Nomor berikutnya berdasarkan bulan dari tanggal transaksi dan nomor urut
/// tertinggi yang ada (bukan COUNT, agar tidak bentrok setelah ada yang dihapus).
fn generate_nomor(conn: &rusqlite::Connection, tipe: &str, tanggal: &str) -> Result<String, String> {
    let prefix = prefix_for(tipe);
    let ym = tanggal[..7].replace('-', "");
    let max: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(CAST(substr(nomor, -4) AS INTEGER)), 0) FROM jurnal WHERE nomor LIKE ?1",
            rusqlite::params![format!("{}-{}-%", prefix, ym)],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(format!("{}-{}-{:04}", prefix, ym, max + 1))
}

fn validate(tanggal: &str, detail: &[JurnalDetailInput]) -> Result<(), String> {
    chrono::NaiveDate::parse_from_str(tanggal, "%Y-%m-%d")
        .map_err(|_| "Format tanggal tidak valid (YYYY-MM-DD)".to_string())?;
    if detail.len() < 2 {
        return Err("Jurnal minimal terdiri dari 2 baris".to_string());
    }
    for d in detail {
        if !d.debit.is_finite() || !d.kredit.is_finite() || d.debit < 0.0 || d.kredit < 0.0 {
            return Err("Nilai debit/kredit tidak valid".to_string());
        }
        if (d.debit > 0.0) == (d.kredit > 0.0) {
            return Err("Setiap baris harus berisi salah satu: debit atau kredit".to_string());
        }
    }
    let total_debit: f64 = detail.iter().map(|d| d.debit).sum();
    let total_kredit: f64 = detail.iter().map(|d| d.kredit).sum();
    if total_debit <= 0.0 {
        return Err("Total jurnal harus lebih dari nol".to_string());
    }
    if (total_debit - total_kredit).abs() > 0.01 {
        return Err("Total debit harus sama dengan total kredit".to_string());
    }
    Ok(())
}

/// Menyimpan jurnal + detail + update saldo akun. Harus dipanggil di dalam transaksi.
pub fn insert_jurnal(
    conn: &rusqlite::Connection,
    tanggal: &str,
    keterangan: Option<&str>,
    tipe: &str,
    detail: &[JurnalDetailInput],
) -> Result<i64, String> {
    validate(tanggal, detail)?;
    pastikan_periode_aktif(conn, tanggal)?;
    for d in detail {
        let (header, aktif): (i64, i64) = conn
            .query_row("SELECT header, aktif FROM akun WHERE id=?1", rusqlite::params![d.akun_id], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .map_err(|_| "Akun tidak ditemukan".to_string())?;
        if header == 1 {
            return Err("Akun induk tidak dapat dipakai dalam transaksi".to_string());
        }
        if aktif != 1 {
            return Err("Akun nonaktif tidak dapat dipakai dalam transaksi".to_string());
        }
    }
    let nomor = generate_nomor(conn, tipe, tanggal)?;
    conn.execute(
        "INSERT INTO jurnal (nomor,tanggal,keterangan,tipe) VALUES (?1,?2,?3,?4)",
        rusqlite::params![nomor, tanggal, keterangan, tipe],
    )
    .map_err(|e| e.to_string())?;
    let jurnal_id = conn.last_insert_rowid();

    for d in detail {
        conn.execute(
            "INSERT INTO jurnal_detail (jurnal_id,akun_id,keterangan,debit,kredit) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![jurnal_id, d.akun_id, d.keterangan, d.debit, d.kredit],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE akun SET saldo = saldo + ?1 - ?2 WHERE id = ?3",
            rusqlite::params![d.debit, d.kredit, d.akun_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(jurnal_id)
}

/// Membalik saldo akun lalu menghapus jurnal. Harus dipanggil di dalam transaksi.
pub fn remove_jurnal(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    let (tanggal, tipe): (String, String) = conn
        .query_row("SELECT tanggal, tipe FROM jurnal WHERE id=?1", rusqlite::params![id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .map_err(|e| e.to_string())?;
    if tipe == "Penutup" {
        return Err("Jurnal penutup tidak dapat dihapus".to_string());
    }
    pastikan_periode_aktif(conn, &tanggal)
        .map_err(|e| format!("Transaksi pada periode tertutup tidak dapat dihapus. {}", e))?;
    conn.execute(
        "UPDATE akun SET saldo = saldo - COALESCE((SELECT SUM(debit - kredit) FROM jurnal_detail WHERE jurnal_id=?1 AND akun_id=akun.id), 0)
         WHERE id IN (SELECT akun_id FROM jurnal_detail WHERE jurnal_id=?1)",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM jurnal WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn fetch_jurnal(conn: &rusqlite::Connection, id: i64) -> Result<Jurnal, String> {
    let jurnal = conn
        .query_row(
            "SELECT id,nomor,tanggal,keterangan,tipe,created_at FROM jurnal WHERE id=?1",
            rusqlite::params![id],
            |row| Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            )),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT jd.id,jd.jurnal_id,jd.akun_id,a.kode,a.nama,jd.keterangan,jd.debit,jd.kredit
             FROM jurnal_detail jd JOIN akun a ON a.id=jd.akun_id WHERE jd.jurnal_id=?1 ORDER BY jd.id",
        )
        .map_err(|e| e.to_string())?;
    let detail: Vec<JurnalDetail> = stmt
        .query_map(rusqlite::params![id], |row| {
            Ok(JurnalDetail {
                id: row.get(0)?,
                jurnal_id: row.get(1)?,
                akun_id: row.get(2)?,
                akun_kode: row.get(3)?,
                akun_nama: row.get(4)?,
                keterangan: row.get(5)?,
                debit: row.get(6)?,
                kredit: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let total_debit: f64 = detail.iter().map(|d| d.debit).sum();
    let total_kredit: f64 = detail.iter().map(|d| d.kredit).sum();

    Ok(Jurnal {
        id: jurnal.0,
        nomor: jurnal.1,
        tanggal: jurnal.2,
        keterangan: jurnal.3,
        tipe: jurnal.4,
        total_debit,
        total_kredit,
        created_at: jurnal.5,
        detail,
    })
}

#[tauri::command]
pub fn get_jurnal(db: State<Database>) -> Result<Vec<Jurnal>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id FROM jurnal ORDER BY tanggal DESC, id DESC")
        .map_err(|e| e.to_string())?;
    let ids: Vec<i64> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    ids.iter()
        .map(|&id| fetch_jurnal(&conn, id))
        .collect()
}

#[tauri::command]
pub fn create_jurnal(db: State<Database>, input: JurnalInput) -> Result<Jurnal, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let tipe = input.tipe.as_deref().unwrap_or("Umum");
    let jurnal_id = insert_jurnal(&tx, &input.tanggal, input.keterangan.as_deref(), tipe, &input.detail)?;
    tx.commit().map_err(|e| e.to_string())?;
    fetch_jurnal(&conn, jurnal_id)
}

#[tauri::command]
pub fn delete_jurnal(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let dipakai: i64 = conn
        .query_row("SELECT COUNT(*) FROM kas_bank WHERE jurnal_id=?1", rusqlite::params![id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if dipakai > 0 {
        return Err("Jurnal ini dibuat dari transaksi Kas & Bank. Hapus transaksinya di menu Kas & Bank.".to_string());
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    remove_jurnal(&tx, id)?;
    tx.commit().map_err(|e| e.to_string())
}
