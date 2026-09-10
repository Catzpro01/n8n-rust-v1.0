# R-6 KERNEL PREFLIGHT — pemetaan tipe n8n → NodeDescriptor/ParameterSchema (kernel kanonik)

**2026-09-09 · agent4 · read-only kernel-asli-d3bcff0 (HEAD f4adc8f) — PERSIAPAN, bukan task resmi (menunggu arahan matt #1106 butir 3a).**

## Fakta kernel (terverifikasi langsung)
- `NodeKind(pub String)` — newtype dotted, mis. `http.request`, `if`, `merge`, `generated.stripe.charges` (id.rs:71-86). `is_generated()` = prefix `generated.`. BUKAN enum; tak ada FromStr n8n.
- `NodeDescriptor` (node.rs:31-51): { kind: NodeKind, version: **u16**, display_name, group: NodeGroup, hints: ResourceHint, params: ParameterSchema, credentials: Vec<CredentialSpec> }. Catatan D89: "explicit node versions; parameters evolve; workflows pin a version".
- `ParameterSchema` (params.rs:22): { fields: Vec<ParameterField> } + `validate(&Value) -> Vec<ValidationError>` (required+visible+kind; displayOptions → `display_if`).
- `ParameterField` (params.rs): { name, displayName, kind: ParameterKind, default?, required, displayOptions?, options?, supports_expression, ... }.

## Implikasi utk Rosetta R-6
1. **Mapping tipe n8n → NodeKind TIDAK mekanis** (camelCase → dot + hapus prefix + kasus khusus). 147 tipe base korpus butuh tabel mapping eksplisit + verifikasi (pola V-1..V-3). Contoh usul: `n8n-nodes-base.httpRequest` → `http.request`; `n8n-nodes-base.if` → `if`; `googleSheets` → `googleSheets`? (kebab/dot belum ditetapkan — OPEN QUESTION).
2. **version u16 vs typeVersion desimal n8n** (1.1–4.7): D89 pin satu u16. Apakah version = major (4) atau major*10+minor (45) — OPEN QUESTION utk @agent1 (owner kernel §3.4). Rosetta menyimpan TypeVersion{major,minor} utuh → konversi ke u16 terjadi di TEPI (bukan di IR).
3. **params n8n bebas-form vs ParameterSchema declarative**: Rosetta parameter dipertahankan utuh; R-6 = membangun ParameterSchema per tipe (dari mana? kode sumber n8n INodeProperties = verifikasi upstream; openapi-codegen agent9 = jalur generated.*). Tidak mengubah node asli.
4. **Langchain @n8n 31 tipe & komunitas 8 tipe** → BUKAN hand-written base; kandidat WCB manifest (agent6) / opaque (R-2).
5. **validate() = sasaran diff-test L1 nanti**: schema salah → validate menolak parameter nyata → test tangkap. R-6 tiap mapping wajib lolos validate thd parameter korpus nyata (fixture ada 171 file).

## Deliverable R-6 (saat disetujui)
Tabel mapping (tipe n8n → kind kanon → versi u16 → ParameterSchema) per tipe base dgn bukti: (a) parameter korpus lolos validate, ATAU (b) [VERIFIKASI-UPSTREAM] utk tipe tanpa fixture. Nol klaim tanpa bukti (penerimaan katalog #418).

## Status
PREFLIGHT SAJA — belum ada mapping ditulis. Menunggu arahan matt.

---
## KEPUTUSAN KONVENSI — #tech-debate #1176 (agent1, 2026-09-09, PROVISIONAL)
**Sumber:** KEPUTUSAN-PROVISIONAL agent1 → agent4 re #1174. Override terbuka @matt/@fern; bila diubah, tabel di-redo.
1. **KONVENSI KIND = OPSI-A**: tabel dot-case EKSPLISIT + fail-loud-unmapped. Preseden kernel node.rs:32 (http.request, if, merge, generated.stripe.charges). Opsi B (transform mekanis) & C (prefix penuh n8n-nodes-base.*) DITOLAK. `splitInBatches` utuh (tidak dipecah) — terlihat di tabel.
2. **VERSION u16 = MAJOR*10 + minor** + GUARD KERAS `minor<10` else unmappable-error. Total order u16 = leksikografis (major,minor); minor kompatibel tidak hilang; guard menutup tabrakan 4.10-vs-5.0.
3. Tabel eksplisit = greppable + diff-test thd korpus + tak bisa salah-tebak (pelajaran #1151: salin dari sumber, jangan turunkan dari ingatan).
**Implikasi:** OPEN QUESTION (1) & (2) di atas TERTUTUP untuk desain R-6; masih menunggu task resmi matt (#1106 butir 3a) utk eksekusi. Agent1 #1177: "R-6 tak lagi terblokir dari sisi saya."
*(Validasi korpus 171 file thd #1176: 0 pelanggaran guard, 217 pasangan (tipe,major) = ukuran tabel minimum — lihat R6-TYPE-INVENTORY.md; publikasi #1199.)*
