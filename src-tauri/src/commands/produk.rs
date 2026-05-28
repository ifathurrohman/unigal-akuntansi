use crate::db::Database;
use crate::models::{Produk, ProdukInput};
use tauri::State;

fn fetch_produk(conn: &rusqlite::Connection, id: i64) -> Result<Produk, String> {
    conn.query_row(
        "SELECT id,kode,nama,satuan,harga_beli,harga_jual,stok,aktif,created_at FROM produk WHERE id=?1",
        rusqlite::params![id],
        |row| Ok(Produk {
            id: row.get(0)?,
            kode: row.get(1)?,
            nama: row.get(2)?,
            satuan: row.get(3)?,
            harga_beli: row.get(4)?,
            harga_jual: row.get(5)?,
            stok: row.get(6)?,
            aktif: row.get::<_, i64>(7)? == 1,
            created_at: row.get(8)?,
        }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_produk(db: State<Database>) -> Result<Vec<Produk>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id,kode,nama,satuan,harga_beli,harga_jual,stok,aktif,created_at FROM produk WHERE aktif=1 ORDER BY nama")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Produk {
                id: row.get(0)?,
                kode: row.get(1)?,
                nama: row.get(2)?,
                satuan: row.get(3)?,
                harga_beli: row.get(4)?,
                harga_jual: row.get(5)?,
                stok: row.get(6)?,
                aktif: row.get::<_, i64>(7)? == 1,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn create_produk(db: State<Database>, input: ProdukInput) -> Result<Produk, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO produk (kode,nama,satuan,harga_beli,harga_jual,stok) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![input.kode, input.nama, input.satuan, input.harga_beli, input.harga_jual, input.stok.unwrap_or(0.0)],
    )
    .map_err(|e| e.to_string())?;
    fetch_produk(&conn, conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_produk(db: State<Database>, id: i64, input: ProdukInput) -> Result<Produk, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE produk SET kode=?1,nama=?2,satuan=?3,harga_beli=?4,harga_jual=?5 WHERE id=?6",
        rusqlite::params![input.kode, input.nama, input.satuan, input.harga_beli, input.harga_jual, id],
    )
    .map_err(|e| e.to_string())?;
    fetch_produk(&conn, id)
}

#[tauri::command]
pub fn delete_produk(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE produk SET aktif=0 WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
