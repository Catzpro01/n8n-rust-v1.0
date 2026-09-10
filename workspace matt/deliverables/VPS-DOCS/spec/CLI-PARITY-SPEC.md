# CLI-PARITY-SPEC v1.0 — Kontrak Paritas CLI n8n

**Target paritas: n8n `2.39.0`** (terverifikasi dari `packages/cli/package.json` di clone upstream)
**Status: KANONIK.** Dokumen ini adalah kontrak. Perubahan butuh ruling Lead Architect.
**Disusun oleh: matt (Lead Architect), 2026-09-09.**

Perintah pemilik produk: *"clinya harus sama persis."* Dokumen ini mendefinisikan apa arti "sama persis"
secara terukur, bukan secara kesan.

---

## 0. TUGAS NOL — selamatkan clone upstream (belum durabel)

Seluruh bukti di dokumen ini diambil dari **`/tmp/n8n-upstream`**. `/tmp` **tidak durabel** — bisa hilang
kapan saja saat reboot atau pembersihan.

```bash
mkdir -p /opt/agent-workspace/upstream && \
cp -r /tmp/n8n-upstream /opt/agent-workspace/upstream/n8n-2.39.0 && \
cd /opt/agent-workspace/upstream/n8n-2.39.0 && git rev-parse HEAD
```
Clone yang sudah ada di `/mnt/extra-storage/agent1-work/n8n-upstream` adalah **sparse checkout** dan
**tidak memuat `packages/cli/src/commands/`** — jadi tidak bisa dipakai untuk pekerjaan ini.

**Setelah disalin, catat sha commit-nya di dokumen ini.** Semua klaim paritas harus terikat pada sha itu.

---

## 1. Daftar perintah — 24, diekstrak dari sumber, bukan dari dokumentasi

Metode ekstraksi (dapat diulang):
```bash
C=/opt/agent-workspace/upstream/n8n-2.39.0/packages/cli/src/commands
grep -rh "@Command(" $C --include="*.ts" -A1 | grep -oE "name: '[a-z:_-]+" | sort -u
```

Hasil terukur pada n8n 2.39.0:

| # | Perintah | File sumber |
|---|---|---|
| 1 | `audit` | `audit.ts` |
| 2 | `db:revert` | `db/revert.ts` |
| 3 | `execute` | `execute.ts` |
| 4 | `execute-batch` | `execute-batch.ts` |
| 5 | `export:credentials` | `export/credentials.ts` |
| 6 | `export:entities` | `export/entities.ts` |
| 7 | `export:nodes` | `export/nodes.ts` |
| 8 | `export:workflow` | `export/workflow.ts` |
| 9 | `import:credentials` | `import/credentials.ts` |
| 10 | `import:entities` | `import/entities.ts` |
| 11 | `import:workflow` | `import/workflow.ts` |
| 12 | `ldap:reset` | `ldap/reset.ts` |
| 13 | `license:clear` | `license/clear.ts` |
| 14 | `license:info` | `license/info.ts` |
| 15 | `list:workflow` | `list/workflow.ts` |
| 16 | `mfa:disable` | `mfa/disable.ts` |
| 17 | `publish:workflow` | `publish/workflow.ts` |
| 18 | `start` | `start.ts` |
| 19 | `ttwf:generate` | `ttwf/generate.ts` |
| 20 | `unpublish:workflow` | `unpublish/workflow.ts` |
| 21 | `update:workflow` | `update/workflow.ts` |
| 22 | `user-management:reset` | `user-management/reset.ts` |
| 23 | `webhook` | `webhook.ts` |
| 24 | `worker` | `worker.ts` |

Plus `base-command.ts` (basis semua perintah) dan `ttwf/worker-pool.ts`.
Ada juga `community-node` — berkas tesnya ada (`__tests__/community-node.test.ts`) tapi tidak muncul dari
grep `@Command` di direktori ini. **Cari di mana ia terdaftar sebelum menyimpulkan ia tidak ada.**

### Temuan penting: dokumentasi resmi TIDAK LENGKAP

Halaman docs `deploy/host-n8n/configure-n8n/use-the-command-line` hanya mendokumentasikan sekitar 18
perintah. **`export:nodes`, `list:workflow`, `execute-batch`, `ttwf:generate`, `db:revert`, `webhook`,
`worker` tidak ada di halaman itu.**

> **Konsekuensi: SUMBER adalah authoritative, bukan dokumentasi.** Siapa pun yang membangun paritas dari
> halaman docs akan menghasilkan CLI yang kurang perintah. Ini persis kelas kesalahan yang sudah kita alami
> sepuluh kali hari ini: mengutip deskripsi, bukan artefak.

