#!/usr/bin/env python3
"""Runner integrasi NORMALIZER LENGKAP (agent4) — menggabungkan aturan a1+a3+a4.
Mandat #168/#170; desain AGENT4-NORMALIZER-DESIGN.md (#179); modul:
  norm_rules_a1.py (agent1): N-06 orderless, N-10 binary, N-12 error
  norm_rules_a3.py (agent3): N-08 timezone offset, N-09 pairedItem, N-22 trigger
  norm_rules_a4.py (agent4): N-01 contamination, N-03 iso->epoch, N-04 token,
                             N-05 sort, N-07 angka, N-11 null, N-20/21 workflow
Driver resmi diff_harness.py milik agent1 tetap sumber kebenaran runner; file ini
adalah bukti komposabilitas end-to-end + rujukan wiring utk agent1 (tidak
menimpa file milik agent1).
QA tooling, bukan kode produk.
"""
import json
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from norm_rules_a4 import (
    normalize_output, normalize_workflow, canonicalize, ensure_timezone,
)
from norm_rules_a1 import (
    normalize_item_binary, normalize_item_error, sort_items_if_orderless,
)
from norm_rules_a3 import (
    normalize_timezone, normalize_paired_items, classify_trigger,
    should_skip_exec, replace_trigger_with_manual,
)

# Registry node orderless (contoh: hasil agregasi/sortir paralel). Data-driven.
ORDERLESS_REGISTRY = {
    "n8n-nodes-base.aggregate", "n8n-nodes-base.summarize", "n8n-nodes-base.sort",
}

AVAILABLE = ("N-01", "N-03", "N-04", "N-05", "N-06", "N-07", "N-08",
             "N-09", "N-10", "N-11", "N-12", "N-20", "N-21", "N-22")


def norm_run_items(items, node_type=None, strict=False):
    """Chain penuh normalisasi utk satu daftar items output."""
    out = []
    for it in items:
        it = normalize_item_binary(it)          # N-10 (a1)
        it = normalize_item_error(it)           # N-12 (a1)
        it = normalize_output(it, strict_num=strict, strict_null=strict)  # N-01,03,04,05,07,11 (a4)
        it = normalize_timezone(it, all_iso=True)  # N-08 (a3)
        it = normalize_paired_items(it, strict=strict)  # N-09 (a3)
        out.append(it)
    out, _ = sort_items_if_orderless(out, node_type, ORDERLESS_REGISTRY)  # N-06 (a1)
    return out


def norm_workflow_chain(wf, tz="UTC"):
    """Pre-normalisasi workflow (N-20/21/22) sebelum eksekusi diff."""
    wf, _ = ensure_timezone(dict(wf), tz)            # N-21 (a4)
    wf = normalize_workflow(wf)                       # N-20 (a4)
    verdict = classify_trigger(wf)                    # N-22 (a3) info
    skip, reason = should_skip_exec(wf)
    if skip:
        return wf, "SKIP-EXEC", (verdict, reason)
    return wf, "RUNNABLE", verdict


def canon(obj):
    return canonicalize(obj)


def compare_runs(items_a, items_b, node_type=None, strict=False):
    na = norm_run_items(items_a, node_type, strict)
    nb = norm_run_items(items_b, node_type, strict)
    return "PASS" if canon(na) == canon(nb) else "DIFF", na, nb


def selftest():
    fails = []

    def check(cond, label):
        if not cond:
            fails.append(label)

    # A vs B: sama secara semantik, beda segala noise
    A = [{"json": {"_corpus_meta": 1, "total": 1.0, "ok": None,
                   "at": "2026-09-09T04:00:00.000Z", "executionId": "run-1",
                   "bin": {"data": "aGVsbG8=", "fileName": "x.txt"}}}]
    B = [{"json": {"executionId": "run-2", "total": 1, "at": "2026-09-09T05:00:00.000+01:00",
                   "bin": {"fileName": "x.txt", "data": "aGVsbG8="}}}]
    v, na, nb = compare_runs(A, B)
    check(v == "PASS", f"noise harus PASS, dapat {v}")
    # deviasi semantik tetap ketahuan
    C = [{"json": {"total": 2}}]
    v2, _, _ = compare_runs([{"json": {"total": 1}}], C)
    check(v2 == "DIFF", "deviasi 1 vs 2 harus DIFF")
    # workflow: trigger cron -> SKIP-EXEC
    wf_skip = {"name": "w", "nodes": [{"name": "C", "type": "n8n-nodes-base.cron",
                "typeVersion": 1, "parameters": {"triggerTimes": "{}"}}],
               "connections": {}}
    wf2, mode, verdict = norm_workflow_chain(wf_skip)
    check(mode == "SKIP-EXEC", f"cron harus SKIP-EXEC, dapat {mode}")
    wf_ok = {"name": "w2", "nodes": [{"name": "M", "type": "n8n-nodes-base.manualTrigger"}],
             "connections": {}}
    wf3, mode3, _ = norm_workflow_chain(wf_ok)
    check(mode3 == "RUNNABLE", "manual harus RUNNABLE")
    if fails:
        raise AssertionError("\n".join(fails))
    return ("selftest integrasi OK: N-01..N-22 (a1+a3+a4) komposabel, "
            "noise lolos & deviasi terdeteksi")


if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("runA", nargs="?", default=None)
    ap.add_argument("runB", nargs="?", default=None)
    a = ap.parse_args()
    if a.selftest:
        print(selftest())
    elif a.runA and a.runB:
        A = json.load(open(a.runA))
        B = json.load(open(a.runB))
        v, _, _ = compare_runs(A.get("items", A), B.get("items", B))
        print("VERDICT:", v)
        sys.exit(0 if v == "PASS" else 2)
    else:
        print(selftest())
