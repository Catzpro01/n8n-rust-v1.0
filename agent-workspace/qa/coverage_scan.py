#!/usr/bin/env python3
"""Node coverage analysis of real corpus vs MVP node list (PRD-2 7.3)."""
import json, os, collections, hashlib

DST = "/opt/agent-workspace/docs/corpus"

T1 = {"n8n-nodes-base.manualTrigger","n8n-nodes-base.scheduleTrigger","n8n-nodes-base.webhook",
      "n8n-nodes-base.httpRequest","n8n-nodes-base.set","n8n-nodes-base.if","n8n-nodes-base.switch",
      "n8n-nodes-base.merge","n8n-nodes-base.code","n8n-nodes-base.noOp","n8n-nodes-base.executeWorkflow",
      "n8n-nodes-base.errorTrigger","n8n-nodes-base.respondToWebhook"}
T2 = {"n8n-nodes-base.filter","n8n-nodes-base.limit","n8n-nodes-base.sort","n8n-nodes-base.splitOut",
      "n8n-nodes-base.aggregate","n8n-nodes-base.wait","n8n-nodes-base.removeDuplicates",
      "n8n-nodes-base.executeCommand"}
T3 = {"n8n-nodes-base.postgres","n8n-nodes-base.gmail","n8n-nodes-base.googleSheets",
      "n8n-nodes-base.slack","n8n-nodes-base.telegram"}
MVP = T1 | T2 | T3

counter = collections.Counter()
files = [f for f in os.listdir(DST) if f.startswith("tpl-") and f.endswith(".json")]
rows = []
for f in files:
    try:
        wf = json.load(open(os.path.join(DST, f)))
        nodes = wf.get("nodes") or []
    except Exception:
        continue
    types = [str(n.get("type", "?")) for n in nodes]
    counter.update(types)
    unsup = sorted({t for t in types if t not in MVP})
    rows.append({"file": f, "n": len(types), "unsup": unsup})

total_files = len(rows)
full_ok = [r for r in rows if not r["unsup"]]
t1_ok = [r for r in rows if all(t in T1 for t in set(sum([c["types"] for c in []], [])) )]  # placeholder unused

# files whose every node is in T1
def only(rs, S):
    out = []
    for f_ in files:
        try:
            wf = json.load(open(os.path.join(DST, f_)))
            ts = [str(n.get("type", "?")) for n in (wf.get("nodes") or [])]
        except Exception:
            continue
        if ts and all(t in S for t in ts):
            out.append(f_)
    return out

only_t1 = only(files, T1)
only_mvp = only(files, MVP)

print(f"file tpl dianalisis: {total_files}")
print(f"file yg seluruh node-nya di MVP (26): {len(only_mvp)} ({round(100*len(only_mvp)/max(total_files,1))}%)")
print(f"file yg seluruh node-nya di Tier1 (13): {len(only_t1)} ({round(100*len(only_t1)/max(total_files,1))}%)")
print("\nTop 30 tipe node:")
for t, c in counter.most_common(30):
    flag = "MVP" if t in MVP else ("T1" if t in T1 else "LUAR")
    print(f"  {c:4d}  {t}  [{flag}]")

sup_counts = collections.Counter()
for r in rows:
    for u in r["unsup"]:
        sup_counts[u] += 1
print("\nNode LUAR MVP paling sering (jumlah file kena):")
for t, c in sup_counts.most_common(25):
    print(f"  {c:4d}  {t}")
