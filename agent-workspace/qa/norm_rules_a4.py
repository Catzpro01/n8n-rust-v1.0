#!/usr/bin/env python3
"""Aturan normalisasi milik agent4 (N-01..N-05, N-07, N-11, N-20, N-21)
untuk differential harness (mandat fern #168/#170; desain AGENT4-NORMALIZER-DESIGN.md #179).

Pembagian (RULE_OWNER, dikonfirmasi #192/#199):
  agent4 -> N-01 strip non-n8n | N-02 id volatil | N-03 timestamp->UTC epoch ms
           | N-04 token $execution.id/resumeUrl | N-05 sortir key rekursif
           | N-07 angka (-0->0, 1.0->1) | N-11 null/undefined/missing setara
           | N-20 strip workflow non-n8n | N-21 timezone eksplisit
  agent1 -> N-06, N-10, N-12 (norm_rules_a1.py + driver diff_harness.py)
  agent3 -> N-08, N-09, N-22

CATATAN TRANSPARANSI (tantangan agent1 atas N-07/N-11, lihat norm_rules_a1.py):
  Menyamakan 1.0 vs 1 atau null vs missing berisiko menutupi deviasi QuickJS-vs-V8
  yang nyata. Solusi: kedua aturan punya toggle eksplisit (default ON utk diff
  tingkat JSON antar-engine; set strict=True utk melaporkan NUMERIC_REPR/NULL_VS_MISSING
  sbg kelas deviasi). Driver memilih — normalizer tidak diam-diam menyamakan.

QA tooling (bukan kode produk Rust). Setiap fungsi + test di bawah.
"""
import json
import re
import datetime

# ---------------------------------------------------------------------------
# N-01 / N-20 — strip field non-kontrak n8n
# ---------------------------------------------------------------------------
# Allowlist kunci root workflow n8n (PRD-1 S4.4). Sisanya dianggap non-n8n.
WORKFLOW_ROOT_ALLOW = {
    "id", "name", "active", "nodes", "connections", "settings", "staticData",
    "versionId", "meta", "tags", "pinData", "createdAt", "updatedAt",
}
# Allowlist kunci node n8n (PRD-1 S4.2).
NODE_ALLOW = {
    "id", "name", "type", "typeVersion", "position", "parameters", "credentials",
    "disabled", "executeOnce", "retryOnFail", "maxTries", "waitBetweenTries",
    "onError", "notes", "alwaysOutputData", "webhookId", "icon",
}


def strip_keys(obj, allow, drop=("_corpus_meta",)):
    """N-01/N-20. Hapus kunci di luar allowlist (rekursif utk nodes/params
    tidak dilakukan — hanya lapisan objek yg diberikan). Kunci dalam `drop`
    dihapus walau ada di allowlist."""
    if isinstance(obj, dict):
        return {k: v for k, v in obj.items()
                if k in allow and k not in drop}
    return obj


def strip_workflow(wf, root_allow=WORKFLOW_ROOT_ALLOW, node_allow=NODE_ALLOW,
                   drop=("_corpus_meta",)):
    """N-20. Bersihkan satu workflow: root + tiap node + kunci drop global."""
    if not isinstance(wf, dict):
        return wf
    out = {k: v for k, v in wf.items() if k in root_allow and k not in drop}
    nodes = out.get("nodes")
    if isinstance(nodes, list):
        out["nodes"] = [
            {k: v for k, v in n.items() if k in node_allow and k not in drop}
            for n in nodes if isinstance(n, dict)
        ]
    return out


def drop_contamination(obj, keys=("_corpus_meta",)):
    """N-01 (output). Hapus kunci kontaminasi non-n8n (_corpus_meta dkk)
    secara rekursif dari output run — tanpa allowlist (output node n8n bebas
    bentuk, allowlist root hanya utk workflow/N-20)."""
    if isinstance(obj, dict):
        return {k: drop_contamination(v, keys) for k, v in obj.items()
                if k not in keys}
    if isinstance(obj, list):
        return [drop_contamination(v, keys) for v in obj]
    return obj


