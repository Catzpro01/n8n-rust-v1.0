# AGENT4_SCHEMA_COMPILER_SPEC — Spesifikasi 8n-schema-compiler

**Penulis:** agent4 (Backend Engineer, ROLE_BACKEND)
**Tanggal:** 2026-09-09 (draf v0.1 -> v0.2)
**Riwayat:** v0.2 menyerap temuan adversarial agent4 F-C2/F-C3/F-C4 (#180) + registry alias deprecated (AGENT4-NODE-ALIAS-DEPRECATION.md #216). Perubahan v0.2 ditandai [v0.2].
**Mandat:** Tugas resmi fern #82 (Schema Compiler & TUI/CLI) + #93 (audit area INodeProperties & OpenAPI Codegen)
**Dokumen acuan:** PRD-1-N8N-ANALYSIS.md (khususnya §7.2 INodeProperties, §7.3 Declarative HTTP routing, §7.5 Credential), PRD-2-RUST.md (§3.2 struktur crate, §4 pilihan teknologi, §7 Node System, §7.4 Codegen OpenAPI)
**Status:** DRAFT untuk review matt (orchestrator) & agent5 (QA/security). Bukan keputusan.

---

## 0. Catatan koreksi penomoran (temuan audit)

Mandat #93 menyebut "audit Section 15-17 (INodeProperties & OpenAPI Codegen)". **Fakta:** PRD-2-RUST.md v1.0 yang diunggah matt (38.808 byte, sha256 `6073140523bb61238ef28b112efda38fcd0f803785651b237ca00f665b71f6ba`) hanya memiliki **16 seksi**, dan area INodeProperties/codegen berada di **§7 (Node System) & §7.4 (Codegen OpenAPI)**, akar datanya di **PRD-1 §7.2/§7.3**. Dokumen ini memakai pemetaan aktual tersebut. (Koreksi serupa mungkin diperlukan untuk pembagian seksi agent lain — disarankan matt membuat tabel pemetaan mandat #93 → seksi aktual di PRD-3-PERFECTION-CHECKLIST.md.)

---

## 1. Tujuan & posisi arsitektur

`8n-schema-compiler` adalah **kompiler skema deklaratif**: membaca deklarasi parameter bergaya n8n `INodeProperties` (yang dalam kernel kita berpadanan dengan `ParameterSchema`/`ParameterField`/`DisplayCondition`) dan menghasilkan artefak turunan. Satu sumber definisi, banyak konsumen — prinsip yang sama dengan `INodeProperties` n8n yang menggerakkan validasi DAN UI dari satu definisi (PRD-1 §7.2).

**Dua arah codegen yang TIDAK BOLEH dicampur:**

| Arah | Nama | Input → Output | Pemilik |
|---|---|---|---|
| **A: deklarasi → konsumen** | `8n-schema-compiler` (spec ini) | `INodeProperties`/`ParameterSchema` → validator, OpenAPI, form schema | agent4 |
| **B: OpenAPI spec → node** | `openapi-codegen` (PRD-2 §7.4) | OpenAPI spec publik → NodeDescriptor + eksekusi HTTP deklaratif | (Fase 2 prototipe, PRD-2) |

Keduanya bertemu pada **IR node yang sama** (`NodeDescriptor` + `ParameterSchema` kernel). A memastikan node yang ditulis/diimpor konsisten; B memperbanyak node dari API eksternal.

**Konsumen output A (Fase 1-3):**
1. **Validator parameter** — saat import workflow & saat eksekusi (resolusi parameter per item, PRD-1 §5.1 langkah 1).
2. **OpenAPI 3.1 spec** — dokumen route REST yang relevan dengan node (Fase 2 `api`/axum).
3. **Form schema** — bangkitkan form UI dari `ParameterSchema` (Fase 3; PRD-2 §9.3 menegaskan `ParameterSchema` memang dirancang untuk ini).
4. **Ekspor/dokumentasi** — daftar parameter node untuk CLI `list` (Fase 1 `cli`) & dokumentasi.

---

## 2. Model input: cakupan tipe INodeProperties

Tabel status dukungan **v0.1 (eksak, tanpa TBD)**. Status `DEFER` = tidak didukung di v0.1 dengan alasan teknis eksplisit, bukan "belum dipikirkan".

| # | Tipe n8n | Representasi IR | Status v0.1 | Catatan |
|---|---|---|---|---|
| 1 | `string` | `StringField` | ✅ SUPPORTED | incl. `typeOptions.codeEditor/rows/password`, `placeholder` |
| 2 | `number` | `NumberField` | ✅ SUPPORTED | `typeOptions.minValue/maxValue` → batas validasi eksak |
| 3 | `boolean` | `BooleanField` | ✅ SUPPORTED | |
| 4 | `options` | `OptionsField(oneOf)` | ✅ SUPPORTED | `options[].value` boleh string/number |
| 5 | `multiOptions` | `MultiOptionsField` | ✅ SUPPORTED | `typeOptions.multipleValues=true` implisit |
| 6 | `dateTime` | `DateTimeField` | ✅ SUPPORTED | format ISO-8601; validasi parse ketat |
| 7 | `json` | `JsonField` | ✅ SUPPORTED | string berisi JSON; parse saat validasi |
| 8 | `collection` | `CollectionField` | ✅ SUPPORTED | objek dengan sub-field opsional |
| 9 | `fixedCollection` | `FixedCollectionField` | ✅ SUPPORTED | bersarang; struktur paling kompleks — wajib golden test |
| 10 | `stringArray` | `StringArrayField` | ✅ SUPPORTED | |
| 11 | `object` / `objectArray` | `ObjectField` | 🟡 PARTIAL | butuh contoh nyata dari node upstream utk mengunci semantik; hanya `object` di v0.1, `objectArray` DEFER |
| 12 | `resourceLocator` | `ResourceLocatorField` | 🟡 PARTIAL | `modes[].name/type` didukung; `initCode`+`autocompleteFunction` (JS) DEFER — butuh keputusan ekspresi/QuickJS |
| 13 | `notice` / `button` | (diabaikan) | ⏭ SKIP | murni UI; tidak masuk validasi/runtime — dicatat di katalog deviasi |
| 14 | `credentials` (declarative di description) | `CredentialRef[]` | ✅ SUPPORTED | referensi tipe kredensial (PRD-1 §7.5) |
| 15 | `filter` (node Filter) | `FilterField` | 🟡 PARTIAL | semantik kondisi kompleks; DEFER ke batch node Filter (Tier 2) |

**Aturan lintas tipe:**
- `default` wajib ada dan tipe-nya konsisten dengan tipe field → kesalahan default = error kompilasi skema (bukan runtime).
- `required: true` tanpa default → error saat import workflow (bukan saat eksekusi) — pesan menyebut nama node & field.
- Nilai parameter boleh **expression** `{{...}}` (PRD-1 §8.1). Validator hanya memeriksa bentuk (string mengandung `{{`); evaluasi adalah domain `expr-quickjs`. Pengecualian: field bertipe number/boolean yang berisi expression harus diizinkan lewat validasi tipe (ekspresi menghasilkan nilai).
- `displayOptions.show/hide` → kompilasi menjadi `DisplayCondition` kernel (sudah ada & teruji di kernel: `parameter_schema_validation_and_visibility`). Semantik: `AND` antar key; `OR` di dalam array value (pola n8n).

---

## 3. Pipeline kompilasi

```
INodeProperties (JSON deklarasi node, dari node source atau impor)
        │  (1) PARSE + NORMALISASI
        ▼
   SchemaIR (representasi antara, validasi semantik di sini)
        │  (2) KONSUMSI: tiga backend + satu refleksi
        ├──▶ (a) Validator Rust (di-compile ke crate nodes-*)   [Fase 1]
        ├──▶ (b) OpenAPI 3.1 schema + route spec (JSON)          [Fase 2]
        ├──▶ (c) Form schema JSON (konsumen UI)                  [Fase 3]
        └──▶ (d) Registry: daftar node + parameter utk CLI/docs  [Fase 1]
```

Tahap (1) adalah **satu-satunya** tempat error skema dilaporkan, dengan format: `node:<nama> field:<path> pesan:<alasan>`. Tidak ada error validasi yang muncul pertama kali saat runtime eksekusi.

## 4. Aturan mapping utama (acuan implementasi)

1. **Kesetiaan**: pemetaan `INodeProperties` → `ParameterSchema` harus setia (PRD-1 §7.2). Setiap field yang tidak dikenal di deklarasi node → warning + catatan katalog deviasi, BUKAN silent drop (anti pola "impor sukses tapi perilaku beda").
2. **`name` adalah kontrak**: path parameter = `name` (dengan titik untuk `fixedCollection` bersarang, pola n8n). Expression `$('Node')` merujuk node by display-name (PRD-1 §4.2): rename node = update referensi — menjadi tanggung jawab **layer workflow**, tapi schema compiler harus menyediakan daftar dependensi nama (nama node yang direferensikan ekspresi) supaya rename-check bisa otomatis. Output: file `node-refs` per workflow (Fase 1).
3. **`routing` (declarative HTTP)**: field dengan `routing.send.{type: query/body/header/path}` dikompilasi menjadi spesifikasi request deklaratif (PRD-1 §7.3). Ini adalah satu-satunya jembatan antara node HTTP-Request & codegen arah B: node hasil codegen OpenAPI memakai bentuk yang sama.
4. **Kredensial**: deklarasi `credentials[]` mengikat node ke tipe kredensial (least-privilege, konsisten dgn prinsip PRD-2 §11). Compiler menerbitkan daftar `(node, credential_type)` untuk audit.
5. **Kesalahan impor**: node dengan tipe field `DEFER` yang dipakai workflow nyata → impor tetap sukses (L1 = 100% parse, PRD-2 §10.1) dengan **daftar field yang diabaikan** dicatat di katalog deviasi; eksekusi hanya gagal bila field itu memengaruhi perilaku node — keputusan per-node, dicatat eksplisit.

## 5. Artefak verifikasi (mengikuti PRD-2 §1.2: wajib, bukan opsional)

1. **Tabel mapping**: 1 baris per tipe (tabel §2 di atas) — 100% tipe n8n tercakup dengan status + alasan.
2. **Golden files**: kompilasi N deklarasi node nyata yang diambil dari repo upstream n8n (sumber: `packages/@n8n/node-cli/src/template/templates/declarative/*/nodes/*/*.node.json` dan node inti seperti HTTP Request/Set/IF) → snapshot output validator & OpenAPI yang di-review.
3. **Uji lintas korpus**: ke-50 workflow korpus (§10.2 PRD-2) melewati tahap (1) parse+validasi tanpa error tak terduga; daftar field DEFER yang tersentuh korpus dirilis sebagai laporan.
4. **Proptest** (arah kiri-kanan): untuk tiap tipe SUPPORTED, properti "valid menurut validator ⇔ parseable & type-consistent" diuji acak.
5. **Tidak ada kode tanpa bukti**: setiap klaim perilaku n8n di spec ini ditandai sumbernya (PRD-1 seksi) atau `[VERIFIKASI-UPSTREAM]` bila harus dibaca dari kode sumber n8n.

## 6. Risiko utama

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Semantik `fixedCollection`/`resourceLocator` tidak persis n8n | Workflow impor berhasil tapi perilaku beda (silent) | Golden test dari deklarasi upstream + daftar deviasi |
| Field `DEFER` dipakai workflow nyata | Gagal eksekusi sebagian node | Laporan korpus (artefak 3) → prioritas perbaikan berbasis data |
| `object`/`filter` semantik longgar | Validasi lolos tapi runtime error | Partisi ketat: hanya bentuk yang terdefinisi eksak yang SUPPORTED |
| Dua arah codegen (A/B) tercampur | Arsitektur kacau | §1 tabel pemisahan; review gate matt |
| Skema berubah → artefak usang | Validator & OpenAPI tidak sinkron | Compiler = satu-satunya jalan (no hand-edit artefak), CI membangun ulang & diff |

## 7. Keputusan yang diminta (untuk matt/fern — rekomendasi disertakan)

1. **K-1**: penempatan crate — usul `crates/schema-compiler/` (tidak bergantung kernel; hanya memakai tipe data `ParameterSchema` via kontrak kernel) — setuju/ubah.
2. **K-2**: status 4 tipe PARTIAL/DEFER (§2) — disetujui sebagaimana tabel, atau ada prioritas berbeda berbasis korpus?
3. **K-3**: format output OpenAPI versi 3.1 (lebih baru, dukungan `type` union lebih baik) vs 3.0 — usul 3.1.
4. **K-4**: kapan artefak (b) & (c) dibangun — usul: (a)+(d) Fase 1; (b) Fase 2; (c) Fase 3, TAPI format (c) ditetapkan di Fase 1 supaya `ParameterSchema` tidak berubah bentuk lagi.

## 8. Ringkasan

Schema-compiler mengubah aturan main "deklarasi node" dari konvensi menjadi artefak terkompilasi yang bisa diverifikasi — sejalan dengan prinsip PRD-2 §3.3 ("enforcement oleh kompilator, bukan konvensi"). Output Fase 0 dari agent4: dokumen ini + tabel mapping final + daftar deklarasi node upstream untuk golden test. Tidak ada kode yang ditulis sebelum PRD-2 disempurnakan & disetujui (mandat #93).


---

## 9. [v0.2] Default SideEffect untuk codegen B (menjawab F-C2)

Node hasil generate OpenAPI tidak mungkin tahu idempotensi endpoint dari spec.
Tanpa default, recovery memilih alarm palsu massal (semua NonIdempotent) atau bahaya duplikasi (semua Idempotent).
**Default berbasis metode HTTP (ditetapkan di registry, bisa di-override manual per node):**

| Metode | SideEffect default | Catatan |
|---|---|---|
| GET / HEAD / OPTIONS | `Idempotent` | tanpa efek samping |
| PUT / DELETE | `Idempotent` | syarat: seluruh parameter path terisi; diverifikasi per spec |
| POST / PATCH | `NonIdempotent` | default aman |
| (spec ambigu) | kompilasi GAGAL | wajib kurasi manual, bukan hasilkan node rusak |

Override manual: file `verifications/<node>.json` (setara verified-nodes PRD-2 §7.4). Aturan ini masuk ke output compiler sebagai `ResourceHint.side_effect` tiap node.

## 10. [v0.2] Schema registry per (kind, typeVersion) (menjawab F-C3)

Workflow n8n mem-pin `typeVersion` (PRD-1 §4.2); node berubah antar versi. Registry skema:
- Kunci: `(kind, typeVersion)` → `ParameterSchema`.
- Impor: validasi thd skema versi yg dipin; versi hilang/0 → fallback skema tertua dikenal + catat deviasi; TANPA migrasi otomatis (migrasi = mapping eksplisit di katalog deviasi).
- Alias deprecated (function/cron) diaplikasikan saat resolve, sebelum validasi — lihat AGENT4-NODE-ALIAS-DEPRECATION.md.
- Output compiler: kompilasi per `(kind,typeVersion)`, bukan per kind.

## 11. [v0.2] Atribut eval-scope pada ParameterSchema (menjawab F-C4)

Expression n8n dievaluasi per-item (PRD-1 §8.1/§8.5), tapi beberapa nilai wajar sekali-per-task.
Setiap `ParameterSchema` mendapat atribut `eval_scope`:
- `per_item` (default) — untuk nilai yang menyentuh `$json/$itemIndex/$input`.
- `per_task` — pengecualian eksplisit (mis. `executeOnce`, kredensial, header statis; `$execution/$workflow` dibekukan saat task start).
Kontrak ini mencegah dua implementasi engine berbeda output utk workflow sama; kasus uji differential wajib: 1 Set node + expression `$json` + `$itemIndex`.

## 12. [v0.2] Ringkasan delta
1. §9 tabel SideEffect default per metode HTTP (F-C2) — keputusan utk matt.
2. §10 registry per (kind,typeVersion) (F-C3) — memengaruhi parser L1 & schema compiler.
3. §11 eval_scope (F-C4) — kontrak kernel/executor.
4. Referensi registry alias deprecated (function/cron, #216).
Keputusan K-1..K-4 (§7) tetap terbuka utk matt.
