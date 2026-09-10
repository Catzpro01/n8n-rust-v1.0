#!/usr/bin/env python3
"""Differential harness driver v0.3 (agent1) — bandingkan output run A vs run B.

Mandat: fern #168/#170. Desain: AGENT4-NORMALIZER-DESIGN.md (#179).
Wiring: mengikuti diff_run_full.py agent4 (bukti komposabilitas) — driver ini
menggantikannya sebagai runner resmi; chain sama persis agar verdict konsisten.
QA tooling (seperti verify_corpus.py), BUKAN kode produk Rust.

Aturan: N-01..N-12 + N-20..N-22 dari modul norm_rules_a1/a3/a4.
Modul yang hilang -> aturan pemiliknya DITOLAK eksplisit (NormRuleMissing),
tidak pernah diam-diam. Default strict=False mengikuti wiring referensi
agent4; default FINAL menunggu NORMALIZATION-MANIFEST.md (agent5).

Exit code (konvensi ERR-007): 0=PASS/RUNNABLE, 2=DIFF/SKIP-EXEC, 1=ERROR.

Penggunaan:
  python3 diff_harness.py runA.json runB.json [--strict] [--orderless T ...]
  python3 diff_harness.py --check-workflow wf.json
  python3 diff_harness.py --selftest
"""

import difflib
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from norm_rules_a1 import (
    NormRuleMissing,
    normalize_item_binary,
    normalize_item_error,
    rule_owner,
    sort_items_if_orderless,
)

try:
    from norm_rules_a4 import normalize_output as a4_normalize_output
    from norm_rules_a4 import normalize_workflow as a4_normalize_workflow
    from norm_rules_a4 import canonicalize as a4_canonicalize
    from norm_rules_a4 import ensure_timezone as a4_ensure_timezone
    HAVE_A4 = True
except ImportError:
    HAVE_A4 = False

try:
    from norm_rules_a3 import normalize_timezone as a3_normalize_timezone
    from norm_rules_a3 import normalize_paired_items as a3_normalize_paired
    from norm_rules_a3 import classify_trigger as a3_classify_trigger
    from norm_rules_a3 import should_skip_exec as a3_should_skip_exec
    HAVE_A3 = True
except ImportError:
    HAVE_A3 = False

A1_RULES = ("N-06", "N-10", "N-12")
A4_RULES = ("N-01", "N-03", "N-04", "N-05", "N-07", "N-11", "N-20", "N-21")
A3_RULES = ("N-08", "N-09", "N-22")
AVAILABLE_RULES = A1_RULES + A4_RULES + A3_RULES


def canonical(obj):
    """Kanonis deterministik (N-05 milik agent4 bila ada, fallback lokal)."""
    if HAVE_A4:
        return a4_canonicalize(obj)  # N-05 agent4, mengembalikan string
    return json.dumps(obj, sort_keys=True, ensure_ascii=True,
                      separators=(",", ":"))


def normalize_run(run, node_type=None, orderless=frozenset(), rules=None,
                  strict=False):
    """Chain penuh = norm_run_items agent4. rules=None berarti semua tersedia."""
    if rules is None:
        rules = [r for r in AVAILABLE_RULES
                 if not (r in A4_RULES and not HAVE_A4)
                 and not (r in A3_RULES and not HAVE_A3)]
    for r in rules:
        if r in A4_RULES and not HAVE_A4:
            raise NormRuleMissing(f"aturan {r} milik agent4 belum tersedia")
        if r in A3_RULES and not HAVE_A3:
            raise NormRuleMissing(f"aturan {r} milik agent3 belum tersedia")
        if r not in AVAILABLE_RULES:
            raise NormRuleMissing(f"aturan {r} tidak dikenal")
    items = run.get("items", run) if isinstance(run, dict) else run
    if not isinstance(items, list):
        raise ValueError("run harus berisi daftar items")
    out = []
    for it in items:
        if "N-10" in rules:
            it = normalize_item_binary(it)
        if "N-12" in rules:
            it = normalize_item_error(it)
        if HAVE_A4 and any(r in rules
                           for r in ("N-01", "N-03", "N-04", "N-05",
                                     "N-07", "N-11")):
            sub = tuple(r for r in ("N-01", "N-03", "N-04", "N-05",
                                    "N-07", "N-11") if r in rules)
            it = a4_normalize_output(it, rules=sub,
                                     strict_num=strict, strict_null=strict)
        if HAVE_A3 and "N-08" in rules:
            it = a3_normalize_timezone(it, all_iso=True)
        if HAVE_A3 and "N-09" in rules:
            it = a3_normalize_paired(it, strict=strict)
        out.append(it)
    if "N-06" in rules:
        out, _ = sort_items_if_orderless(out, node_type, orderless)
    return {"items": out}