### Perubahan merusak n8n 2.0 yang WAJIB diikuti

Di n8n 2.0, **toggle active/inactive digantikan model publish/unpublish**:
- `publish:workflow --id=<ID> [--versionId=<V>]` — **tidak punya flag `--all`**, dan itu disengaja
  (mencegah publikasi massal tidak sengaja di produksi)
- `unpublish:workflow --id=<ID> | --all`
- `update:workflow --id=<ID> --active=true|false` — **DEPRECATED sejak 2.0, akan dihapus**

**Kita meniru 2.39.0: `publish`/`unpublish` adalah jalan utama, `update:workflow` tetap ada tapi ditandai
deprecated.** Jangan membangun model active/inactive lama.

---

## 2. Pola definisi flag di 2.39.0 — zod `flagsSchema`

n8n 2.39.0 **tidak** mendefinisikan flag dengan decorator per-flag. Polanya (contoh nyata `execute.ts:17-28`):

```ts
const flagsSchema = z.object({
	id: z.string().describe('id of the workflow to execute').optional(),
	rawOutput: z.boolean().describe('Outputs only JSON data, with no other text').optional(),
	/**@deprecated */
	file: z.string().describe('DEPRECATED: Please use --id instead').optional(),
});

@Command({
	name: 'execute',
	description: 'Executes a given workflow',
	flagsSchema,
})
export class Execute extends BaseCommand<z.infer<typeof flagsSchema>> { ... }
```

**Resep ekstraksi flag untuk setiap perintah:**
```bash
sed -n '/flagsSchema = z.object/,/^});/p' $C/<file>.ts
```
Itu memberi nama flag, tipe (`z.string()`/`z.boolean()`/`z.number()`), keterangan dari `.describe()`,
keopsionalan (`.optional()`), dan penanda `@deprecated`.

**Setiap flag yang diekstrak harus dicatat ke tabel paritas (bagian 4) dengan `verified(file:line)`.**
Tanpa itu, klaim "flag sudah sama" tidak bisa diperiksa.

### Flag yang terverifikasi untuk `execute` (contoh isian tabel)

| Flag | Tipe | Keterangan (dari `.describe()`) | Sumber |
|---|---|---|---|
| `--id` | string, opsional | id of the workflow to execute | `execute.ts:18` |
| `--rawOutput` | boolean, opsional | Outputs only JSON data, with no other text | `execute.ts:19` |
| `--file` | string, opsional, **DEPRECATED** | DEPRECATED: Please use --id instead | `execute.ts:20-21` |

Perilaku terukur dari `execute.ts`:
- `:49` bila `--id` tidak ada → error
- `:54` bila `--file` dipakai → jalur deprecated
- `:128` `this.logger.info(JSON.stringify(data, null, 2))` — **indentasi 2 spasi**
- `:137` bila `rawOutput === undefined` → ada cabang keluaran berbeda
- `:141` `this.log(JSON.stringify(data, null, 2))`
- `:149` `if (error instanceof ExecutionBaseError) this.logger.error(error.description!)`

> **Detail seperti "indentasi 2 spasi" dan "logger.error memakai `error.description`" adalah bagian dari
> "sama persis".** Keluaran yang berbeda spasi putihnya bukan keluaran yang sama.

---

## 3. Definisi "sama persis" — lima lapisan, semua terukur

| Lapis | Apa yang dibandingkan | Cara menguji |
|---|---|---|
| **L1 Nama & struktur** | 24 nama perintah, bentuk `grup:sub` (`export:workflow`), keberadaan `--help`/`--version` | bandingkan daftar `--help` puncak |
| **L2 Flag per perintah** | nama flag, tipe, opsional/wajib, alias pendek (`-o` untuk `--output`), penanda deprecated | tabel paritas (bagian 4) dengan `verified(file:line)` |
| **L3 Teks bantuan** | urutan bagian, kalimat description, format tabel flag | diff `n8n <cmd> --help` vs `n8n-rust <cmd> --help` |
| **L4 Keluaran & kode keluar** | bentuk JSON, indentasi, urutan kunci, pesan error, **exit code** | uji snapshot per perintah |
| **L5 Efek** | apa yang berubah di DB / filesystem setelah perintah jalan | uji integrasi terhadap SQLite |

**L3 butuh keputusan desain, dan saya putuskan di sini:**

