//! Tipe error Rosetta — fail-loud dengan konteks file, tanpa kehilangan sebab.

use std::path::PathBuf;

/// Kesalahan Rosetta. Semua varian menyertakan label sumber (path/asal) supaya
/// laporan bisa menunjuk file persis.
#[derive(Debug, thiserror::Error)]
pub enum RosettaError {
    /// Gagal baca file (I/O).
    #[error("io error on {path}: {source}")]
    Io {
        /// Path/label sumber.
        path: PathBuf,
        /// Sebab I/O asli.
        #[source]
        source: std::io::Error,
    },
    /// JSON tidak valid secara sintaksis.
    #[error("json parse error in {path}: {message}")]
    Json {
        /// Label sumber.
        path: String,
        /// Pesan parser JSON.
        message: String,
    },
    /// JSON valid tetapi bukan struktur workflow yang dapat dipahami.
    #[error("workflow structure invalid in {path}: {reason}")]
    Structure {
        /// Label sumber.
        path: String,
        /// Alasan struktural (mis. `nodes` bukan array, node tanpa `name`).
        reason: String,
    },
    /// JSON melebihi batas kedalaman — fail-closed sebelum serde_json
    /// (RULING 39b/41; guard di batas API 50b).
    #[error("json depth exceeded in {path}: found {found} > max {max}")]
    DepthExceeded {
        /// Label sumber (path/asal).
        path: String,
        /// Kedalaman yang ditemukan saat batas terlampaui (early-return kernel).
        found: u32,
        /// Batas kedalaman yang berlaku (MAX_JSON_DEPTH).
        max: u32,
    },
}
