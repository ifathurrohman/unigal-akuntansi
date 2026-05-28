use crate::db::Database;
use crate::models::{Akun, AkunInput};
use tauri::State;

#[tauri::command]
pub fn get_akun(db: State<Database>) -> Result<Vec<Akun>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id,kode,nama,tipe,sub_tipe,saldo_normal,saldo,aktif,created_at FROM akun ORDER BY kode")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Akun {
                id: row.get(0)?,
                kode: row.get(1)?,
                nama: row.get(2)?,
                tipe: row.get(3)?,
                sub_tipe: row.get(4)?,
                saldo_normal: row.get(5)?,
                saldo: row.get(6)?,
                aktif: row.get::<_, i64>(7)? == 1,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn get_akun_kas_bank(db: State<Database>) -> Result<Vec<Akun>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id,kode,nama,tipe,sub_tipe,saldo_normal,saldo,aktif,created_at FROM akun WHERE tipe='Aset' AND sub_tipe='Lancar' AND (nama LIKE '%Kas%' OR nama LIKE '%Bank%') ORDER BY kode")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Akun {
                id: row.get(0)?,
                kode: row.get(1)?,
                nama: row.get(2)?,
                tipe: row.get(3)?,
                sub_tipe: row.get(4)?,
                saldo_normal: row.get(5)?,
                saldo: row.get(6)?,
                aktif: row.get::<_, i64>(7)? == 1,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn create_akun(db: State<Database>, input: AkunInput) -> Result<Akun, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO akun (kode,nama,tipe,sub_tipe,saldo_normal,saldo) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![
            input.kode,
            input.nama,
            input.tipe,
            input.sub_tipe,
            input.saldo_normal,
            input.saldo.unwrap_or(0.0)
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let akun = conn
        .query_row(
            "SELECT id,kode,nama,tipe,sub_tipe,saldo_normal,saldo,aktif,created_at FROM akun WHERE id=?1",
            rusqlite::params![id],
            |row| Ok(Akun {
                id: row.get(0)?,
                kode: row.get(1)?,
                nama: row.get(2)?,
                tipe: row.get(3)?,
                sub_tipe: row.get(4)?,
                saldo_normal: row.get(5)?,
                saldo: row.get(6)?,
                aktif: row.get::<_, i64>(7)? == 1,
                created_at: row.get(8)?,
            }),
        )
        .map_err(|e| e.to_string())?;
    Ok(akun)
}

#[tauri::command]
pub fn update_akun(db: State<Database>, id: i64, input: AkunInput) -> Result<Akun, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE akun SET kode=?1,nama=?2,tipe=?3,sub_tipe=?4,saldo_normal=?5 WHERE id=?6",
        rusqlite::params![input.kode, input.nama, input.tipe, input.sub_tipe, input.saldo_normal, id],
    )
    .map_err(|e| e.to_string())?;

    let akun = conn
        .query_row(
            "SELECT id,kode,nama,tipe,sub_tipe,saldo_normal,saldo,aktif,created_at FROM akun WHERE id=?1",
            rusqlite::params![id],
            |row| Ok(Akun {
                id: row.get(0)?,
                kode: row.get(1)?,
                nama: row.get(2)?,
                tipe: row.get(3)?,
                sub_tipe: row.get(4)?,
                saldo_normal: row.get(5)?,
                saldo: row.get(6)?,
                aktif: row.get::<_, i64>(7)? == 1,
                created_at: row.get(8)?,
            }),
        )
        .map_err(|e| e.to_string())?;
    Ok(akun)
}

#[tauri::command]
pub fn delete_akun(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM akun WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
