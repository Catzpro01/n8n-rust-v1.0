import json, os, hashlib, subprocess, collections

DST = "/opt/agent-workspace/docs/corpus"
rows = []
for f in sorted(os.listdir(DST)):
    if not (f.startswith("tpl-") and f.endswith(".json")):
        continue
    p = os.path.join(DST, f)
    try:
        wf = json.load(open(p))
        nodes = wf.get("nodes") or []
        ntypes = sorted({str(n.get("type", "")) for n in nodes})
        wname = (wf.get("name") or "")[:80]
        nnodes = len(nodes)
    except Exception:
        wname, ntypes, nnodes = "", ["PARSE-ERROR"], 0
    raw = open(p, "rb").read()
    owner = subprocess.run(["stat", "-c", "%U", p], capture_output=True, text=True).stdout.strip()
    rows.append({"file": f, "owner": owner, "name": wname, "nodes": nnodes,
                 "types": ",".join(ntypes)[:250], "sha": hashlib.sha256(raw).hexdigest()})

md = [
    "# MANIFEST CORPUS GABUNGAN LENGKAP - 171 TEMPLATE PUBLIK N8N.IO",
    "",
    "Dibuat: 2026-09-09 (agent4) - regenerasi atas rekomendasi agent2 (pesan #281).",
    "Cakupan: seluruh tpl-*.json di direktori ini. Fixtures repo-internal (12) TIDAK dihitung (subfolder _fixtures-repo-internal/, berlabel).",
    "Sumber: template publik n8n.io via probe id (agent4: 1-700) & probe step-10 id 701-2500 + id besar (agent1).",
    "Kolom owner = penulis unggahan. Sha256 = file apa adanya.",
    "",
    "| File | Owner | Nama | Node | SHA-256 |",
    "|---|---|---|---|---|",
]
for r in rows:
    md.append("| {f} | {o} | {n} | {c} | {s} |".format(f=r["file"], o=r["owner"], n=r["name"], c=r["nodes"], s=r["sha"]))
md.append("")
md.append("Ringkasan per owner:")
cnt = collections.Counter(r["owner"] for r in rows)
for k, v in sorted(cnt.items()):
    md.append("- {}: {} file".format(k, v))
out = os.path.join(DST, "MANIFEST-CORPUS-ALL.md")
open(out, "w").write("\n".join(md) + "\n")
os.chmod(out, 0o644)
print("MANIFEST-CORPUS-ALL.md ditulis ulang:", len(rows), "entri")
for k, v in sorted(cnt.items()):
    print(" ", k, v)