def check_workflow(wf):
    """Pre-normalisasi workflow (N-20/21/22). Kembalikan (wf, mode, info)."""
    if not HAVE_A4:
        raise NormRuleMissing("N-20/21 milik agent4 belum tersedia")
    if not HAVE_A3:
        raise NormRuleMissing("N-22 milik agent3 belum tersedia")
    wf, _ = a4_ensure_timezone(dict(wf), "UTC")
    wf = a4_normalize_workflow(wf)
    verdict = a3_classify_trigger(wf)
    skip, reason = a3_should_skip_exec(wf)
    if skip:
        return wf, "SKIP-EXEC", (verdict, reason)
    return wf, "RUNNABLE", verdict


def compare(canon_a, canon_b):
    if canon_a == canon_b:
        return "PASS", []
    diff = list(difflib.unified_diff(
        canon_a.split(","), canon_b.split(","), "A", "B", lineterm=""))
    return "DIFF", diff[:60]


def main(argv):
    import argparse
    ap = argparse.ArgumentParser(description="Differential harness driver")
    ap.add_argument("run_a", nargs="?")
    ap.add_argument("run_b", nargs="?")
    ap.add_argument("--node-type", default=None)
    ap.add_argument("--orderless", nargs="*", default=[])
    ap.add_argument("--rules", nargs="*", default=None)
    ap.add_argument("--strict", action="store_true",
                    help="varian strict N-07/N-09/N-11")
    ap.add_argument("--strict-no-normalization", action="store_true",
                    help="mode audit: NOL aturan")
    ap.add_argument("--check-workflow", default=None)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args(argv)
    if args.selftest:
        return selftest()
    if args.check_workflow:
        try:
            wf = json.load(open(args.check_workflow))
            _, mode, info = check_workflow(wf)
        except (ValueError, NormRuleMissing, FileNotFoundError) as e:
            print(f"ERROR: {e}", file=sys.stderr)
            return 1
        print(json.dumps({"mode": mode, "info": str(info)}, indent=1))
        return 0 if mode == "RUNNABLE" else 2
    if not args.run_a or not args.run_b:
        ap.error("butuh runA dan runB, --check-workflow, atau --selftest")
    rules = [] if args.strict_no_normalization else args.rules
    try:
        run_a = json.load(open(args.run_a))
        run_b = json.load(open(args.run_b))
        na = normalize_run(run_a, args.node_type, set(args.orderless),
                           rules, args.strict)
        nb = normalize_run(run_b, args.node_type, set(args.orderless),
                           rules, args.strict)
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except (ValueError, NormRuleMissing) as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    verdict, diff = compare(canonical(na), canonical(nb))
    applied = list(rules) if rules is not None else "ALL-AVAILABLE"
    report = {
        "verdict": verdict,
        "strict": bool(args.strict),
        "strict_no_normalization": bool(args.strict_no_normalization),
        "rules_applied": applied,
        "modules": {"a1": True, "a3": HAVE_A3, "a4": HAVE_A4},
        "orderless": sorted(set(args.orderless)),
        "items_a": len(na["items"]),
        "items_b": len(nb["items"]),
    }
    print(json.dumps(report, indent=1))
    if diff:
        print("--- diff (potong 60 baris) ---", file=sys.stderr)
        print("\n".join(diff), file=sys.stderr)
    return 0 if verdict == "PASS" else 2


