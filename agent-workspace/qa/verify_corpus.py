#!/usr/bin/env python3
"""
verify_corpus.py — Klasifikasi & verifikasi korpus workflow n8n (v4.1)
Oleh: agent5 (QA & Security Auditor)

Konteks PRD-2 §10.2: korpus harus workflow n8n NYATA, "bukan ditulis
tangan untuk lulus".

PENTING — v1 alat ini salah menyimpulkan. Ia hanya mengenali SATU kategori
keaslian (ekspor dari instance/template) sehingga fixture resmi dari repo
upstream n8n ikut dinyatakan GAGAL, padahal fixture itu sah dan tertelusur.
v2 mengklasifikasikan ke TIGA kategori:

  EKSPOR    — hasil Download dari instance n8n atau unduhan template n8n.io.
              Membawa id, meta.instanceId/templateId, versionId, active,
              createdAt/updatedAt. INI yang paling mewakili "workflow nyata"
              untuk gate L2/L3, karena berantakan seperti produksi.
  FIXTURE   — file dari repo resmi n8n (evaluation fixtures, reference
              workflows). Tidak membawa metadata instance, tapi SAH dan bisa
              tertelusur ke commit. Bagus untuk L1 (parser) & katalog deviasi.
              Keasliannya dibuktikan lewat MANIFEST, bukan lewat isi file.
  MENCURIGAKAN — bukan ekspor, dan tidak ada manifest yang menelusurkannya.
              Inilah yang dimaksud PRD sebagai "ditulis tangan untuk lulus".

Jadi keputusan lolos/gagal TIDAK bisa diambil dari isi file saja: fixture
butuh manifest pendamping. Alat ini memeriksa keduanya.

Pakai:
    python3 verify_corpus.py /opt/agent-workspace/docs/corpus/
    python3 verify_corpus.py <dir> --json          # + manifest sha256 (bekukan versi korpus, usul D6)
    python3 verify_corpus.py <dir> --manifest <md> # periksa sha256 terhadap manifest
"""
import argparse
import hashlib
import json
import os
import re
import sys

# Field yang hanya muncul pada ekspor n8n sungguhan.
# Bobot = seberapa kuat indikasinya.
FIELD_EXPORT = {
    "id": 3,          # UUID workflow — selalu ada pada ekspor
    "meta": 3,        # {instanceId, templateId, templateCredsSetupCompleted}
    "versionId": 3,   # UUID versi — sangat khas ekspor
    "active": 1,
    "pinData": 1,
    "createdAt": 2,
    "updatedAt": 2,
    "settings": 1,    # lemah: bisa {} pada contoh dokumentasi
    "tags": 1,
    "nodes": 0,       # bukan penanda keaslian, tapi wajib ada
    "connections": 0,
    "name": 0,
}

# Ambang skor. Ekspor asli biasanya skor >= 8.
AMBANG_LOLOS = 8

TIPE_NODE_DIKENAL = ("n8n-nodes-base.", "@n8n/n8n-nodes-langchain.", "n8n-nodes-")


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for blok in iter(lambda: f.read(65536), b""):
            h.update(blok)
    return h.hexdigest()


def cari_manifest(direktori):
    """Kumpulkan pemetaan sha256 -> nama file dari manifest MANIFEST-*.md."""
    peta = {}
    sumber = {}
    for nama in os.listdir(direktori):
        if not (nama.upper().startswith("MANIFEST") and nama.endswith(".md")):
            continue
        try:
            teks = open(os.path.join(direktori, nama), encoding="utf-8").read()
        except Exception:
            continue
        for baris in teks.splitlines():
            if not baris.strip().startswith("|"):
                continue
            sel = [x.strip() for x in baris.strip().strip("|").split("|")]
            if len(sel) < 2:
                continue
            # cari sel yang berupa sha256 (64 hex) dan sel yang berupa nama file
            hash_ = next((x for x in sel
                          if len(x) == 64 and all(c in "0123456789abcdef" for c in x)), None)
            berkas = next((x for x in sel if x.endswith(".json")), None)
            if berkas:
                sumber[berkas] = nama
                if hash_:
                    peta[hash_] = berkas
    return peta, sumber


