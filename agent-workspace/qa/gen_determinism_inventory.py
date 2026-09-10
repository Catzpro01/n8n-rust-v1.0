#!/usr/bin/env python3
"""gen_determinism_inventory.py — agent5 (QA).
Memindai korpus n8n untuk sumber nondeterminisme, menulis DETERMINISM-SOURCE-INVENTORY.md.
Deterministik, tanpa network, stdlib saja. Jawaban atas seruan agent4 #343.

REVISI v2 (agent5, menindaklanjuti koreksi TERVERIFIKASI agent4 #384):
  Kategori cron dipecah dari 1 regex mentah menjadi 2 kelas NODE-TYPE, karena
  `cronExpression|triggerTimes|toCronExpression` menggabungkan dua jalur kode n8n yang
  BERBEDA: node cron v1 (n8n-nodes-base.cron, triggerTimes -> toCronExpression detik acak)
  vs scheduleTrigger v2 (n8n-nodes-base.scheduleTrigger, cronExpression). Verifikasi agent5:
  13 file cron-v1 + 1 file scheduleTrigger-cronExpression (tpl-2371), 0 tumpang tindih.
  Split regex TIDAK bersih (cronExpression juga muncul di 3 file cron-v1), jadi klasifikasi
  harus per node-type, bukan per string. Angka kesimpulan besar (eksternal 74%) tidak berubah.
"""
import json, glob, collections, re, hashlib, datetime, sys

C = "/opt/agent-workspace/docs/corpus"
OUT = "/opt/agent-workspace/docs/DETERMINISM-SOURCE-INVENTORY.md"

files = sorted(glob.glob(C + "/tpl-*.json"))

# Kategori berbasis REGEX (dipindai dari string parameter node).
POLA = collections.OrderedDict([
    ("jam/tanggal (Date/$now/luxon)",   r"\$now|\$today|Date\.now|new Date|DateTime|\.toDate\(|luxon"),
    ("Math.random/$random",             r"Math\.random|\$random"),
    ("UUID/nanoid",                      r"\$uuid|randomUUID|uuidv4|nanoid"),
    ("$execution.id (bukan $itemIndex)", r"\$execution\.id|executionId"),
    ("webhookUrl/$resumeWebhook",        r"\$webhook|webhookUrl|resumeWebhookUrl"),
    ("$env/process.env",                 r"\$env\b|process\.env"),
])

# Kategori cron berbasis NODE-TYPE (bukan regex) — koreksi agent4 #384.
CRON_V1 = "cron v1 node (n8n-nodes-base.cron, triggerTimes -> toCronExpression detik acak)"
SCHED_V2_CRON = "scheduleTrigger v2 cronExpression (n8n-nodes-base.scheduleTrigger)"
CRON_V1_TYPE = "n8n-nodes-base.cron"
SCHED_V2_TYPE = "n8n-nodes-base.scheduleTrigger"

EXT = ("httpRequest", "gmail", "slack", "openAi", "googleSheets", "telegram", "googleDrive",
       "webhook", "agent", "lmChat", "rssFeedRead", "ftp", "mongoDb", "airtable", "github", "youTube")
EKSTERNAL = "PANGGILAN EKSTERNAL (node pihak ke-3)"

hf = collections.Counter()
hi = collections.Counter()
for fp in files:
    try:
        d = json.load(open(fp, encoding="utf-8"))
    except Exception:
        continue
    seen = set()
    for n in (d.get("nodes") or []):
        t = (n.get("type") or "")
        text = json.dumps(n.get("parameters", {}), ensure_ascii=False)
        for lab, rx in POLA.items():
            if re.search(rx, text, re.I):
                hi[lab] += 1
                seen.add(lab)
        # cron: klasifikasi per node-type (koreksi agent4 #384)
        if t == CRON_V1_TYPE:
            hi[CRON_V1] += 1
            seen.add(CRON_V1)
        elif t == SCHED_V2_TYPE and "cronExpression" in text:
            hi[SCHED_V2_CRON] += 1
            seen.add(SCHED_V2_CRON)
        if any(e.lower() in t.lower() for e in EXT):
            hi[EKSTERNAL] += 1
            seen.add(EKSTERNAL)
    for s in seen:
        hf[s] += 1

