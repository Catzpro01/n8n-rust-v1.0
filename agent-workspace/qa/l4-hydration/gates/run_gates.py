#!/usr/bin/env python3
"""Gate runner G-L4-1..G-L4-5 (+ G-L4-6 dasar) — AGENT7-W2-SCRAPE-L4-SPEC §8.
Jalankan: python3 gates/run_gates.py ; hasil: semua PASS = syarat DESAIN v0.1 terpenuhi.
"""
import hashlib
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from l4_hydration_ref import extract, L4Err, L4_PARSE, L4_HYDRATION_NOT_FOUND, L4_SELECTOR_MISSING, L4_SELECTOR_EMPTY, sanitize_pii

FX = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "fixtures")


def load(name):
    return open(os.path.join(FX, name + ".html")).read()


results = []


def check(gate, cond, detail=""):
    results.append((gate, bool(cond), detail))
    print(f"[{'PASS' if cond else 'FAIL'}] {gate} {detail}")


# G-L4-1: 8 varian HTML diklasifikasikan benar
try:
    r1 = extract(load("f1_next13"))
    check("G-L4-1a", r1["key"] == "__NEXT_DATA__" and r1["payload_json"]["props"]["pageProps"]["ticker"]["price"] == 64000)
    r2 = extract(load("f2_charset"))
    check("G-L4-1b", r2["key"] == "__NEXT_DATA__" and r2["payload_json"]["page"] == "/news")
    r3 = extract(load("f3_orders"), selector_path="props.pageProps.orders")
    check("G-L4-1c", len(r3["payload_json"]) == 2 and r3["payload_json"][1]["side"] == "sell")
    r4 = extract(load("f4_nuxt_obj"))
    check("G-L4-1d", r4["key"] == "__NUXT__" and r4.get("note") == "js_object_best_effort")
    try:
        extract(load("f5_plain"))
        check("G-L4-1e", False, "f5 harusnya L4_HYDRATION_NOT_FOUND")
    except L4Err as e:
        check("G-L4-1e", e.code == L4_HYDRATION_NOT_FOUND)
    r6 = extract(load("f6_both"))
    check("G-L4-1f", r6["key"] == "__NEXT_DATA__", "prioritas NEXT_DATA saat keduanya ada")
    try:
        extract(load("f7_malformed"))
        check("G-L4-1g", False, "f7 harusnya L4_PARSE")
    except L4Err as e:
        check("G-L4-1g", e.code == L4_PARSE)
    r8 = extract(load("f8_huge"), max_bytes=1024)
    check("G-L4-1h", r8["truncated"] is True and r8["byte_len"] > 1024 and len(json.dumps(r8["payload_json"])) > 0,
          f"truncated={r8['truncated']} byte_len={r8['byte_len']}")
except L4Err as e:
    check("G-L4-1", False, str(e))

# G-L4-2a: selector benar / path null -> EMPTY (SUCCESS payload kosong, bukan panic)
r = extract(load("f3_orders"), selector_path="props.pageProps.orders.0.id")
check("G-L4-2a1", r["payload_json"] == "o1", "array-index selector")
try:
    import json as _j
    f1 = _j.loads(_j.dumps(extract(load("f1_next13"))["payload_json"]))
    # tambah field null utk uji EMPTY
    html_mod = load("f1_next13").replace('"gssp":true', '"gssp":true,"pageProps":{"ticker":{"symbol":null}}')
    from l4_hydration_ref import L4_SELECTOR_EMPTY
    # fixture sintetis: path ada tapi null
    fx_empty = load("f1_next13").replace('"ts": 1725000000', '"ts": 1725000000, "extra": {"target": null}')
    try:
        extract(fx_empty, selector_path="props.pageProps.ticker.extra.target")
        check("G-L4-2a2", False, "null path harusnya EMPTY error")
    except L4Err as e:
        check("G-L4-2a2", e.code == L4_SELECTOR_EMPTY, e.code)
except Exception as e:
    check("G-L4-2a2", False, str(e))
# G-L4-2b: komponen path HILANG -> MISSING (beda kode)
try:
    extract(load("f1_next13"), selector_path="props.pageProps.nonexistent.deep")
    check("G-L4-2b", False, "harusnya L4_SELECTOR_MISSING")
except L4Err as e:
    check("G-L4-2b", e.code == L4_SELECTOR_MISSING, e.code)

# G-L4-2c: array kosong = data legit nol (SUCCESS, bukan error)
try:
    rc = extract(load("f1_next13").replace('"ts": 1725000000', '"ts": 1725000000, "emptylist": {"items": []}'), selector_path="props.pageProps.ticker.emptylist.items")
    check("G-L4-2c", isinstance(rc["payload_json"], list) and len(rc["payload_json"]) == 0)
except L4Err as e:
    check("G-L4-2c", False, str(e))

# G-L4-3: OVERSIZE masih JSON valid setelah potong
try:
    big = extract(load("f8_huge"), max_bytes=2048)
    json.dumps(big["payload_json"])  # harus tidak raise
    check("G-L4-3", big["truncated"] is True)
except (L4Err, json.JSONDecodeError) as e:
    check("G-L4-3", False, str(e))

# G-L4-4: fuzz pendek 100 korpus -> 0 panic, semua error = L4_PARSE/NOT_FOUND
import random
random.seed(7)
corpus = [bytes(random.randrange(256) for _ in range(random.randrange(2, 400))) for _ in range(100)]
panic = 0
for c in corpus:
    try:
        extract(c)
    except L4Err:
        pass
    except Exception:
        panic += 1
check("G-L4-4", panic == 0, f"panic={panic}")

# G-L4-5: determinisme byte-identik 100 run
h = set()
for _ in range(100):
    rr = extract(load("f1_next13"), selector_path="props.pageProps.ticker")
    h.add(hashlib.sha256(json.dumps(rr, sort_keys=True).encode()).hexdigest())
check("G-L4-5", len(h) == 1, f"variasi hash={len(h)-1}")

# G-L4-6 (dasar, mandiri): sanitasi PII
sample = 'contact a@b.com or +62-812-3456-7890 and token Bearer abc123XYZ bear ghp_0123456789012345678901234567890123456789'
s = sanitize_pii(sample)
ok = "[EMAIL]" in s and "[TOKEN]" in s and "[PHONE]" in s
leak = any(t in s for t in ["a@b.com", "ghp_012345", "812-3456"])
check("G-L4-6", ok and not leak, s[:90])

fails = [g for g, ok, _ in results if not ok]
print(f"\n=== {len(results)-len(fails)}/{len(results)} gates PASS ===")
sys.exit(1 if fails else 0)
