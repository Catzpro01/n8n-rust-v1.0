# ADR-0005: VPS = runtime; build di sandbox; GitHub = progress per tiket

**Status:** DITERIMA (konfirmasi Pemilik ronde 2, 2026-09-10: "ya")

## Pembagian
| Pihak | Peran |
|---|---|
| **VPS 103.171.85.230** (2 vCPU/2 GB/50 GB) | **Build + runtime produksi**: toolchain Rust + `cargo build` (internet penuh), systemd service, **+ 4 GB swap** (perintah Pemilik: guard anti-OOM, bukan jalan tol), bind + auth single-user (ADR-0001) |
| **Sandbox agent** (2 vCPU/4 GB/20 GB) | Development + editing sumber + commit/push. **Tidak bisa build Rust**: egress sandbox terblokir ke `static.rust-lang.org` & `crates.io` (fakta terukur 2026-09-10; hanya GitHub/npm/PyPI yang terbuka) |
| **GitHub** (branch sesi) | Source of truth + **setiap tiket selesai = commit + push** (progress tersimpan); tag per milestone |

## Alasan
Amendemen 2026-09-10 (fakta egress): sandbox TIDAK bisa mengunduh toolchain/crates.

Amendemen 2 (2026-09-10, keputusan Pemilik: "bagaimana jika langsung di github saja"):
**Mesin build+test = GitHub Actions runner** (2 vCPU/7 GB, internet penuh), dipicu push
ke branch `arena/**` (`.github/workflows/rust-ci.yml`), hasil build+test dikembalikan
sebagai file di branch `ci-results` (dibaca agent via git). **VPS = runtime saja**
(systemd + 4 GB swap). Loop: agent edit → push per tiket → Actions build+test →
agent baca `ci-results` → fix → push. Deploy ke VPS dilakukan saat akses dibuka
(atau via job deploy setelah Security Group mengizinkan rentang IP GitHub).

## Kondisi
Akses SSH agent ke VPS saat ini **terblokir firewall PERSISTEN** (tetap reset pre-banner
port 22 & 2222 bahkan setelah VPS reboot 2026-09-10 → bukan fail2ban; kemungkinan aturan
server ufw/nftables atau firewall panel idcloudhost). Pemilik sudah menjalankan 4 perintah
console-web sebagai user `fern` (key `user@MDMTEST` + NOPASSWD sudo untuk fern — itu untuk
perangkat Pemilik, bukan akses agent). Yang dibutuhkan: whitelist IP egress sandbox agent
(bisa dilihat di log VPS) atau buka port 22 dari panel firewall idcloudhost.