MEK = {
    EKSTERNAL: "**record-replay** (Layer 1a) — sudah dibutuhkan utk differential testing",
    CRON_V1: "registri alias + **aturan detik=0 eksplisit** (DEV-A4; n8n sendiri nondeterministik)",
    SCHED_V2_CRON: "registri alias + **verifikasi upstream dulu** (jalur kode beda; detik acak belum dikonfirmasi)",
}
for k in POLA:
    if "UUID" in k or "random" in k.lower():
        MEK[k] = "**PRNG ter-seed** dari execution_id (proxy global QuickJS)"
    elif "jam" in k:
        MEK[k] = "**jam beku** ke timestamp eksekusi (proxy global QuickJS)"
    else:
        MEK[k] = "**injeksi dari record** (nilai logis direkam saat run asli)"

man = hashlib.sha256()
for fp in files:
    man.update(hashlib.sha256(open(fp, "rb").read()).hexdigest().encode())
versi = man.hexdigest()

L = []
L.append("# DETERMINISM-SOURCE-INVENTORY.md")
L.append("## Inventaris sumber nondeterminisme per korpus — agent5 (QA)")
L.append("")
L.append(f"**Tanggal:** {datetime.datetime.now(datetime.timezone.utc):%Y-%m-%d %H:%M} UTC  ")
L.append(f"**Korpus:** {len(files)} file `tpl-*.json` di root `{C}`  ")
L.append(f"**Versi korpus (sha256 gabungan):** `{versi}`  ")
L.append("**Cara reproduksi:** `python3 /opt/agent-workspace/qa/gen_determinism_inventory.py` "
         "— deterministik, tanpa network, stdlib saja.")
L.append("")
L.append("Ini jawaban atas seruan agent4 (#343): *\"sumber nondeterminisme diinventarisir per node\"*.")
L.append("Angka di bawah = berapa **FILE** yang tersentuh dan berapa **INSTANCE** node, diukur dari")
L.append("korpus nyata, bukan asumsi. Ia menentukan mekanisme determinisme mana yang menutup")
L.append("berapa persen korpus — jadi urutan fase bisa diputuskan dari data, bukan dari selera.")
L.append("")
L.append("> **REVISI v2 (koreksi agent4 #384, terverifikasi agent5):** baris cron dipecah dari 1 regex")
L.append("> mentah menjadi 2 kelas node-type. Regex lama `cronExpression|triggerTimes|toCronExpression`")
L.append("> menggabungkan dua jalur kode n8n yang berbeda. Verifikasi: 13 file cron-v1 + 1 file")
L.append("> scheduleTrigger-cronExpression (`tpl-2371`), 0 tumpang tindih. Split per regex TIDAK bersih")
L.append("> (`cronExpression` juga muncul di 3 file cron-v1), jadi klasifikasi harus per node-type.")
L.append("> Angka kesimpulan besar (eksternal 74%) tidak berubah.")
L.append("")
L.append("| Sumber nondeterminisme | File | Instance | Ditangani oleh |")
L.append("|---|---|---|---|")
for k in sorted(hi, key=lambda x: (-hf[x], x)):
    L.append(f"| {k} | {hf[k]} | {hi[k]} | {MEK.get(k,'—')} |")
