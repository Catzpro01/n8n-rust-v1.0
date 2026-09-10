# PREFACTOR-4 — Audit Trait Node (kernel) untuk TB-01

- Penugasan: RULING-63 (#1664) — "Audit trait Node di kernel. TB-01 bergantung padanya.
  Buat ia bisa dipakai implementor node tanpa perubahan. Laporkan apa yang menghalangi,
  jangan diam-diam mengubah kontraknya."
- Pelaksana: agent4 (ROLE_SCHEMA) — 2026-09-10. Jenis tiket: **AFK audit** + 1 keputusan
  **HITL** (di §5). Read-only thd pohon kanonik; probe kompilasi di /tmp (di luar pohon).
- Anchor sha: kernel-asli-d3bcff0 (git HEAD; workspace rust-engine mengacu kernel via path).

## 0. Batas verifikasi (jujur)
- Audit kontrak = baca sumber + grep, BUKAN eksekusi runtime (tidak ada engine yang
  memanggil trait — TB-01 belum ada). Probe = `cargo check` compile-only, tanpa runtime
  tokio (async_trait macro expand), tanpa test perilaku.
- Yang diverifikasi: bentuk kontrak, ketersediaan jalur dependensi, keterkompilasian
  implementor minimal. Yang TIDAK diverifikasi: perilaku engine ↔ node (belum ada).

## 1. Kontrak terverifikasi (file:line)
Kernel: `/opt/agent-workspace/kernel-asli-d3bcff0/crates/kernel` (lib.rs 100+ baris; 7 modul;
3.073 baris src). Freeze D115/A-05: hanya serde/serde_json/async-trait/thiserror
(Cargo.toml `[dependencies]`; lib.rs:7-11).

- `pub trait Node: Send + Sync` @ src/node.rs:17 — async_trait; dua method:
  `descriptor(&self) -> &NodeDescriptor` (statis, sekali; node.rs:24-25) dan
  `async fn execute(&self, ctx: &mut NodeContext<'_>) -> Result<NodeOutput, NodeError>`
  (node.rs:27-31; semua akses input via ctx; larangan menangkap payload mentah).
- `NodeDescriptor` @ node.rs:30-75 — {kind: NodeKind, version: u16, display_name, group,
  hints: ResourceHint, params: ParameterSchema, credentials: Vec<CredentialSpec>,
  inputs/outputs: u8 (default 1), execute_once, description?, icon?}; `new(...)` @ node.rs:80.
- `NodeGroup`/`ResourceHint`/`Weight`/`BufferingMode`/`SideEffect` @ node.rs:88-230
  (default aman: Action/Batch/SideEffect::None/resumable).
- `NodeOutput` @ node.rs:240 — branches: Vec<ItemList>; helper single/none/two.
- `NodeContext<'a>` @ src/context.rs:19-65 — struct trait-object:
  execution: &ExecutionMeta, node: &NodeMeta, input: &[ItemList], prior: &dyn PriorOutputs,
  credentials, static_data, expr, http, blobs, spill, log, cancel, deadline, attempt.
  Method: input(i) @ context.rs:92, remaining_ms @ 103, is_cancelled @ 110.
- `ExecutionMeta` @ context.rs:120-148 — {id, workflow_id, workflow_version, mode
  (Production/Manual/Test/Webhook), timezone IANA, execution_order (V0Legacy/V1), started_at}.
- `NodeMeta` @ context.rs:188-193 — {id: NodeId, position: (i32,i32), input_index: u8}.
- `Item` @ src/item.rs:52-73 — {json: Value (serde_json; bukan generic, by design), binary?,
  paired_item?}; `Item::new/from_obj/estimated_bytes` @ 75-95.
- `ItemList` @ item.rs:203-210 — Inline(Vec<Item>) | Spilled(SpilledList); empty/single/len/
  ram_footprint/from_vec/view @ 263-320. `ItemView` @ 335+ (len/is_empty/cursor(chunk)).
- `ParameterSchema`/`ParameterField`/`ParameterKind`/`DisplayCondition`/`CredentialSpec`
  @ src/params.rs:22-309 — schema tunggal utk validasi + form (A-30); validate(params: &Value)
  @ params.rs:44 (dipanggil dari LUAR, mis. validator).
- Jalur dependensi: rust-engine/Cargo.toml:27-28 — kernel + data-plane path ke
  kernel-asli-d3bcff0 → KONTRAK SUDAH menjadi dep workspace rust-engine (Cargo.lock memuat
  kernel). Versi kernel workspace: serde 1, serde_json 1, async-trait 0.1, thiserror 2,
  rust-version 1.80, edition 2021.
- Konvensi NodeKind: kernel `NodeKind(pub String)` @ id.rs:71 (contoh `http.request`, `if`,
  `merge`, `generated.stripe.charges` — node.rs:32); kindmap rosetta (kindmap.rs, RATIFIED
  agent1 #1241 + RULING 35 #1249) menurunkan NodeKind dot-case dari type_full n8n →
  KONSISTEN dua sisi (contoh: n8n-nodes-base.httpRequest → `http.request`).

## 2. TEMUAN — apa yang menghalangi implementor node (urut dampak)

### T-1 [KRITIS] execute() TIDAK punya kanal untuk parameter INSTANCE node
Bukti (verified): grep `parameter|params` di seluruh src kernel hanya menemukan
(1) `NodeDescriptor.params: ParameterSchema` (node.rs:47) = SKEMA per-TIPE utk validator/UI,
(2) `ParameterSchema::validate(params)` (params.rs:44) yang menerima params dari pemanggil
luar, (3) evaluasi DisplayCondition thd params (params.rs:169/267). **Tidak ada** field/method
parameter di `NodeContext` (context.rs:19-65), `NodeMeta` (context.rs:188-193 = id/position/
input_index), maupun `ExecutionMeta` (context.rs:120-148).
Konsekuensi: perilaku node nyata BERASAL dari parameter workflow JSON (`node.parameters`):
Set → assignments; HTTP Request → url/method; IF → kondisi; dst. Implementor `execute()`
tidak dapat membaca konfigurasi instance-nya lewat ctx. Opsi yang tersisa (bawa params dalam
struct node sendiri) tidak punya kontrak factory — engine tidak tahu cara meng-instantiate
node ber-state dari workflow (lihat T-2). **TB-01 (Set) tidak dapat diimplementasikan thd
trait tanpa memutus salah satu: ubah kontrak (tambah kanal params) ATAU bangun pola
instance+factory di luar kernel (executor/nodes-core) yang belum ada kontraknya.**

### T-2 [SEDANG] Tidak ada contoh implementor maupun uji kontrak
Kernel dev-dependencies hanya serde_json (Cargo.toml `[dev-dependencies]`); tidak ada
implementasi `Node` contoh, tidak ada test yang meng-instantiate implementor. Pola hidup
node (stateless + ctx vs struct ber-state + factory) TIDAK ditetapkan di mana pun — lihat
komentar node.rs:2-15 ("plugin boundary") yang hanya bicara pemanggilan, bukan pembuatan.
Implementor TB-01 harus menebak pola; kesalahan pola = ulang.

### T-3 [INFO — beban sisi ENGINE, bukan implementor] Semua service wajib non-optional
`NodeContext` memuat 10+ trait-object TANPA opsi kosong (expr, http, blobs, spill, log,
credentials, prior, static_data, cancel; context.rs:19-65). Engine TB-01 (2 node statis)
wajib menyediakan implementasi semua — termasuk ExpressionEngine, HttpClient, BlobStore,
SpillStore — atau tidak bisa membentuk ctx sama sekali. Sengaja (least-privilege D92,
RAM D72), bukan bug; dicatat agar TB-01 menganggarkan stub service sejak awal.

## 3. Yang BUKAN penghalang (diverifikasi, agar tidak digarap sia-sia)
- Jalur dependensi: sudah ada (rust-engine → kernel path; Cargo.lock konsisten).
- Versi/toolchain: serde 1/serde_json 1/async-trait 0.1/thiserror 2 vs workspace rust-engine
  (rosetta/mcp pin serde =1.0.197 — kompatibel dgn "1"); kernel rust-version 1.80.
- NodeKind dua sisi konsisten (kindmap RATIFIED; contoh kernel cocok).
- Model schema ganda (ParameterSchema kernel vs param_schema korpus rosetta 96+50) = bentuk
  BEDA tapi bukan blocker TB-01: deskriptor Set bisa memakai ParameterSchema statis manual;
  konverter korpus→kernel = pekerjaan lanjut (dicatat, tidak dikerjakan).

## 4. Bukti keterimplementasian (probe compile, di luar pohon)
Crate `/tmp/pf4-probe` (workspace terisolasi `[workspace]` kosong; kernel via path dep
absolute; CARGO_TARGET_DIR=/tmp/pf4-target):
- Mengimplementasikan `Node` utk struct SetNode {descriptor, params} — `#[async_trait]`,
  `descriptor()` mengembalikan &self.descriptor (OnceLock/static tak perlu),
  `execute()` membaca `ctx.input(0)` + `self.params`.
- Hasil: `cargo check` → `Finished dev profile ... in 0.11s` (0 error).
- Kesimpulan: kontrak Node TERKOMPILASI utk implementor sederhana; satu-satunya gap
  fungsional = T-1 (kanal params instance). Probe dibuang (bukan artefak).

## 5. Rekomendasi + keputusan yang diminta (HITL → Pemilik Produk / C-6)
PREFACTOR-4 dilarang mengubah kontrak; berikut opsi utk keputusan:
- **Opsi-1 (ubah kontrak, rekomendasi):** tambahkan kanal parameter instance terkecil di
  `NodeMeta` (`pub params: &'a Value` atau `Map<String, Value>`) @ context.rs:188 — engine
  selalu memegang `node.parameters` workflow saat memanggil execute; selaras A-30 (satu
  sumber schema+validasi+form) dan menghilangkan T-1 permanen utk SEMUA node. Biaya: field
  baru di struct (breaking utk konstruktor manual — saat ini tidak ada konsumen di luar
  kernel, jadi murah). Mekanik: keputusan + ADR kecil (KERNEL-SPEC), eksekusi boleh agent4/
  C-6.
- **Opsi-2 (tanpa ubah kontrak):** pola instance+factory di crate executor/nodes-core:
  node struct membawa params; registry `kind → fn(&Value) -> Box<dyn Node>`; kernel tetap
  bersih. TB-01 bisa jalan dengan Opsi-2, tetapi setiap crate node harus menaati pola yang
  baru ditulis (belum ada) dan schema-validasi instance tetap di luar.
- Keputusan yang diminta (SATU): kanal parameter instance masuk kontrak (Opsi-1) atau pola
  eksternal (Opsi-2)? Rekomendasi agent4: **Opsi-1** — murah sekarang (0 konsumen), makin
  mahal nanti.
- Tanpa keputusan ini, TB-01 terblokir pada T-1 (bukan pada hal lain).

## 6. Kepatuhan
- Nol perubahan pada pohon kanonik (rust-engine & kernel-asli-d3bcff0): audit read-only;
  probe di /tmp, tidak di-commit, target di /tmp. Tidak ada commit baru.
- Laporan ini: /opt/agent-workspace/docs/PREFACTOR-4-AUDIT-TRAIT-NODE.md (belum di-version-
  control; docs/ = artefak bersama, sha256 laporan dicatat di pesan forum).