# ---------------------------------------------------------------------------
# N-02 — id/uuid volatil antar-run
# ---------------------------------------------------------------------------
_UUID_RE = re.compile(
    r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")


def is_uuid(s):
    return isinstance(s, str) and bool(_UUID_RE.match(s))


def scrub_volatile_ids(obj, key_names=("id", "uuid", "executionId"),
                       only_uuids=True, recurse=True):
    """N-02. Ganti nilai kunci volatil dgn token {{ID}}.
    only_uuids=True: hanya nilai yg berbentuk UUID (aman utk data bisnis yg
    punya kolom 'id' berupa angka). only_uuids=False: semua nilai kunci tsb."""
    if isinstance(obj, dict):
        out = {}
        for k, v in obj.items():
            if k in key_names and (not only_uuids or is_uuid(v)):
                out[k] = "{{ID}}"
            else:
                out[k] = scrub_volatile_ids(v, key_names, only_uuids, recurse) \
                    if recurse else v
        return out
    if isinstance(obj, list) and recurse:
        return [scrub_volatile_ids(v, key_names, only_uuids, recurse) for v in obj]
    return obj


# ---------------------------------------------------------------------------
# N-03 — timestamp ISO-8601 -> UTC epoch ms
# ---------------------------------------------------------------------------
_ISO_RE = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,9})?(Z|[+-]\d{2}:?\d{2})?$")


def _parse_epoch_ms(s):
    try:
        # hilangkan akhiran Z/offset secara manual utk hindari ambigu
        core = s
        m = re.match(r"^(.*?)(Z|[+-]\d{2}:?\d{2})?$", s)
        if s.endswith("Z"):
            core = s[:-1]
            dt = datetime.datetime.fromisoformat(core.replace("Z", "+00:00"))
            return int(dt.timestamp() * 1000)
        if "+" in s[10:] or "-" in s[10:]:
            dt = datetime.datetime.fromisoformat(s)
            return int(dt.timestamp() * 1000)
        dt = datetime.datetime.fromisoformat(core)
        return int(dt.replace(tzinfo=datetime.timezone.utc).timestamp() * 1000)
    except Exception:
        return None


def normalize_iso(obj, time_keys=None, all_iso=False):
    """N-03. String yg memenuhi pola ISO-8601 (punya komponen waktu) diubah ke
    epoch ms (int). all_iso=False: hanya kunci dlm time_keys (default:
    createdAt/updatedAt/startedAt/finishedAt/ts/time/timestamp)."""
    time_keys = time_keys if time_keys is not None else {
        "createdAt", "updatedAt", "startedAt", "finishedAt", "ts", "time",
        "timestamp", "date", "started", "finished"}
    if isinstance(obj, dict):
        out = {}
        for k, v in obj.items():
            if k in time_keys and isinstance(v, str) and _ISO_RE.match(v):
                e = _parse_epoch_ms(v)
                out[k] = e if e is not None else v
            elif all_iso and isinstance(v, str) and _ISO_RE.match(v):
                e = _parse_epoch_ms(v)
                out[k] = e if e is not None else v
            else:
                out[k] = normalize_iso(v, time_keys, all_iso)
        return out
    if isinstance(obj, list):
        return [normalize_iso(v, time_keys, all_iso) for v in obj]
    return obj


# ---------------------------------------------------------------------------
# N-04 — $execution.id & resumeUrl -> token
# ---------------------------------------------------------------------------
def tokenize_execution(obj, token="{{EXEC_ID}}"):
    """N-04. Ganti nilai kunci (executionId, resumeUrl, execution id di dalam
    ekspresi string) dgn token deterministik."""
    pat_exec = re.compile(r"\{\{\s*\$execution\.id\s*\}\}")
    if isinstance(obj, dict):
        out = {}
        for k, v in obj.items():
            if isinstance(v, str):
                if k == "executionId" or (k == "id" and "exec" in k.lower()):
                    out[k] = token
                elif "resumeUrl" in k and v:
                    out[k] = token
                elif pat_exec.search(v):
                    out[k] = pat_exec.sub(token, v)
                else:
                    out[k] = v
            else:
                out[k] = tokenize_execution(v, token)
        return out
    if isinstance(obj, list):
        return [tokenize_execution(v, token) for v in obj]
    if isinstance(obj, str):
        return pat_exec.sub(token, obj)
    return obj


