use crate::db::Database;
use crate::models::{Akun, AkunInput};
use tauri::State;

const SELECT_AKUN: &str =
    "SELECT id,kode,nama,tipe,sub_tipe,saldo_normal,saldo,aktif,header,kas_bank,created_at FROM akun";

fn map_akun(row: &rusqlite::Row) -> rusqlite::Result<Akun> {
    Ok(Akun {
        id: row.get(0)?,
        kode: row.get(1)?,
        nama: row.get(2)?,
        tipe: row.get(3)?,
        sub_tipe: row.get(4)?,
        saldo_normal: row.get(5)?,
        saldo: row.get(6)?,
        aktif: row.get::<_, i64>(7)? == 1,
        header: row.get::<_, i64>(8)? == 1,
        kas_bank: row.get::<_, i64>(9)? == 1,
        created_at: row.get(10)?,
    })
}

fn fetch_akun(conn: &rusqlite::Connection, id: i64) -> Result<Akun, String> {
    conn.query_row(&format!("{} WHERE id=?1", SELECT_AKUN), rusqlite::params![id], map_akun)
        .map_err(|e| e.to_string())
}

fn akun_dipakai(conn: &rusqlite::Connection, id: i64) -> Result<bool, String> {
    let n: i64 = conn
        .query_row(
            "SELECT (SELECT COUNT(*) FROM jurnal_detail WHERE akun_id=?1)
                  + (SELECT COUNT(*) FROM kas_bank WHERE akun_id=?1 OR akun_lawan_id=?1)
                  + (SELECT COUNT(*) FROM produk WHERE akun_persediaan_id=?1)
                  + (SELECT COUNT(*) FROM perusahaan WHERE akun_laba_ditahan_id=?1)",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(n > 0)
}

fn validate(input: &AkunInput) -> Result<(), String> {
    if input.kode.trim().is_empty() || input.nama.trim().is_empty() {
        return Err("Kode dan nama akun wajib diisi".to_string());
    }
    if input.header.unwrap_or(false) && input.kas_bank.unwrap_or(false) {
        return Err("Akun induk tidak dapat ditandai sebagai akun kas/bank".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn get_akun(db: State<Database>) -> Result<Vec<Akun>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(&format!("{} ORDER BY kode", SELECT_AKUN))
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], map_akun).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn get_akun_kas_bank(db: State<Database>) -> Result<Vec<Akun>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(&format!("{} WHERE kas_bank=1 AND header=0 AND aktif=1 ORDER BY kode", SELECT_AKUN))
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], map_akun).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

/// Saldo awal tidak bisa diisi di sini: saldo selalu berasal dari jurnal (gunakan jurnal tipe "Saldo Awal").
#[tauri::command]
pub fn create_akun(db: State<Database>, input: AkunInput) -> Result<Akun, String> {
    validate(&input)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO akun (kode,nama,tipe,sub_tipe,saldo_normal,saldo,header,kas_bank) VALUES (?1,?2,?3,?4,?5,0,?6,?7)",
        rusqlite::params![
            input.kode.trim(),
            input.nama.trim(),
            input.tipe,
            input.sub_tipe,
            input.saldo_normal,
            input.header.unwrap_or(false) as i64,
            input.kas_bank.unwrap_or(false) as i64
        ],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") { "Kode akun sudah digunakan".to_string() } else { e.to_string() }
    })?;
    fetch_akun(&conn, conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_akun(db: State<Database>, id: i64, input: AkunInput) -> Result<Akun, String> {
    validate(&input)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let (tipe, normal, header): (String, String, i64) = conn
        .query_row("SELECT tipe,saldo_normal,header FROM akun WHERE id=?1", rusqlite::params![id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(|e| e.to_string())?;
    let header_baru = input.header.unwrap_or(false);
    if (tipe != input.tipe || normal != input.saldo_normal || (header == 1) != header_baru)
        && akun_dipakai(&conn, id)?
    {
        return Err("Tipe, saldo normal, dan status induk tidak dapat diubah karena akun sudah dipakai".to_string());
    }
    conn.execute(
        "UPDATE akun SET kode=?1,nama=?2,tipe=?3,sub_tipe=?4,saldo_normal=?5,header=?6,kas_bank=?7 WHERE id=?8",
        rusqlite::params![
            input.kode.trim(), input.nama.trim(), input.tipe, input.sub_tipe, input.saldo_normal,
            header_baru as i64, input.kas_bank.unwrap_or(false) as i64, id
        ],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") { "Kode akun sudah digunakan".to_string() } else { e.to_string() }
    })?;
    fetch_akun(&conn, id)
}

#[tauri::command]
pub fn delete_akun(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    if akun_dipakai(&conn, id)? {
        return Err("Akun tidak dapat dihapus karena sudah dipakai dalam transaksi atau pengaturan".to_string());
    }
    conn.execute("DELETE FROM akun WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