def selftest():
    fails = []

    def check(name, cond):
        print(("PASS " if cond else "FAIL ") + name)
        if not cond:
            fails.append(name)

    a = {"items": [{"json": {"x": 1}, "error": {"message": "boom", "stack": "aaa"}}]}
    b = {"items": [{"json": {"x": 1}, "error": {"message": "boom", "stack": "bbb"}}]}
    va, _ = compare(canonical(normalize_run(a, rules=list(A1_RULES))),
                    canonical(normalize_run(b, rules=list(A1_RULES))))
    check("N-12 stack diabaikan", va == "PASS")

    c = {"items": [{"json": {"x": 1}, "error": {"message": "beda"}}]}
    vc, _ = compare(canonical(normalize_run(a, rules=list(A1_RULES))),
                    canonical(normalize_run(c, rules=list(A1_RULES))))
    check("N-12 message beda = DIFF", vc == "DIFF")

    import hashlib as _hl
    d = {"items": [{"json": {}, "binary": {"f": {"data": "aGVsbG8=",
         "fileName": "a.bin", "mimeType": "application/octet-stream"}}}]}
    nd = normalize_run(d, rules=list(A1_RULES))["items"][0]["binary"]["f"]
    check("N-10 sha256 benar + data dibuang",
          nd.get("__binary_sha256") == _hl.sha256(b"aGVsbG8=").hexdigest()
          and "data" not in nd)
    check("N-10 metadata utuh", nd.get("fileName") == "a.bin")

    e = {"items": [{"json": {"v": 2}}, {"json": {"v": 1}}]}
    f = {"items": [{"json": {"v": 1}}, {"json": {"v": 2}}]}
    ve, _ = compare(canonical(normalize_run(e, "X", rules=list(A1_RULES))),
                    canonical(normalize_run(f, "X", rules=list(A1_RULES))))
    check("N-06 default order sensitif", ve == "DIFF")
    ve2, _ = compare(
        canonical(normalize_run(e, "X", {"X"}, list(A1_RULES))),
        canonical(normalize_run(f, "X", {"X"}, list(A1_RULES))))
    check("N-06 orderless disortir", ve2 == "PASS")

    try:
        normalize_run(e, rules=("N-99",))
        check("aturan asing ditolak", False)
    except NormRuleMissing:
        check("aturan asing ditolak", True)

    check("kanonis deterministik",
          canonical(normalize_run(e, rules=list(A1_RULES)))
          == canonical(normalize_run(e, rules=list(A1_RULES))))

    vs, _ = compare(canonical(normalize_run(a, rules=[])),
                    canonical(normalize_run(b, rules=[])))
    check("strict-no-normalization menangkap stack", vs == "DIFF")

    if HAVE_A3 and HAVE_A4:
        A = [{"json": {"_corpus_meta": 1, "total": 1.0,
                       "at": "2026-09-09T04:00:00.000Z",
                       "executionId": "run-1"}}]
        B = [{"json": {"executionId": "run-2", "total": 1,
                       "at": "2026-09-09T05:00:00.000+01:00"}}]
        vi, _ = compare(canonical(normalize_run({"items": A})),
                        canonical(normalize_run({"items": B})))
        check("integrasi a1+a3+a4 noise=PASS", vi == "PASS")
        C = [{"json": {"total": 2}}]
        vj, _ = compare(canonical(normalize_run({"items": [{"json": {"total": 1}}]})),
                        canonical(normalize_run({"items": C})))
        check("integrasi deviasi 1vs2=DIFF", vj == "DIFF")
        wf_skip = {"name": "w", "nodes": [
            {"name": "C", "type": "n8n-nodes-base.cron",
             "typeVersion": 1, "parameters": {"triggerTimes": "{}"}}],
            "connections": {}}
        _, mode, _ = check_workflow(wf_skip)
        check("cron -> SKIP-EXEC", mode == "SKIP-EXEC")
    else:
        print("SKIP 3 uji integrasi (modul a3/a4 tidak ada)")

    total = 12 if (HAVE_A3 and HAVE_A4) else 9
    ran = 12 if (HAVE_A3 and HAVE_A4) else 9
    print(f"selftest: {ran - len(fails)}/{total} lolos")
    return 0 if not fails else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