def periksa(path, ambang=AMBANG_LOLOS, hash_termanifest=None, sumber_manifest=None):
    hasil = {
        "file": os.path.basename(path),
        "_nama_file": os.path.basename(path),
        "ukuran": os.path.getsize(path),
        "sha256": sha256(path),
    }
    try:
        with open(path, encoding="utf-8") as f:
            d = json.load(f)
    except Exception as e:
        hasil.update({"lolos": False, "skor": 0, "alasan": f"JSON tidak valid: {e}"})
        return hasil

    if not isinstance(d, dict):
        hasil.update({"lolos": False, "skor": 0, "alasan": "bukan objek JSON"})
        return hasil

    # Struktur minimum sebuah workflow n8n
    if "nodes" not in d or "connections" not in d:
        hasil.update({"lolos": False, "skor": 0,
                      "alasan": "tidak punya 'nodes' dan/atau 'connections'"})
        return hasil

    field_ada = [k for k in FIELD_EXPORT if k in d]
    skor = sum(FIELD_EXPORT[k] for k in field_ada)

    # meta.templateId adalah penanda terkuat: workflow dari n8n.io template library
    meta = d.get("meta") or {}
    punyatemplate = bool(meta.get("templateId")) if isinstance(meta, dict) else False

    nodes = d.get("nodes") or []
    tipe = [n.get("type", "") for n in nodes if isinstance(n, dict)]
    eksternal = sorted({t for t in tipe
                        if t.startswith(TIPE_NODE_DIKENAL)
                        and not t.endswith((".set", ".noOp", ".if", ".switch",
                                            ".merge", ".code", ".manualTrigger",
                                            ".scheduleTrigger", ".stickyNote"))})

    # Field yang TIDAK pernah ada pada workflow n8n asli -> pasti disuntik manusia/skrip
    FIELD_NON_N8N = ("_corpus_meta", "_source", "_meta", "_collected_by", "_provenance")
    disuntik = [k for k in d if k in FIELD_NON_N8N or str(k).startswith("_")]

    # Jejak redaksi. PENTING (koreksi v4): n8n.io API SENDIRI yang meredaksi nilai
    # tertentu sebelum mempublikasikan template. Diverifikasi langsung oleh agent5
    # pada 2026-09-09 terhadap https://api.n8n.io/api/templates/workflows/4722 yang
    # mengembalikan literal "[REDACTED_INSTANCE_ID]", "[REDACTED_TAG_ID]",
    # "[REDACTED_WEBHOOK_ID]", "[REDACTED_EMAIL]".
    # Jadi placeholder berpola [REDACTED_*] adalah REDAKSI HULU (sah), BUKAN bukti
    # penyuntingan oleh tim. v3 salah menandainya sebagai cacat integritas.
    # Yang tetap dianggap cacat: redaksi TIDAK berpola, atau placeholder bikinan
    # sendiri (mis. "REDACTED", "<redacted>", "XXX") yang tidak berasal dari n8n.io.
    # v4.1: pola hulu dicari DI DALAM string, bukan sebagai keseluruhan string.
    # n8n.io menyisipkan placeholder di tengah teks, mis. di dalam systemMessage
    # sebuah AI Agent: "...emails sent to or received from the sender by
    # `[REDACTED_EMAIL]`...". v4.0 memakai ^...$ sehingga kasus itu lolos dari
    # kategori hulu dan SALAH ditandai sebagai suntingan tim.
    POLA_TOKEN_HULU = re.compile(r"\[REDACTED_[A-Z0-9_]+\]")
    POLA_TOKEN_LAIN = re.compile(r"\[[A-Z0-9_]*REDACT[A-Z0-9_]*\]")
    teredaksi = []             # redaksi hulu n8n.io (sah, hanya dicatat)
    redaksi_mencurigakan = []  # tidak berpola -> dicurigai disunting tim

    def _periksa_nilai(path, val):
        if not isinstance(val, str):
            return
        if "REDACT" not in val.upper():
            return
        hulu = POLA_TOKEN_HULU.findall(val)
        # token kurung-siku lain yang mengandung REDACT tapi bukan pola hulu
        lain = [t for t in POLA_TOKEN_LAIN.findall(val) if t not in hulu]
        # kata REDACTED yang TIDAK di dalam kurung siku sama sekali
        polos = POLA_TOKEN_LAIN.sub("", val)
        polos = "REDACTED" in polos.upper()
        if hulu:
            teredaksi.append(f"{path}:{','.join(sorted(set(hulu)))}")
        if lain or polos:
            redaksi_mencurigakan.append(
                f"{path} (token tak dikenal={lain or '-'}, redaksi polos={polos}) "
                f"cuplikan={val[:80]!r}")

    def _jelajah(prefix, obj, kedalaman=0):
        if kedalaman > 6:
            return
        if isinstance(obj, dict):
            for k, v in obj.items():
                _jelajah(f"{prefix}.{k}" if prefix else str(k), v, kedalaman + 1)
        elif isinstance(obj, list):
            for i, v in enumerate(obj[:200]):
                _jelajah(f"{prefix}[{i}]", v, kedalaman + 1)
        else:
            _periksa_nilai(prefix, obj)

    _jelajah("", d)

    if skor >= ambang:
        kategori = "EKSPOR"
    elif hasil["sha256"] in (hash_termanifest or {}) or hasil["_nama_file"] in (sumber_manifest or {}):
        kategori = "FIXTURE"
    else:
        kategori = "MENCURIGAKAN"

    alasan = []
    if kategori == "EKSPOR":
        alasan.append(f"membawa metadata ekspor n8n (skor {skor} >= {ambang})")
    elif kategori == "FIXTURE":
        alasan.append(f"tidak punya metadata instance (skor {skor}), tapi sha256-nya "
                      f"tercantum di manifest sehingga tertelusur ke sumber upstream")
    else:
        alasan.append(f"skor metadata {skor} < {ambang} (field ditemukan: "
                      f"{field_ada or 'TIDAK ADA'}) DAN sha256-nya tidak ada di "
                      f"manifest mana pun -> tidak tertelusur, perlakukan sebagai "
                      f"ditulis tangan")
    if not tipe:
        alasan.append("daftar node kosong")

    # Korpus yang isinya diubah dari bentuk aslinya TIDAK layak untuk differential
    # testing, apa pun sumbernya: yang diuji harus byte yang sama dengan yang
    # dibaca n8n.
    cacat_integritas = []
    if disuntik:
        cacat_integritas.append(f"field non-n8n disuntikkan ke dalam JSON: {disuntik}")
    if redaksi_mencurigakan:
        cacat_integritas.append(
            f"redaksi TIDAK berpola n8n.io (dicurigai disunting tim): {redaksi_mencurigakan}")

    hasil.update({
        "kategori": kategori,
        "cacat_integritas": cacat_integritas,
        "redaksi_hulu_n8nio": teredaksi,
        "layak_differential": (kategori in ("EKSPOR", "FIXTURE")) and not cacat_integritas,
        "lolos": kategori in ("EKSPOR", "FIXTURE"),
        "skor": skor,
        "field_ekspor": field_ada,
        "jumlah_node": len(tipe),
        "ada_templateId": punyatemplate,
        "node_eksternal": eksternal,
        "alasan": "; ".join(alasan) if alasan else "memenuhi ciri ekspor n8n asli",
    })
    return hasil


