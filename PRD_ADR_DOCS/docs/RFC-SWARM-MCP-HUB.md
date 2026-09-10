# RFC-SWARM-MCP-HUB: Arsitektur Unified Real-Time Rust MCP Swarm Hub
**Status:** PROPOSED & RATIFIED BY PROJECT OWNER (@fern)  
**Target Implementor:** @agent7 (`crates/mcp`)  
**Arsitek Peninjau:** @matt (Lead Architect)  
**Batas Memori:** <10 MB RSS (Bagian dari Invarian <500 MB Hard-Cap)  

---

## 1. MOTIVASI & AKAR MASALAH
Saat ini, koordinasi antar 11 agen SWE bergantung pada polling periodik SQLite (`comm.db`) via `while sleep 10` atau `sleep 30`. Hal ini menimbulkan dua kelemahan sistemik:
1. **Jeda Eksekusi (Execution Delay)**: Ada latency 10–30 detik sebelum sebuah agen menyadari adanya pesan, review, atau tugas baru.
2. **Risiko Timeout**: Saat agen menunggu ulasan atau keputusan arsitektur secara pasif, watchdog timeout atau koneksi klien eksternal berisiko terputus (*idle timeout*).
3. **Bahaya Bloat**: Menjalankan 11 server MCP Python/Node terpisah akan memakan memori >1.1 GB RAM, melanggar batas <500MB mesin produksi.

---

## 2. PRINSIP ARSITEKTUR: SINGLE RUST MULTIPLEXED HUB
Alih-alih 11 server terpisah, dibangun **SATU daemon Rust tunggal** berbasis `crates/mcp` yang bertindak sebagai **Swarm Hub**:
- **Multiplexed Transport**: Mendukung `stdio` (untuk worker lokal), `Unix Domain Socket` (/run/agent-comm/mcp.sock), dan `SSE (Server-Sent Events) / HTTP streaming` untuk worker eksternal.
- **Efisiensi Ekstrem**: Memanfaatkan basis kode `crates/mcp` buatan @agent7 (49/49 test hijau) dengan footprint memori terukur hanya **~3-6 MB RSS**.
- **Single Source of Truth**: Tetap berbasis pada SQLite WAL `/var/lib/agent-comm/comm.db`, tanpa memperkenalkan state store liar.

---

## 3. FITUR KUNCI

### 3.1 Zero-Delay Event Streaming (Push vs Polling)
- Hub memantau commit log SQLite WAL via inotify / channel push.
- Setiap kali pesan baru masuk ke `comm.db`, frame `notifications/message` langsung di-push ke seluruh klien agen yang terhubung dalam <5 milidetik.

### 3.2 Zero-Timeout Keep-Alive
- Hub secara otomatis mengirimkan frame ping `{"jsonrpc": "2.0", "method": "ping"}` setiap 15 detik ke seluruh koneksi aktif.
- Menjaga koneksi TCP/socket tetap hidup dan mencegah agen terputus.

### 3.3 Kontrak MCP Tools Terstandarisasi
Setiap agen berinteraksi melalui tool call ber-skema ketat:
1. `mcp_forum_post`: Mengirim pesan ke channel/recipient tanpa raw bash.
2. `mcp_claim_task`: Mengklaim tiket tugas dengan pembaruan heartbeat atomik.
3. `mcp_submit_review`: Mengirimkan tinjauan kode formal (APPROVE/REJECT + findings).
4. `mcp_heartbeat`: Memperbarui detak jantung tugas secara mandiri.

### 3.4 Jembatan Komunitas (GitHub Bi-directional Sync)
- Terhubung langsung dengan `github_forum_sync.py` untuk menyiarkan pesan publik dan milestone kode ke GitHub Issues #1..#6 secara transparan kepada komunitas pengembang eksternal.

---

## 4. RENCANA IMPLEMENTASI (@agent7)
1. **Fase 1**: Tambahkan binary `crates/mcp/src/bin/mcp_hub.rs` dengan support listener Unix domain socket dan SSE.
2. **Fase 2**: Integrasikan adapter `comm.db` SQLite reader/writer langsung di dalam crate MCP.
3. **Fase 3**: Uji beban (benchmark) untuk memastikan peak RSS tetap <10 MB di bawah 11 koneksi simultan.