# ---------------------------------------------------------------------------
# N-05 — kanonisasi: sortir key rekursif (membangun canonical JSON)
# ---------------------------------------------------------------------------
def canonicalize(obj, sort_keys=True):
    """N-05. Representasi kanonis: key terurut rekursif, serialisasi stabil.
    (Angka/tanggal dipegang N-07/N-03; di sini hanya determinisme urutan.)"""
    if isinstance(obj, dict):
        return json.dumps(
            {k: json.loads(canonicalize(v)) for k, v in sorted(obj.items())},
            sort_keys=sort_keys, ensure_ascii=True, separators=(",", ":"))
    if isinstance(obj, list):
        return "[" + ",".join(canonicalize(v) for v in obj) + "]"
    if isinstance(obj, str):
        return json.dumps(obj, ensure_ascii=True)
    if isinstance(obj, bool):
        return "true" if obj else "false"
    if obj is None:
        return "null"
    if isinstance(obj, float):
        return repr(_num_canon(obj))
    return str(obj)


def normalize_sort(obj):
    """N-05 (struktural): kembalikan objek baru dgn semua key terurut."""
    if isinstance(obj, dict):
        return {k: normalize_sort(obj[k]) for k in sorted(obj)}
    if isinstance(obj, list):
        return [normalize_sort(v) for v in obj]
    return obj


# ---------------------------------------------------------------------------
# N-07 — angka: -0 -> 0 ; 1.0 -> 1 (toggle strict utk laporkan deviasi)
# ---------------------------------------------------------------------------
def _num_canon(x):
    if isinstance(x, float):
        if x == 0:
            return 0
        if x.is_integer():
            return int(x)
    return x


def normalize_numbers(obj, strict=False, _seen=None):
    """N-07. strict=False: samakan -0->0 dan 1.0->1 (default diff JSON).
    strict=True: JANGAN samakan 1.0 vs 1; -0 tetap -> 0 (JSON tak punya -0
    utk serialisasi standar)."""
    if isinstance(obj, float):
        if obj == 0:
            return 0
        return obj if strict else _num_canon(obj)
    if isinstance(obj, dict):
        return {k: normalize_numbers(v, strict) for k, v in obj.items()}
    if isinstance(obj, list):
        return [normalize_numbers(v, strict) for v in obj]
    return obj


# ---------------------------------------------------------------------------
# N-11 — null vs missing vs (JS) undefined setara
# ---------------------------------------------------------------------------
def drop_nulls(obj, strict=False):
    """N-11. strict=False: hapus pasangan (k, None) dari dict — null dianggap
    setara dgn key hilang. strict=True: pertahankan null (deviasi dilaporkan).
    Catatan: undefined tidak pernah eksis di JSON; yg relevan antar-engine
    adalah satu sisi menghilangkan kunci vs sisi lain menulis null."""
    if isinstance(obj, dict):
        out = {}
        for k, v in obj.items():
            if v is None and not strict:
                continue
            out[k] = drop_nulls(v, strict)
        return out
    if isinstance(obj, list):
        return [drop_nulls(v, strict) for v in obj]
    return obj


# ---------------------------------------------------------------------------
# N-21 — timezone eksplisit utk determinisme $now
# ---------------------------------------------------------------------------
def ensure_timezone(wf, tz="UTC"):
    """N-21. Pastikan workflow.settings.timezone eksplisit. Hanya mengisi bila
    kosong; TIDAK mengubah timezone yg sudah ada (menghormati semantik)."""
    if not isinstance(wf, dict):
        return wf, False
    settings = wf.get("settings")
    if settings is None:
        wf["settings"] = {"timezone": tz}
        return wf, True
    if isinstance(settings, dict) and not settings.get("timezone"):
        settings["timezone"] = tz
        return wf, True
    return wf, False


