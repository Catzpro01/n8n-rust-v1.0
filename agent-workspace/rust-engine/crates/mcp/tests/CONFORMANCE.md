# CONFORMANCE.md — status kasus TC-01..TC-64 (in-tree, crates/mcp/tests/conformance.rs)

- Sumber kasus: docs/AGENT7-MCP-CONFORMANCE-TESTS.md v0.1 (ruang-id 01..64; 48 baris terdefinisi)
- Mandat: fern #1208 (RE: #1184 + DM #1193) — in-tree conformance harness kanonik; menyempurnakan acceptance #1092
- Hasil: `cargo test -p mcp` → 49 unit + 53 conformance = 102 hijau; clippy --all-targets 0 warning
- Berkas: tests/conformance.rs (sha berubah — lihat git d14cb48); +1 uji Ruling 39b (MAX_JSON_DEPTH=64)
- Metode: in-proses thd ServerCtx/facade/registry — deterministik, tanpa proses spawn, tanpa wall-clock,
  tanpa jaringan; deps tetap serde/serde_json (Ruling 7 #1023). +3 uji integritas registri + 1 frame-helper
  + 1 uji Ruling 39b (MAX_JSON_DEPTH=64, string-aware).

## Ringkasan status (48 baris TC + 4 uji ekstra)

| Status | Jumlah | Arti |
|---|---|---|
| PASS | 28 | perilaku kanonik sesuai tabel v0.1 |
| ADAPTED | 16 | asersi mengikuti kanonik; delta vs harfiah tabel v0.1 tercatat (komentar dlm kode) |
| GAP-SPEC | 2 | TC-13, TC-22 — keputusan spec diperlukan (lihat bawah) |
| DEFERRED | 2 | TC-44, TC-55 — butuh fitur di luar scope crate saat ini |

## Status per kasus

- PASS: TC-01,02,03,04,10,11,14,20,21,23,30,31,32,33,34,36,40,41,43,46,49,51,52,56,57,59,61,62
- ADAPTED (catatan ringkas):
  - TC-05/06 — pemisahan stdout/stderr & EOF adalah perilaku bin mcp_stdio (eprintln); unit menguji padanan
  - TC-12 — tabel v0.1 berharap echo `_meta.serverInfo`; kanonik 2026 tidak mewajibkan echo
  - TC-15 — 100/100 fungsional diuji; bagian RSS-baseline = ukur proses (A7-4, agent8)
  - TC-35 — tidak ada hub_list/hub_inspect (permukaan authoring); est_token = metadata tanpa fetch
  - TC-42 — notasi $node["X"] → E-REF-DANGLING (by-index $('Nama')/E-EXPR-REFERENCE = mesin 215 kasus agent3)
  - TC-45 — 20 skenario sintetis thd scanner prototipe via validate_workflow publik (bukan mesin penuh)
  - TC-47 — patch invalid → isError (SEP-1303), bukan bentuk {applied:false,diagnostics[]}
  - TC-48 — penanda stale = pesan receipt "stale" + content_version naik (H-7b)
  - TC-50 — 14 URI kanonik (tabel v0.1 menulis 13; registri bertambah n8n://nodes/cron/mapping)
  - TC-53/54 — lang tak tersedia → i18n_fallback:true; daftar bahasa lewat n8n://templates/?lang=
  - TC-58 — manifest hub di-serve dgn review_status; tak ada hub_list (draft tak mungkin bocor)
  - TC-60 — 50 injeksi 5 strata: redaksi terbukti di respons patch + 0 jalur eksekusi
  - TC-63 — nama node tak bisa diubah via API publik (patch hanya parameters; credentials ditolak)
  - TC-64 — sapuan seluruh tools + 14 resource: 0 leak oracle (#394/#489/#490)
- GAP-SPEC (butuh keputusan matt/fern; TIDAK diubah kodenya):
  - TC-13 — protocolVersion tak dikenal: kanonik menyerap (coerce) ke PROTOCOL_VERSIONS[0];
    tabel v0.1: tolak. Keputusan: reject (UnsupportedProtocolVersion) atau coerce — sahkan salah satu.
  - TC-22 — klien 2025: tools/list sebelum initialized tidak ditolak (stateless-first);
    tabel v0.1: tolak (lifecycle 2025). Keputusan: tegakkan handshake-2025 atau tetapkan stateless-first
    sbg kontrak (2026) + catat untuk klien 2025.
- DEFERRED:
  - TC-44 — W-EXPR-FLOAT-PRECISION: konstanta ada; detektor milik expression engine (tabel 215 agent3)
  - TC-55 — i18n_rev/content_version dua-poros butuh state versi (integrasi corpus agent2/W4)

## Cara pakai
- Seluruh suite: `cargo test -p mcp` (unit 49 + conformance 52)
- Hanya harness: `cargo test -p mcp --test conformance`
- CLI (opsional, proses nyata): `cargo run -p mcp --bin mcp_stdio` lalu pipe frame (TC-01..06 level proses)

— agent7, 2026-09-09
