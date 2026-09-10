# BAHASA-BERSAMA.md — Standar Glosarium, Kontrak Tipe Data, dan Protokol Kompak Swarm
**Versi:** 1.2.0-CANONICAL | **Status:** RATIFIED & BINDING | **Otoritas:** Pemilik Proyek & Lead Architect (@matt, @fern)

Dokumen ini adalah **Kamus Resmi Tunggal (Single Source of Truth)** untuk seluruh agen pengembang n8n-Rust Swarm. Seluruh tipe data, peristilahan, dan protokol komunikasi WAJIB merujuk pada standar ini untuk mencegah 100% miskomunikasi antar-agen.

---

## 1. HIERARKI DOKUMEN & OTORITAS
1. **PRD-3-CANONICAL-APPROVED.md** + **PRD-WORKFLOW-HUB-STANDALONE.md**: Dokumen arsitektur tertinggi yang disahkan.
2. **PRD-1 dan PRD-2**: Resmi **SUPERSEDED** (usang, tidak boleh dijadikan acuan arsitektur/tipe aktif).
3. **Multi-Tenancy**: Resmi **DICABUT**. Arsitektur 100% fokus pada *Single-Tenant Ultra-Efficiency (<500MB RAM)* dengan isolasi sandbox WASM (32MB limit) per eksekusi node/workflow.
4. **Otoritas Supervisi & Arsitektur**: `@fern` adalah Juru Bicara Resmi & Kepala Pengawas Pemilik Proyek dengan mandat eksekutif penuh. `@matt` adalah Lead Architect & Chief AI Orchestrator penanggung jawab integritas kernel kanonik dan pemegang otoritas merge/veto.

---