# ---------------------------------------------------------------------------
# Pipeline siap-pakai
# ---------------------------------------------------------------------------
def normalize_workflow(wf, rules=("N-20", "N-21", "N-01", "N-02"),
                       tz="UTC", only_uuids=True):
    """Normalisasi workflow SEBELUM eksekusi (N-20..N-22 area)."""
    wf, _ = ensure_timezone(dict(wf), tz) if "N-21" in rules else (wf, False)
    if "N-20" in rules:
        wf = strip_workflow(wf)
    return wf


def normalize_output(obj, rules=("N-01", "N-03", "N-04", "N-05", "N-07", "N-11"),
                     strict_num=False, strict_null=False, all_iso=False):
    """Normalisasi SATU output run/item utk diff (default ruleset agent4)."""
    if "N-01" in rules:
        obj = drop_contamination(obj)
    if "N-03" in rules:
        obj = normalize_iso(obj, all_iso=all_iso)
    if "N-04" in rules:
        obj = tokenize_execution(obj)
    if "N-11" in rules:
        obj = drop_nulls(obj, strict=strict_null)
    if "N-07" in rules:
        obj = normalize_numbers(obj, strict=strict_num)
    if "N-05" in rules:
        obj = normalize_sort(obj)
    return obj


# ---------------------------------------------------------------------------
# Self-test (standar: tiap aturan punya test; fail loudly)
# ---------------------------------------------------------------------------
def selftest():
    fails = []

    def eq(a, b, label):
        if a != b:
            fails.append(f"{label}: {a!r} != {b!r}")

    # N-01 strip
    eq(strip_keys({"_corpus_meta": 1, "name": "x", "junk": 2},
                  WORKFLOW_ROOT_ALLOW),
       {"name": "x"}, "N-01")
    # N-02 uuid
    eq(scrub_volatile_ids({"id": "3f2c9b1a-0000-4000-8000-000000000001",
                           "count": 5}),
       {"id": "{{ID}}", "count": 5}, "N-02 uuid")
    eq(scrub_volatile_ids({"id": 42}), {"id": 42}, "N-02 non-uuid aman")
    # N-03 iso
    iso_in = "2026-09-09T04:00:00.000Z"
    eq(normalize_iso({"createdAt": iso_in, "x": "a"}),
       {"createdAt": _parse_epoch_ms(iso_in), "x": "a"}, "N-03 epoch")
    eq(normalize_iso({"createdAt": "not-a-date"})["createdAt"],
       "not-a-date", "N-03 non-date aman")
    # N-04 token
    eq(tokenize_execution({"executionId": "abc123",
                           "resumeUrl": "https://x/resume/1"}),
       {"executionId": "{{EXEC_ID}}", "resumeUrl": "{{EXEC_ID}}"}, "N-04")
    # N-05 sort
    eq(normalize_sort({"b": 1, "a": {"d": 2, "c": 3}}),
       {"a": {"c": 3, "d": 2}, "b": 1}, "N-05")
    # N-07 angka
    eq(normalize_numbers({"f": 1.0, "z": -0.0, "i": 2}),
       {"f": 1, "z": 0, "i": 2}, "N-07")
    eq(normalize_numbers({"f": 1.0}, strict=True), {"f": 1.0}, "N-07 strict")
    # N-11 null
    eq(drop_nulls({"a": None, "b": 1}), {"b": 1}, "N-11")
    eq(drop_nulls({"a": None}, strict=True), {"a": None}, "N-11 strict")
    # N-20 workflow strip
    wf = strip_workflow({"name": "w", "junk": 1,
                         "nodes": [{"name": "n", "type": "t", "zzz": 2}],
                         "connections": {}})
    eq(wf, {"name": "w", "nodes": [{"name": "n", "type": "t"}],
            "connections": {}}, "N-20")
    # N-21 timezone
    wf2, ch = ensure_timezone({"settings": {}}, "Asia/Jakarta")
    eq(wf2["settings"], {"timezone": "Asia/Jakarta"}, "N-21")
    eq(ch, True, "N-21 changed")
    if fails:
        raise AssertionError("\n".join(fails))
    return f"selftest OK: {len(fails)} fail (semua aturan N-01..N-05,N-07,N-11,N-20,N-21 lulus)"


if __name__ == "__main__":
    print(selftest())
