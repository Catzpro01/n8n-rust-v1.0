#!/usr/bin/env python3
"""Uji verify_corpus.py v4 pada 3 positif + 3 negatif SEBELUM dipakai menuduh siapa pun.
Pelajaran ERR-021: alat verifikasi wajib diuji dulu pada kasus yang sudah diketahui
jawabannya. Dijalankan DI VPS sebagai agent5."""
import json, os, shutil, subprocess, sys, tempfile, hashlib

VC = "/opt/agent-workspace/qa/verify_corpus.py"
C = "/opt/agent-workspace/docs/corpus"
gagal = 0

def jalankan(d):
    p = subprocess.run(["python3", VC, d], capture_output=True, text=True, timeout=300)
    return p.returncode, p.stdout, p.stderr

def cek(nama, syarat, keterangan=""):
    global gagal
    tanda = "LULUS " if syarat else "GAGAL "
    if not syarat:
        gagal += 1
    print(f"  [{tanda}] {nama} {keterangan}")

tmp = tempfile.mkdtemp(prefix="uji-vc4-")

# ============================ POSITIF 1: tpl-4722 (redaksi HULU n8n.io, sah)
d1 = os.path.join(tmp, "p1"); os.makedirs(d1)
src4722 = os.path.join(C, "tpl-4722-gmail-ai-email-manager.json")
shutil.copy2(src4722, d1)
raw = open(src4722, encoding="utf-8").read()
man = open(os.path.join(d1, "MANIFEST-UJI.md"), "w", encoding="utf-8")
man.write("| File | SHA-256 |\n|---|---|\n")
man.write(f"| tpl-4722-gmail-ai-email-manager.json | {hashlib.sha256(raw.encode()).hexdigest()} |\n")
man.close()
rc, out, err = jalankan(d1)
print("\nPOSITIF 1 - tpl-4722, redaksi [REDACTED_INSTANCE_ID] berasal dari n8n.io API")
cek("tidak lagi dinyatakan ISI TERUBAH", "ISI TERUBAH" not in out)
cek("dinyatakan OK", "\n" in out and " OK\n" in out.replace("\r", ""))
cek("redaksi hulu terhitung 1 file", "redaksi HULU n8n.io (sah, bukan cacat): 1 file" in out)
cek("exit code 0 (layak semua)", rc == 0, f"(rc={rc})")

# ============================ NEGATIF 1: suntik _corpus_meta
d2 = os.path.join(tmp, "n1"); os.makedirs(d2)
d = json.loads(raw)
d["_corpus_meta"] = {"sumber": "n8n.io", "diunduh": "2026-09-09"}
open(os.path.join(d2, "tpl-4722-gmail-ai-email-manager.json"), "w").write(json.dumps(d))
shutil.copy2(os.path.join(d1, "MANIFEST-UJI.md"), d2)
rc, out, err = jalankan(d2)
print("\nNEGATIF 1 - field _corpus_meta disuntikkan ke dalam JSON")
cek("terdeteksi sebagai ISI TERUBAH", "ISI TERUBAH" in out)
cek("menyebut field yang disuntik", "_corpus_meta" in out)
cek("exit code 2 (tidak layak)", rc == 2, f"(rc={rc})")

# ============================ NEGATIF 2: redaksi bikinan sendiri (non-pola n8n.io)
d3 = os.path.join(tmp, "n2"); os.makedirs(d3)
d = json.loads(raw)
d["meta"]["instanceId"] = "REDACTED"          # tanpa kurung siku -> bukan pola n8n.io
open(os.path.join(d3, "tpl-4722-gmail-ai-email-manager.json"), "w").write(json.dumps(d))
shutil.copy2(os.path.join(d1, "MANIFEST-UJI.md"), d3)
rc, out, err = jalankan(d3)
print("\nNEGATIF 2 - nilai 'REDACTED' polos (bukan pola [REDACTED_*] n8n.io)")
cek("terdeteksi sebagai ISI TERUBAH", "ISI TERUBAH" in out)
cek("disebut redaksi mencurigakan", "dicurigai disunting tim" in out)
cek("exit code 2", rc == 2, f"(rc={rc})")

# ============================ POSITIF 2: subdirektori ikut terpindai (rekursif)
rc, out, err = jalankan(C)
print("\nPOSITIF 2 - pemindaian rekursif atas korpus nyata")
n_root = len([f for f in os.listdir(C) if f.endswith(".json")])
n_all = sum(len([f for f in fs if f.endswith(".json")]) for _, _, fs in os.walk(C))
m = [l for l in out.splitlines() if l.startswith("file: ")]
print("   ", m[0] if m else "(ringkasan tidak ditemukan)")
cek(f"jumlah file yang dipindai = {n_all} (bukan {n_root})",
    bool(m) and m[0].startswith(f"file: {n_all} "), f"(root={n_root}, total={n_all})")
cek("isi exec-diff-mvp/ muncul di laporan", "exec-diff-mvp/" in out)
cek("isi _fixtures-repo-internal/ muncul di laporan", "_fixtures-repo-internal/" in out)

# ============================ POSITIF 3: exec-diff-mvp bersih dari cacat integritas
blok = [l for l in out.splitlines() if l.strip().startswith("exec-diff-mvp/")]
cacat = [l for l in blok if "! " in l]
print("\nPOSITIF 3 - 13 file exec-diff-mvp milik agent4 (sudah saya verifikasi byte-per-byte)")
cek("13 file terpindai", len(blok) >= 13, f"(ketemu {len(blok)})")
cek("tidak ada cacat integritas", not cacat, str(cacat[:2]))

# ============================ NEGATIF 3: file tanpa penelusuran
d4 = os.path.join(tmp, "n3"); os.makedirs(d4)
open(os.path.join(d4, "karangan-bebas.json"), "w").write(json.dumps({
    "name": "Karangan", "nodes": [{"name": "A", "type": "n8n-nodes-base.set",
                                    "parameters": {}, "typeVersion": 1, "position": [0, 0]}],
    "connections": {}}))
rc, out, err = jalankan(d4)
print("\nNEGATIF 3 - file karangan tanpa metadata & tanpa manifest")
cek("diklasifikasikan MENCURIGAKAN", "MENCURIGAKAN" in out)
cek("exit code 2", rc == 2, f"(rc={rc})")

shutil.rmtree(tmp, ignore_errors=True)
print("\n" + "=" * 70)
print("HASIL: " + ("SEMUA UJI LULUS" if gagal == 0 else f"{gagal} UJI GAGAL"))
sys.exit(1 if gagal else 0)