## 2. GLOSARIUM & STANDAR TIPE DATA INTI KANONIK (RUST CANONICAL TYPES)
*(Disahkan per Ruling 3 Lead Architect @matt #1000 & Directive @fern #994. Tipe fiktif PayloadRef, SpillId(uuid), SpillRef, Bytes resmi DILARANG)*

| Konsep / Istilah | Definisi Kanonik di `kernel/src` | Tipe Rust Standar | Keterangan & Batasan |
| :--- | :--- | :--- | :--- |
| **Item** | Satuan data payload eksekusi (JSON-compatible). | `struct Item` | Membungkus `serde_json::Value`. |
| **ItemList** | Wadah daftar item eksekusi node (zero-copy inline vs spilled). | `enum ItemList { Inline(Vec<Item>), Spilled(SpilledList) }` | 41 kecocokan di kernel. Pengganti resmi PayloadRef. |
| **SpilledList** | Metadata daftar item yang di-spill ke disk saat melewati ambang RAM. | `struct SpilledList { path: SpillPath, len: u32, total_bytes: u64, codec: SpillCodec }` | item.rs:212-219. TIDAK punya field `store_ref`/`content_id`; store diakses sebagai parameter `&dyn SpillStore` (item.rs:303,319). |
| **ContentId** | Identitas konten numerik 64-bit untuk CASD deduplikasi (bukan penamaan berkas spill). | `struct ContentId(pub u64)` | **BUKAN UUID!** Murni u64 bebas dependensi eksternal. Penamaan berkas fisik memakai `SpillPath`. |
| **BinaryLocation** | Penunjuk lokasi data biner (Inline vs Ref ContentId). | `enum BinaryLocation` | `item.rs:159-162`. `Inline(Vec<u8>)` atau `Ref(ContentId)`. |
| **SpillStore** | Trait penyimpanan spill (In-Memory atau FileSpillStore). | `pub trait SpillStore: Send + Sync` | Disk streaming bebas leak; izin berkas diatur via set_permissions 0600 (Ruling 25 melarang libc::umask). |
| **Checkpoint** | Snapshot state eksekusi untuk recovery dan replay. | `struct Checkpoint` | Titik simpan status eksekusi workflow. |
| **SideEffect** | Pencatatan efek samping I/O non-deterministik. | `enum SideEffect` | Rekam I/O waktu/jaringan untuk replay equality. |
| **ParameterSchema** | Definisi skema dan validasi parameter node. | `struct ParameterSchema` | Kontrak tipe input parameter node. |
| **PriorOutputs** | Trait akses output node sebelumnya dalam DAG. | `pub trait PriorOutputs` | Trait sambungan context eksekusi node. |
| **Hash BLAKE3** | Hash kriptografi 32-byte untuk payload digest, CASD dedup, dan event log. | `[u8; 32]` atau `blake3::Hash` | Hex string 64-karakter. Cepat (0.48 ms/MB). |
| **Checksum SHA-256** | Checksum kanonik identitas berkas, manifest WCB, dan FreeTSA RFC 3161 anchor. | `[u8; 32]` | Standar integritas absolut dan verifikasi eksternal. |
| **Execution Envelope** | Catatan eksekusi deterministik dengan rantai rolling audit hash. | `struct ExecutionEnvelope` | Framing biner kanonik u32-BE prefix per field (C-02); BUKAN JSON terurut. |
| **Timeline Replay** | Rekaman urutan event transisi eksekusi workflow. | `struct TimelineEvent` | JSON-L terindeks urutan (seq, attempt, node_id). |

---

## 2.1 DEFINISI STANDAR STATUS SELESAI (DONE SEMANTICS)
*(Per Audit #960 & Ruling #1000 Matt, disahkan Directive #994 Fern)*

1. **`DONE-CODE` (Implementasi Kode Nyata)**:
   - Wajib memiliki path kode nyata di `crates/` kanonik atau crate prototype terverifikasi.
   - Wajib memiliki suite `#[test]` yang lulus 100% (termasuk uji negatif/mutan).
   - Wajib lulus tinjauan independen oleh agen reviewer terpisah (Pilar 5 SOP #864).
   - Wajib mencantumkan commit hash spesifik.
2. **`DONE-DOC` (Spesifikasi & Rencana Arsitektur)**:
   - Berlaku untuk tugas berawalan "Spesifikasi", "Kontrak", "Rencana", atau "RFC".
   - Wajib disetujui minimal oleh 2 peer reviewer formal.
   - Berstatus `[CONSENSUS-REACHED]` di forum sebelum implementasi kode dapat dimulai.

---

## 3. PROTOKOL KERJA KOMPAK & SINKRONISASI MITRA (BUDDY SWARM)
Untuk memastikan seluruh agen selalu bekerja kompak dan tidak ada agen yang terisolasi atau mengalami kebuntuan komunikasi, ditetapkan **5 Pasangan Mitra Kompak**:

1. **Mitra A (Kernel & Storage)**: `@agent1` (ROLE_CORE) & `@agent2` (ROLE_STORAGE)
   - Lingkup: `crates/engine-core`, `crates/storage`, `crates/data-plane`, FileSpillStore, memory bounds.
2. **Mitra B (Ekspresi, Runtime & Efisiensi)**: `@agent3` (ROLE_EXPRESSION) & `@agent8` (ROLE_PERF)
   - Lingkup: `casd-prototype`, `ebc-prototype`, QuickJS eval, benchmarking RAM <500MB.
3. **Mitra C (Skema, Parsing & Kontrak)**: `@agent4` (ROLE_SCHEMA) & `@agent6` (ROLE_WASM)
   - Lingkup: Rosetta Parser, WCB WASM bridge, normalisasi JSON, node manifest.
4. **Mitra D (Koleksi Data & Ekstraksi)**: `@agent7` (ROLE_AI_MCP) & `@agent9` (ROLE_INTEGRATION)
   - Lingkup: 4-Tier Cascading Scraping, Scrapling, Puppeteer MCP, Public Data Connectors.
5. **Mitra E (Kepatuhan, Keamanan & Arsitektur)**: `@agent10` (ROLE_COMPLIANCE) & `@matt` (ROLE_ARCHITECT)
   - Lingkup: Audit hash-chain, verifikasi gate CI/CD, merge ke `kernel-asli-d3bcff0`.

---

## 4. STANDAR KOMUNIKASI & REGISTRY SWARM (#973 & #994)
1. **Konvensi Tag Judul Wajib**:
   - `[STATUS]` : Laporan perkembangan tugas aktif dan status menunggu.
   - `[REVIEW-REQ]` : Permintaan review resmi dengan menyebutkan nama reviewer.
   - `[CONSENSUS-ACK]` : Persetujuan formal atas RFC/dokumen.
   - `[HANDOFF]` : Serah terima antar-fase implementor ke reviewer.
   - `[RFC]` : Usulan perubahan arsitektur atau kontrak baru.
   - `[DIRECTIVE]` : Arahan resmi dari Juru Bicara `@fern` atau Lead Architect `@matt`.
2. **Registri Sentral**: Seluruh RFC dan permintaan review aktif dicatat dan dilacak di `docs/SWARM-REGISTRY.md` (dikuratori oleh `@agent9`).
3. **Single Source of Truth**: Tabel antrean `task_queue` di `comm.db` adalah satu-satunya cermin kebenaran status eksekusi.
4. **Status Menunggu**: Agen yang sedang menunggu review konsensus diakui berada dalam status **AKTIF-MENUNGGU** (bukan idle kosong).
