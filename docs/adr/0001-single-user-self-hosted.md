# ADR-0001: Target produk single-user self-hosted

**Status:** DITERIMA (keputusan Pemilik, 2026-09-10, ronde grill #1)
**Ganti:** keputusan PRD v0.2 "publik + multi-tenant" (FR-MT-03…07)

## Konteks
PRD v0.2 mengunci produk untuk publik + multi-tenant. Pemilik memutuskan proyek ini
**untuk diri sendiri, self-hosted** di VPS pribadi (2 vCPU/2 GB/50 GB) dan PC 1-core/500 MB.

## Keputusan
- Scope = single-user, satu mesin. Multi-tenant, RBAC multi-pengguna, tim, sharing, mobile: **dipotong dari scope utama**.
- "Pintu arsitektur" dibiarkan: kolom `tenant_id` (default `local`) di skema storage, supaya multi-tenant bisa dibuka lagi tanpa rewrite skema.
- Permukaan keamanan disederhanakan: auth = satu user lokal (password/API key) + bind loopback/LAN/VPN.

## Konsekuensi
- Hemat effort ±1–2 bulan (estimasi PRD v0.2) dialihkan ke target skala (100k–1jt node) & custom node.
- Fitur "tim/marketplace" n8n asli tidak dikejar sebagai parity; lihat ADR-0006 (kecuali Workflow Hub pribadi, fase akhir).
