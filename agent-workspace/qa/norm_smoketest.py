#!/usr/bin/env python3
"""Smoke-test normalizer workflow chain atas subset exec-diff-mvp (13 file nyata).
QA tooling agent4 — tidak mengubah file; hanya validasi & laporan."""
import json, os, sys, hashlib

QA = "/opt/agent-workspace/qa"
SUB = "/opt/agent-workspace/docs/corpus/exec-diff-mvp"
sys.path.insert(0, QA)

from norm_rules_a4 import normalize_workflow, ensure_timezone
from norm_rules_a3 import classify_trigger, should_skip_exec

rows = []
for fn in sorted(os.listdir(SUB)):
    if not fn.endswith(".json"):
        continue
    p = os.path.join(SUB, fn)
    try:
        wf = json.load(open(p))
    except Exception as e:
        rows.append({"file": fn, "status": "PARSE-ERROR", "detail": str(e)[:80]})
        continue
    sha_in = hashlib.sha256(open(p, "rb").read()).hexdigest()[:12]
    try:
        wf2, _ = ensure_timezone(dict(wf), "UTC")
        wf2 = normalize_workflow(wf2)
        # round-trip: serialize ulang harus deterministik
        ser = json.dumps(wf2, sort_keys=True, ensure_ascii=True, separators=(",", ":"))
        sha_out = hashlib.sha256(ser.encode()).hexdigest()[:12]
        kind = classify_trigger(wf2)[0]
        skip, reason = should_skip_exec(wf2)
        rows.append({"file": fn, "status": "OK", "nodes": len(wf2.get("nodes") or []),
                     "trigger": kind, "exec": "SKIP" if skip else "RUN",
                     "sha_in": sha_in, "sha_norm": sha_out})
    except Exception as e:
        rows.append({"file": fn, "status": "NORM-ERROR", "detail": str(e)[:100]})

md = ["# SMOKE-TEST NORMALIZER atas exec-diff-mvp (agent4)",
      "",
      "Uji chain normalisasi workflow (N-20 strip, N-21 timezone) + klasifikasi trigger (N-22) pada 13 file subset nyata.",
      "",
      "| File | Status | Node | Trigger | Exec | sha-in | sha-norm |",
      "|---|---|---|---|---|---|---|"]
ok = err = 0
for r in rows:
    if r["status"] == "OK":
        ok += 1
        md.append(f"| {r['file']} | {r['status']} | {r['nodes']} | {r['trigger']} | {r['exec']} | {r['sha_in']} | {r['sha_norm']} |")
    else:
        err += 1
        md.append(f"| {r['file']} | {r['status']} | - | - | - | - | {r.get('detail','')} |")
md.append("")
md.append(f"Ringkasan: {len(rows)} file | OK {ok} | Error {err}")
out = os.path.join(SUB, "NORMALIZER-SMOKETEST-agent4.md")
open(out, "w").write("\n".join(md) + "\n")
print(f"ok={ok} err={err} total={len(rows)}")
for r in rows:
    if r["status"] != "OK":
        print(" !", r)
