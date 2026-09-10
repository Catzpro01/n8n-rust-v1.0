#!/usr/bin/env python3
"""Extend harvest: probe id 261..700, append up to 30 best new unique templates."""
import json, re, ssl, time, urllib.request, hashlib, os

ctx = ssl.create_default_context()
BASE = "https://api.n8n.io/api/workflows/"
DST = "/opt/agent-workspace/docs/corpus"
UA = {"User-Agent": "Mozilla/5.0 agent4-corpus"}

def get(url, tries=4):
    for i in range(tries):
        try:
            req = urllib.request.Request(url, headers=UA)
            with urllib.request.urlopen(req, timeout=40, context=ctx) as r:
                return json.loads(r.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            if e.code in (404, 400):
                return None
            time.sleep(2)
        except Exception:
            time.sleep(2)
    return None

existing = set()
for f in os.listdir(DST):
    if f.endswith(".json"):
        try:
            sig = f.split("-", 2)[2]  # fallback
        except Exception:
            pass
        existing.add(f)
# signatures of already stored (nodes)
import json as _j
stored_sigs = set()
for f in os.listdir(DST):
    if f.endswith(".json") and f.startswith("tpl-"):
        try:
            wf = _j.load(open(os.path.join(DST, f)))
            stored_sigs.add(_j.dumps(wf.get("nodes"), sort_keys=True)[:500])
        except Exception:
            pass
print(f"file tpl existing: {len(existing)}", flush=True)

found = []
for tid in range(261, 701):
    d = get(f"{BASE}{tid}")
    if not d:
        continue
    inner = d.get("data") or {}
    wf = inner.get("workflow") or {}
    nodes = wf.get("nodes")
    if not isinstance(nodes, list) or not nodes:
        continue
    sig = json.dumps(nodes, sort_keys=True)[:500]
    if sig in stored_sigs:
        continue
    stored_sigs.add(sig)
    found.append({"tid": tid, "name": inner.get("name") or wf.get("name") or "",
                  "workflow": wf})
    if tid % 100 == 0:
        print(f"probe... id={tid} | new valid={len(found)}", flush=True)
    time.sleep(0.07)
print(f"new valid unik id 261..700: {len(found)}", flush=True)

DET = {
    "n8n-nodes-base.manualTrigger","n8n-nodes-base.scheduleTrigger","n8n-nodes-base.webhook",
    "n8n-nodes-base.if","n8n-nodes-base.switch","n8n-nodes-base.merge","n8n-nodes-base.wait",
    "n8n-nodes-base.filter","n8n-nodes-base.limit","n8n-nodes-base.sort","n8n-nodes-base.removeDuplicates",
    "n8n-nodes-base.noOp","n8n-nodes-base.splitOut","n8n-nodes-base.aggregate","n8n-nodes-base.set",
    "n8n-nodes-base.code","n8n-nodes-base.httpRequest","n8n-nodes-base.executeWorkflow",
    "n8n-nodes-base.errorTrigger","n8n-nodes-base.respondToWebhook","n8n-nodes-base.loopOverItems",
    "n8n-nodes-base.itemList","n8n-nodes-base.renameKeys","n8n-nodes-base.summarize",
    "n8n-nodes-base.convertToFile","n8n-nodes-base.extractFromFile","n8n-nodes-base.dateTime",
    "n8n-nodes-base.crypto","n8n-nodes-base.compression","n8n-nodes-base.editImage",
    "n8n-nodes-base.rssFeedRead","n8n-nodes-base.readBinaryFile","n8n-nodes-base.writeBinaryFile",
    "n8n-nodes-base.spreadsheetFile","n8n-nodes-base.ftp","n8n-nodes-base.intervalTrigger",
    "n8n-nodes-base.cron","n8n-nodes-base.workflowTrigger","n8n-nodes-base.stopAndError",
}

def slugify(name):
    return re.sub(r"[^a-z0-9]+", "-", (name or "").lower()).strip("-")[:60] or "untitled"

rows = []
for it in found:
    wf = it["workflow"]
    nodes = wf.get("nodes") or []
    types = [str(n.get("type", "")) for n in nodes]
    det = sum(1 for t in types if t in DET)
    rows.append({"tid": it["tid"], "name": it["name"], "nodes": len(nodes),
                 "types": types, "det_ratio": round(det / len(types), 2), "wf": wf})

rows.sort(key=lambda r: (-r["det_ratio"], r["nodes"]))
chosen = rows[:30]

stored_prev = json.load(open("/tmp/corpus2_rows.json")) if os.path.exists("/tmp/corpus2_rows.json") else []
stored = list(stored_prev)
for r in chosen:
    wf = r["wf"]
    raw = json.dumps(wf, ensure_ascii=False).encode("utf-8")
    sha = hashlib.sha256(raw).hexdigest()
    fname = f"tpl-{r['tid']}-{slugify(r['name'])}.json"
    with open(os.path.join(DST, fname), "wb") as f:
        f.write(raw)
    stored.append({"file": fname, "tid": r["tid"], "name": r["name"],
                   "nodes": r["nodes"], "det_ratio": r["det_ratio"],
                   "sha": sha, "bytes": len(raw),
                   "types": ",".join(sorted(set(r["types"])))[:500],
                   "url": f"https://n8n.io/workflows/{r['tid']}"})
json.dump(stored, open("/tmp/corpus2_rows.json", "w"))
print(f"total tersimpan: {len(stored)} (batch2 ini +{len(chosen)})")
for s in sorted(stored, key=lambda x: -x["det_ratio"])[:6]:
    print(" ", s["file"], "| det:", s["det_ratio"], "| nodes:", s["nodes"])