L.append("")
L.append("## Cara membaca")
L.append("")
ekst_pct = 100 * hf[EKSTERNAL] // len(files)
L.append(f"- **{hf[EKSTERNAL]}/{len(files)} file ({ekst_pct}%)** punya panggilan eksternal. Ini sumber")
L.append("  nondeterminisme **DOMINAN**, dan ia ditangani lapisan record-replay yang **wajib kita")
L.append("  bangun anyway** untuk differential testing (PRD-2 §10.2). Jadi bukan biaya baru —")
L.append("  ia biaya yang sudah kita tanggung, dan determinisme eksekusi menumpang gratis di atasnya.")
L.append("- Sisanya (jam, random, UUID, cron, execution.id, webhookUrl, env) butuh **proxy global")
L.append("  deterministik** di crate `expr-quickjs`: jam beku + PRNG ter-seed. agent3 mengusulkan")
L.append("  ini di #307/#340; tabel di atas memberi **prioritasnya berdasarkan frekuensi korpus**.")
cron_total = hf[CRON_V1] + hf[SCHED_V2_CRON]
L.append(f"- **Cron (total {cron_total} file unik):** {hf[CRON_V1]} file cron-v1 (jalur `toCronExpression`,")
L.append(f"  detik acak — DEV-A4, n8n nondeterministik) + {hf[SCHED_V2_CRON]} file scheduleTrigger-v2")
L.append("  (`tpl-2371`, cronExpression). Kedua jalur adalah kode n8n yang BERBEDA; apakah scheduleTrigger")
L.append("  juga menyuntik detik acak **belum diverifikasi di upstream** (agent4 #390). Jangan samakan")
L.append("  prioritasnya sebelum jalur v2 dikonfirmasi.")
L.append("")
L.append("## Catatan kejujuran — batasan inventaris ini")
L.append("")
L.append("1. Ini memindai **string parameter**, bukan mengeksekusi expression. `$itemIndex` dan")
L.append("   `$runIndex` **sengaja dikeluarkan** dari kategori `$execution.id` karena keduanya")
L.append("   **deterministik** (posisi item dalam run), sedangkan `$execution.id` tidak. Regex awal")
L.append("   saya mencampur keduanya; saya pisahkan. Kalau tidak, angkanya melebih-lebihkan.")
L.append("2. Node `Code`/`function` bisa memanggil `Date.now()`/`Math.random()` di dalam string JS")
L.append("   yang tidak selalu tertangkap pola di atas. Inventaris ini **batas bawah**, bukan lengkap.")
L.append("3. Urutan iterasi object (K3 di DEVIATION-CATALOG) dan presisi float (K2) adalah sumber")
L.append("   nondeterminisme **antar-runtime** (V8 vs QuickJS), bukan antar-run di runtime yang sama.")
L.append("   Keduanya tidak muncul di tabel ini karena bukan properti korpus — tapi justru keduanya")
L.append("   yang paling mungkin membuat gate L3 gagal. Lihat DEVIATION-CATALOG §6.")
L.append("4. Cron adalah kasus khusus: **n8n sendiri nondeterministik** di jalur v1")
L.append("   (`toCronExpression` menyuntik detik acak — agent4 #264, DEV-A4). Jadi 'rekam dari run")
L.append("   asli' tidak cukup; konversinya harus menetapkan aturan detik yang eksplisit (usul agent5:")
L.append("   kategori `N8N-NONDETERMINISTIC`, detik=0). Jalur v2 (scheduleTrigger cronExpression) belum")
L.append("   dikonfirmasi perilakunya — itu keputusan/verifikasi terpisah, bukan sekadar perekaman.")
L.append("")
L.append("_Ditulis oleh agent5 (QA & Security Auditor). Koreksi dipersilakan — inventaris ini_")
L.append("_batas bawah dan saya nyatakan sendiri di mana ia tidak lengkap. Revisi v2 menyerap_")
L.append("_koreksi terverifikasi agent4 (#384); itu contoh budaya verify-everything yang benar._")

open(OUT, "w", encoding="utf-8").write("\n".join(L))
print("ditulis:", OUT)
print("versi korpus:", versi[:16], "| file:", len(files))
for k in sorted(hi, key=lambda x: (-hf[x], x)):
    print(f"  {hf[k]:3d} file  {hi[k]:4d} inst  {k}")
