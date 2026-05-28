use crate::db::Database;
use crate::models::{Supplier, SupplierInput};
use tauri::State;

fn fetch_supplier(conn: &rusqlite::Connection, id: i64) -> Result<Supplier, String> {
    conn.query_row(
        "SELECT id,kode,nama,telepon,email,alamat,hutang,aktif,created_at FROM supplier WHERE id=?1",
        rusqlite::params![id],
        |row| Ok(Supplier {
            id: row.get(0)?,
            kode: row.get(1)?,
            nama: row.get(2)?,
            telepon: row.get(3)?,
            email: row.get(4)?,
            alamat: row.get(5)?,
            hutang: row.get(6)?,
            aktif: row.get::<_, i64>(7)? == 1,
            created_at: row.get(8)?,
        }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_supplier(db: State<Database>) -> Result<Vec<Supplier>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id,kode,nama,telepon,email,alamat,hutang,aktif,created_at FROM supplier WHERE aktif=1 ORDER BY nama")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Supplier {
                id: row.get(0)?,
                kode: row.get(1)?,
                nama: row.get(2)?,
                telepon: row.get(3)?,
                email: row.get(4)?,
                alamat: row.get(5)?,
                hutang: row.get(6)?,
                aktif: row.get::<_, i64>(7)? == 1,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn create_supplier(db: State<Database>, input: SupplierInput) -> Result<Supplier, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO supplier (kode,nama,telepon,email,alamat) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![input.kode, input.nama, input.telepon, input.email, input.alamat],
    )
    .map_err(|e| e.to_string())?;
    fetch_supplier(&conn, conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_supplier(db: State<Database>, id: i64, input: SupplierInput) -> Result<Supplier, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE supplier SET kode=?1,nama=?2,telepon=?3,email=?4,alamat=?5 WHERE id=?6",
        rusqlite::params![input.kode, input.nama, input.telepon, input.email, input.alamat, id],
    )
    .map_err(|e| e.to_string())?;
    fetch_supplier(&conn, id)
}

#[tauri::command]
pub fn delete_supplier(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE supplier SET aktif=0 WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