Teks bantuan n8n dihasilkan oleh kerangka `@oclif`/decorator n8n, jadi format persisnya (lebar kolom,
simbol, warna) adalah keluaran kerangka, bukan sesuatu yang ditulis tangan di tiap perintah.

> **Putusan: L1, L2, L4, L5 wajib sama persis. L3 wajib sama secara SEMANTIK — kalimat description identik,
> flag dan keterangannya identik dan berurutan sama — tetapi tata letak kerangka boleh berbeda selama
> setiap informasi yang sama hadir.**

Alasan: meniru byte-per-byte keluaran bantuan oclif berarti menulis ulang oclif, yang tidak menambah nilai
bagi pemilik produk dan akan pecah setiap kali n8n naik versi kerangka. **Yang membuat CLI "terasa sama"
adalah perintahnya, flag-nya, keluarannya, dan efeknya — bukan jumlah spasi di tabel bantuan.**

Bila pemilik produk menghendaki L3 byte-identical juga, itu keputusan yang harus dinyatakan eksplisit, dan
konsekuensinya harus disebut: pekerjaan bertambah dan rapuh terhadap versi kerangka n8n.

---

## 4. Tabel paritas — artefak yang harus diisi oleh pemilik lane CLI

Satu baris per flag per perintah. **Tidak ada baris tanpa `verified(file:line)`.**

```
| perintah | flag | tipe | wajib? | alias | deprecated | keterangan | sumber verified | status kita |
```

Contoh baris terisi:
```
| execute | --id | string | tidak | - | tidak | id of the workflow to execute | execute.ts:18 | BELUM |
```

**Kolom `status kita` hanya boleh berisi: `BELUM`, `SEBAGIAN`, `SELESAI+sha`, `TIDAK-RELEVAN+alasan`.**
`TIDAK-RELEVAN` wajib beralasan — contoh sah: `license:clear` dan `license:info` bila kita tidak
mengimplementasikan lisensi berbayar; `worker` dan `webhook` bila mode queue tidak dibangun untuk
single-instance.

> **Peringatan: `TIDAK-RELEVAN` adalah tempat klaim palsu bersembunyi.** Setiap baris `TIDAK-RELEVAN`
> harus disetujui Kritikus (lihat REPLAN-BACKEND-MATANG.md §C).

---

## 5. Prioritas pengerjaan — jalur terpendek ke "bisa dipakai sendiri"

Pemilik produk ingin **merasakan n8n-rust langsung**. Maka urutannya bukan abjad, bukan nomor, tapi
berguna-lebih-dulu:

**Tahap 1 — bisa menjalankan satu workflow (tanpa ini, tidak ada yang bisa dirasakan):**
1. `start` — server hidup, port default, data dir default
2. `import:workflow --input=<file>` — muat workflow JSON n8n asli
3. `list:workflow` — lihat yang termuat
4. `execute --id=<ID> [--rawOutput]` — jalankan, lihat hasil
5. `export:workflow --id=<ID> --output=<file>` — keluarkan lagi

**Tahap 2 — bisa mengelola:**
6. `publish:workflow` / `unpublish:workflow`
7. `import:credentials` / `export:credentials`
8. `update:workflow` (deprecated, tapi harus ada agar impor skrip lama tidak pecah)
9. `audit`

**Tahap 3 — sisanya atau `TIDAK-RELEVAN` beralasan:**
10. `execute-batch`, `export:entities`, `import:entities`, `export:nodes`, `db:revert`,
    `user-management:reset`, `mfa:disable`, `ldap:reset`, `license:*`, `community-node`, `ttwf:generate`,
    `worker`, `webhook`

**Catatan untuk Tahap 3 yang berhubungan dengan produk single-instance:**
- `worker` dan `webhook` adalah perintah **mode queue** (multi-main). Produk kita self-hosted
  single-instance, jadi keduanya kandidat `TIDAK-RELEVAN` — **tapi harus diputuskan eksplisit, bukan
  dilupakan.**
- `license:*` — kita tidak punya lisensi berbayar. Kandidat `TIDAK-RELEVAN` atau stub yang menjelaskan.
- `ldap:reset`, `mfa:disable`, `user-management:reset` — bergantung pada `auth`. Bila `auth` minimal,
  ketiganya mengikuti.

---

## 6. Uji paritas yang harus ada sebelum klaim "sama persis"

**Wajib ada, dan wajib bisa gagal:**

1. **Uji daftar perintah.** Ambil keluaran `--help` puncak kita, ekstrak nama perintah, bandingkan dengan
   daftar 24 di §1 dikurangi yang `TIDAK-RELEVAN` beralasan. Gagal bila ada yang hilang.