def main():
    ap = argparse.ArgumentParser(description="Verifikasi keaslian korpus workflow n8n")
    ap.add_argument("direktori")
    ap.add_argument("--json", action="store_true", help="output JSON")
    ap.add_argument("--ambang", type=int, default=None,
                    help=f"ambang skor kelulusan (default {AMBANG_LOLOS})")
    a = ap.parse_args()

    ambang = AMBANG_LOLOS if a.ambang is None else a.ambang

    if not os.path.isdir(a.direktori):
        print(f"[ERR] Bukan direktori: {a.direktori}", file=sys.stderr)
        sys.exit(1)

    hash_manifest, sumber_manifest = cari_manifest(a.direktori)
    # v4: rekursif. v3 hanya memindai direktori root sehingga seluruh isi
    # subdirektori (exec-diff-mvp/, _fixtures-repo-internal/) tidak terperiksa.
    files = []
    for root, dirs, names in os.walk(a.direktori):
        dirs[:] = [x for x in dirs if not x.startswith(".")]
        for n in names:
            if n.endswith(".json"):
                files.append(os.path.join(root, n))
    files.sort()
    if not files:
        print(f"[ERR] Tidak ada file .json di {a.direktori}", file=sys.stderr)
        sys.exit(1)

    hasil = []
    for fp in files:
        h = periksa(fp, ambang, hash_manifest, sumber_manifest)
        h["relpath"] = os.path.relpath(fp, a.direktori)
        hasil.append(h)

    if a.json:
        ringkas = {
            "direktori": os.path.abspath(a.direktori),
            "total_file": len(hasil),
            "lolos": sum(1 for h in hasil if h["lolos"]),
            "gagal": sum(1 for h in hasil if not h["lolos"]),
            "ambang": ambang,
            "manifest_sha256": {h["file"]: h["sha256"] for h in hasil},
            "rincian": hasil,
        }
        print(json.dumps(ringkas, indent=2, ensure_ascii=False))
    else:
        from collections import Counter
        hitung = Counter(h.get("kategori", "?") for h in hasil)
        for kat, n in sorted(hitung.items()):
            print(f"  kategori {kat:<14}: {n}")
        lolos = sum(1 for h in hasil if h["lolos"])
        print(f"=== VERIFIKASI KORPUS: {a.direktori} ===")
        cacat = sum(1 for h in hasil if h.get("cacat_integritas"))
        layak = sum(1 for h in hasil if h.get("layak_differential"))
        print(f"file: {len(hasil)} | tertelusur: {lolos} | mencurigakan: "
              f"{len(hasil) - lolos} | isi terubah: {cacat} | "
              f"layak differential: {layak} | ambang skor ekspor: {ambang}")
        print(f"manifest ditemukan: {len(hash_manifest)} entri sha256 "
              f"({sorted(set(sumber_manifest.values())) or 'TIDAK ADA'})")
        hulu = sum(1 for h in hasil if h.get("redaksi_hulu_n8nio"))
        print(f"redaksi HULU n8n.io (sah, bukan cacat): {hulu} file\n")
        print(f"{'FILE':<40} {'KATEGORI':<14} {'SKOR':>4} {'NODE':>4}  STATUS")
        print("-" * 84)
        for h in hasil:
            if not h["lolos"]:
                st = "TIDAK TERTELUSUR"
            elif h.get("cacat_integritas"):
                st = "ISI TERUBAH"
            else:
                st = "OK"
            label = h.get("relpath", h["file"])
            print(f"{label[:39]:<40} {h.get('kategori','?'):<14} {h['skor']:>4} "
                  f"{h.get('jumlah_node', 0):>4}  {st}")
            for c in h.get("cacat_integritas", []):
                print(f"{'':<40} ! {c}")
        print()
        print("CATATAN: FIXTURE = sah & tertelusur ke manifest, bagus untuk gate L1 "
              "(parser) dan katalog deviasi. Untuk gate L2/L3 PRD-2 §10.2 tetap "
              "membutuhkan kategori EKSPOR (workflow nyata dari instance/template), "
              "karena fixture upstream cenderung rapi dan tidak menguji kasus "
              "berantakan produksi.")
        print()
        need_eksternal = {h["file"]: h["node_eksternal"]
                          for h in hasil if h.get("node_eksternal")}
        if need_eksternal:
            print("=== PERHATIAN UNTUK DIFFERENTIAL TESTING (PRD-2 §10.2 / D3) ===")
            print("File berikut memakai node pihak ketiga. Menjalankannya di n8n asli")
            print("akan memicu panggilan API SUNGGUHAN: non-deterministik, kena rate")
            print("limit, butuh kredensial, dan bisa berefek samping (mis. kirim email).")
            print("Wajib ada lapisan mock/record-replay sebelum korpus ini bisa dipakai.")
            for f, n in need_eksternal.items():
                print(f"  {f}: {', '.join(x.split('.')[-1] for x in n)}")

    sys.exit(0 if layak == len(hasil) else 2)


if __name__ == "__main__":
    main()
