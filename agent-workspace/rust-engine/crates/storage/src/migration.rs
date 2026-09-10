//! Migration runner — menjalankan file SQL dari migrations/ directory.

use crate::errors::StorageError;
use rusqlite::Connection;
use std::path::Path;

/// Jalankan semua migrasi yang belum diterapkan.
pub fn run_migrations(conn: &Connection, migrations_dir: &Path) -> Result<u32, StorageError> {
    // Buat tracking table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations_applied (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now')),
            filename TEXT NOT NULL
        )"
    ).map_err(|e| StorageError::Migration(e.to_string()))?;

    let mut applied = 0u32;
    
    // Scan migration files
    let mut entries: Vec<_> = std::fs::read_dir(migrations_dir)
        .map_err(|e| StorageError::Migration(format!("cannot read migrations dir: {}", e)))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let filename = entry.file_name().to_string_lossy().to_string();
        let version: i64 = filename.split('_')
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        // Cek apakah sudah diterapkan
        let already: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM _migrations_applied WHERE version = ?1",
            [version],
            |row| row.get(0),
        )?;

        if already {
            continue;
        }

        let sql = std::fs::read_to_string(entry.path())
            .map_err(|e| StorageError::Migration(format!("cannot read {}: {}", filename, e)))?;

        // Eksekusi dalam transaksi
        conn.execute_batch(&sql)
            .map_err(|e| StorageError::Migration(format!("failed at {}: {}", filename, e)))?;

        // Catat migration
        conn.execute(
            "INSERT INTO _migrations_applied (version, filename) VALUES (?1, ?2)",
            rusqlite::params![version, filename],
        )?;

        applied += 1;
        println!("Migration applied: version={} filename={}", version, filename);
    }

    Ok(applied)
}