2. **Uji flag per perintah Tahap 1.** Untuk tiap perintah Tahap 1, bandingkan himpunan flag kita dengan
   tabel paritas. Gagal bila ada flag hilang atau tipe berbeda.
3. **Uji snapshot keluaran `execute --rawOutput`.** Emas: JSON dengan indentasi 2 spasi, urutan kunci
   mengikuti n8n. **Wajib ada satu mutan (disiplin 50e) yang membuktikan snapshot ini bisa gagal** — ubah
   indentasi jadi 4, test harus merah.
4. **Uji kode keluar.** `execute` tanpa `--id` → exit code bukan 0 dan pesan error yang sesuai.
   `execute --id=<tidak-ada>` → exit code bukan 0.
5. **Uji bulat (round-trip).** `export:workflow --id=X --output=a.json` lalu `import:workflow --input=a.json`
   harus menghasilkan workflow yang setara. **Uji ini yang paling penting** karena inilah yang akan dilakukan
   pemilik produk pertamakali dengan file n8n aslinya.
6. **Uji interoperabilitas dengan n8n asli.** Ambil workflow JSON hasil ekspor dari n8n 2.39.0 asli, impor
   ke kita, jalankan. **Ini ujian sebenarnya dari "seperti menginstall n8n asli".**

> **Semua uji di atas harus punya mutan yang membuktikan ia bisa gagal.** Uji yang tidak pernah gagal bukan
> uji. Itu pelajaran yang sudah dibayar mahal hari ini.

---

## 7. Yang BUKAN bagian paritas CLI

Supaya tidak ada yang membangun hal yang tidak diminta:

- **UI web.** Pemilik produk akan membangunnya sendiri di Arena dengan mengimpor berkas kita. **Tugas kita
  hanya backend + CLI.** Jangan ada yang mulai membuat frontend.
- **API surface untuk UI** tetap harus dibangun (crate `api`), karena UI Arena akan memanggilnya — tapi itu
  kontrak HTTP, bukan CLI.
- **Mode queue / multi-main / horizontal scaling.** Produk single-instance.
- **Skala besar.** Perintah pemilik produk: *"jangan diperlambat untuk yang skala lebih besar."*

---

*Dokumen ini kanonik. Bila sumber n8n versi berikutnya berbeda, naikkan versi target lewat ruling, jangan
diam-diam mengubah dokumen ini.*


---

## DIVERGENSI SADAR (disetujui Pemilik Produk, bukan drift)

Bagian ini mencatat tempat kita **sengaja berbeda** dari n8n asli. Setiap entri punya tanggal, keputusan, dan
alasan. Kalau sebuah perbedaan tidak terdaftar di sini, ia adalah **cacat paritas**, bukan pilihan.

### D-1 · Kode keluar saat node gagal

| | |
|---|---|
| **Diputuskan** | 2026-09-10, oleh Pemilik Produk |
| **Perilaku n8n asli** | Tidak mengubah kode keluar. Di `execute.ts` satu-satunya `process.exit(1)` adalah untuk workflow-id-tidak-ditemukan (baris 67–70). Jalur node-gagal (124–137) melempar error yang ditangkap `catch` sendiri (144–151) yang hanya mencatat ke log — tanpa rethrow, tanpa exit. Proses berakhir normal; pembeda satu-satunya adalah teks keluaran. |
| **Perilaku kita** | **Kode keluar tidak-nol saat sebuah node gagal.** |
| **Alasan** | Pemakai menjalankan ini lewat skrip dan cron. `exit 0` pada kegagalan membuat kegagalan tidak terdeteksi oleh apa pun di luar teks. Paritas di titik ini berarti menyalin cacat. |
| **Bukti fakta** | Ditemukan agent9 (#1723), diverifikasi independen agent1 (#1724) terhadap `upstream/n8n-2.39.0` anchor `fcf21f5e`. |
| **Konsekuensi** | Skrip yang ditulis untuk n8n asli dan mengandalkan `exit 0` akan tetap "berhasil" di n8n-rust saat workflow gagal. Itu perubahan perilaku yang terlihat, dan disengaja. |

**Aturan untuk entri berikutnya:** divergensi hanya boleh ditambahkan oleh Pemilik Produk atau atas keputusan
yang beliau setujui. Agen tidak boleh menambahkannya sendiri — itu jalur terpendek mengubah drift menjadi
"kebijakan".
